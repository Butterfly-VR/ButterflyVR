// functionallity for the NetNodeManager server
use crate::messages::*;
use crate::net_nodes::NetworkedNode;
use crate::serializer::*;
use crate::voice;
use crate::voice::FRAME_LENGTH;
use bitvec::prelude::*;
use build_time::build_time_utc;
use std::collections::{HashSet, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use std::{cmp, collections::HashMap};

use godot::classes::Engine;
use godot::prelude::*;
use netcode::{ClientIndex, ConnectToken, NetcodeSocket, Server};
const CHANNEL_ACK: u16 = u16::MAX;
const BYTE: usize = 8;
const BYTES2: usize = 16;
const BYTES8: usize = 64;
const PACKET_HEADER_SIZE: usize = BYTES2 + BYTES8;
const PACKET_HEADER_SIZE_ACK: usize = BYTES2;
const CHANNEL1_HEADER_SIZE: usize = BYTES8;
const HIT_RATE_HISTORY_LENGTH: usize = 128;
const CHANNEL_CLIENT_ID: u16 = u16::MAX - 1;
#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct NetNodeServer {
    #[var]
    pub id: u16,
    next_id: u16,
    pub networked_nodes: Vec<Gd<NetworkedNode>>,
    voice_manager: voice::VoiceStreamManager,
    server_networker: ServerNetworker,
    message_buffer: VecDeque<BitVec<u64, Lsb0>>,
    message_handlers: HashMap<u16, Gd<MessageHandler>>,
    base: Base<Node>,
}

#[godot_api]
pub impl NetNodeServer {
    #[signal]
    pub fn player_joined(player: u16);
    #[signal]
    pub fn player_left(player: u16);
    pub fn register_node(&mut self, new_node_ref: Gd<NetworkedNode>, new_node: &mut NetworkedNode) {
        self.queue_message(MessageHandler::create_id_sync_message(
            new_node_ref.clone().upcast(),
            new_node.objectid,
            Some(new_node.owner_id),
        ));
        self.networked_nodes.push(new_node_ref);
    }
    pub fn unregister_node(&mut self, removed_node_ref: Gd<NetworkedNode>) {
        for client in self.server_networker.clients.values_mut() {
            if let Some(idx) = client
                .priorities
                .iter()
                .position(|x| x.0 == removed_node_ref)
            {
                client.priorities.remove(idx);
            }
        }
        if let Some(idx) = self
            .networked_nodes
            .iter()
            .position(|x| *x == removed_node_ref)
        {
            self.networked_nodes.remove(idx);
        }
    }
    pub fn unregister_all(&mut self) {
        // todo: update this
        self.next_id = 0;
        self.networked_nodes.clear();
    }
    pub fn get_next_object_id(&mut self) -> u16 {
        self.next_id += 1;
        self.next_id
    }
    pub fn register_message(&mut self, handler: Gd<MessageHandler>, message_type: u16) {
        self.message_handlers.insert(message_type, handler);
    }
    pub fn unregister_message(&mut self, message_type: u16) {
        self.message_handlers.remove(&message_type);
    }
    pub fn queue_message(&mut self, message: BitVec<u64, Lsb0>) {
        self.message_buffer.push_back(message);
    }
    pub fn start_server(&mut self, bind_addr: String, private_key: [u8; 32]) {
        const PROTOCOL_ID: u64 = 0;
        self.server_networker = ServerNetworker {
            server: Server::new(bind_addr, PROTOCOL_ID, private_key).unwrap(),
            ..Default::default()
        };
    }
    pub fn get_next_client(&mut self) -> PackedByteArray {
        let mut result: PackedByteArray = PackedByteArray::new();
        let tmp: [u8; netcode::CONNECT_TOKEN_BYTES] =
            self.server_networker.get_token().try_into_bytes().unwrap();
        result.extend(tmp);
        result
    }
    pub fn register_player_object(&mut self, client_id: u16, object: Gd<Node3D>) {
        let client = self
            .server_networker
            .clients
            .values_mut()
            .find(|x| x.id == client_id);
        if let Some(client) = client {
            client.player_position_object = Some(object);
        }
    }
    fn update_network_nodes(&mut self) {
        for client in self.server_networker.clients.iter_mut() {
            for packet_tuple in client.1.packet_buffers.get_mut(0).unwrap().drain(..) {
                let packet = packet_tuple.0;
                let mut pointer: usize = PACKET_HEADER_SIZE + CHANNEL1_HEADER_SIZE;
                while pointer + BYTES2 <= packet.len() {
                    let next_obj: u16 = packet[pointer..pointer + BYTES2].load_le();
                    pointer += BYTES2;
                    let tmp = self
                        .networked_nodes
                        .iter()
                        .find(|x| Gd::bind(x).objectid == next_obj);
                    if tmp.is_none() {
                        godot_warn!(
                            "got update for nonexistant netnode with objectid: {:#?}",
                            next_obj
                        ); // will give a few spurious errors if we get sync data for a netnode before the creation event
                        break;
                    }
                    let node = Gd::bind(tmp.unwrap());
                    let types_buff: Vec<NetworkedValueTypes> = node.get_networked_values_types();
                    node.update_networked_values(&mut pointer, packet.as_bitslice(), &types_buff);
                }
            }
        }
    }

    fn send_packets_server(&mut self) {
        const PACKET_MAX_SIZE_THRESHOLD: usize = 80;
        const BANDWIDTH_BUDGET: usize = 512000;
        const MAX_SINGLE_PACKET_PAYLOAD_LENGTH: usize = 4800;
        let bandwidth_per_tick =
            BANDWIDTH_BUDGET / Engine::singleton().get_physics_ticks_per_second() as usize;
        let networker = &mut self.server_networker;
        let mut buffer: Vec<(ClientIndex, BitVec<u64>, u16)> = Vec::new();
        for client in networker.clients.iter_mut() {
            client.1.remaining_bandwidth += bandwidth_per_tick;
            'outer: while client.1.remaining_bandwidth > PACKET_MAX_SIZE_THRESHOLD {
                if !client.1.id_received {
                    buffer.push((
                        *client.0,
                        BitVec::<u64>::from_element(client.1.id as u64),
                        CHANNEL_CLIENT_ID,
                    ));
                    client.1.id_received = true;
                    break;
                }
                let mut packet: BitVec<u64> =
                    BitVec::with_capacity(MAX_SINGLE_PACKET_PAYLOAD_LENGTH);
                // acks
                for ack in client.1.waiting_acks.iter() {
                    packet.extend(ack.0.view_bits::<Lsb0>());
                    packet.extend(ack.1.view_bits::<Lsb0>());
                }
                client.1.waiting_acks.clear();
                if !packet.is_empty() {
                    client.1.remaining_bandwidth =
                        client.1.remaining_bandwidth.saturating_sub(packet.len());
                    buffer.push((*client.0, packet, CHANNEL_ACK));
                }
                // channel 3 (messages)
                while self.message_buffer.len() > client.1.message_buffer_position {
                    let mut packet: BitVec<u64> =
                        BitVec::with_capacity(MAX_SINGLE_PACKET_PAYLOAD_LENGTH);
                    let message = &self.message_buffer[client.1.message_buffer_position];
                    packet.extend(message);
                    if (client.1.remaining_bandwidth as i64 - packet.len() as i64) < 0 {
                        break 'outer;
                    }
                    client.1.remaining_bandwidth -= packet.len();
                    buffer.push((*client.0, packet, 3));
                    client.1.message_buffer_position += 1;
                }
                let mut packet: BitVec<u64> =
                    BitVec::with_capacity(MAX_SINGLE_PACKET_PAYLOAD_LENGTH);
                // channel 2 (initial sync)
                if client.1.sync_progress < (self.networked_nodes.len() as u64).saturating_sub(1)
                    && !client.1.finished_sync
                {
                    packet.extend(0u64.view_bits::<Lsb0>());
                    for index in client.1.sync_progress..self.networked_nodes.len() as u64 {
                        let node: GdRef<NetworkedNode> =
                            self.networked_nodes[index as usize].bind();
                        let tmp = node.get_byte_data(&node.get_networked_values_types());
                        if tmp.len() + packet.len()
                            > cmp::min(
                                client.1.remaining_bandwidth,
                                MAX_SINGLE_PACKET_PAYLOAD_LENGTH,
                            )
                        {
                            break;
                        }
                        client.1.sync_progress = index;
                        packet.extend(tmp);
                    }

                    if packet.len() > CHANNEL1_HEADER_SIZE {
                        client.1.remaining_bandwidth -= packet.len();
                        buffer.push((*client.0, packet, 2));
                        continue;
                    }
                } else {
                    client.1.finished_sync = true;
                }
                let mut packet: BitVec<u64> =
                    BitVec::with_capacity(MAX_SINGLE_PACKET_PAYLOAD_LENGTH);
                // channel 1 (syncing)
                packet.extend(
                    (SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64)
                        .view_bits::<Lsb0>(),
                );
                if client.1.priorities.len() != self.networked_nodes.len() {
                    client.1.priorities.clear();
                    for node_ref in self.networked_nodes.iter() {
                        client.1.priorities.push((node_ref.clone(), 0));
                    }
                }
                client.1.priorities.sort_unstable_by(|a, b| a.1.cmp(&b.1));
                for value in client.1.priorities.iter_mut() {
                    let node_ref = &value.0;
                    let node = Gd::bind(node_ref);
                    if (value.1 != 0) && (node.owner_id != client.1.id) {
                        let tmp = node.get_byte_data(&node.get_networked_values_types());
                        if tmp.len() + packet.len()
                            > cmp::min(
                                client.1.remaining_bandwidth,
                                MAX_SINGLE_PACKET_PAYLOAD_LENGTH,
                            )
                        {
                            break;
                        }
                        packet.extend(tmp);
                        value.1 = 0;
                    }
                }
                if packet.len() > CHANNEL1_HEADER_SIZE {
                    client.1.remaining_bandwidth -= packet.len();
                    buffer.push((*client.0, packet, 1));
                    continue;
                }
                packet.clear();

                break;
            }
        }
        for packet in buffer {
            networker.send(packet.1.as_bitslice(), packet.2, packet.0);
        }
    }
    fn tick_server(&mut self) {
        // cycle buffers, poll for new packets from the networker
        let new_players = self.server_networker.poll();
        if !new_players.is_empty() {
            for player in new_players {
                godot_warn!("new player");
                self.signals().player_joined().emit(player);
            }
        }
        // check for and handle disconnected clients
        loop {
            let mut dc_client: Option<ClientIndex> = None;
            for client in self.server_networker.clients.iter() {
                if self.server_networker.server.client_id(*client.0).is_none() {
                    dc_client = Some(client.0.clone());
                    let id = client.1.id;
                    self.signals().player_left().emit(id);
                    break;
                }
            }
            if dc_client.is_some() {
                self.server_networker.clients.remove(&dc_client.unwrap());
            } else {
                break;
            }
        }
        let networker = &mut self.server_networker;
        let mut current_frame_hit_rates: HashMap<ClientIndex, u64> =
            HashMap::from_iter(networker.clients.iter().map(|x| (*x.0, 0)));
        let mut current_frame_miss_rates: HashMap<ClientIndex, u64> =
            HashMap::from_iter(networker.clients.iter().map(|x| (*x.0, 0)));
        // then for each packet check channel id and:
        for packet_tuple in networker.packet_buffer.drain(..) {
            if !networker.clients.contains_key(&packet_tuple.1) {
                continue;
            }
            let packet = &packet_tuple.0;
            let client = networker.clients.get_mut(&packet_tuple.1).unwrap();
            if packet.len() < BYTES2 {
                godot_warn!("got packet with invalid size");
                continue;
            }
            let mut pointer: usize = 0;
            let channelid: u16 = packet[pointer..pointer + BYTES2].load_le();
            pointer += BYTES2;
            let packet_number: u64 = packet[pointer..pointer + BYTES8].load_le();
            pointer += BYTES8;
            match channelid {
                0 => {
                    // control packets
                }
                // get channel 1 data for inputs and remote owned objects and send to buffer cycle
                1 => {
                    if packet.len() < PACKET_HEADER_SIZE + CHANNEL1_HEADER_SIZE {
                        godot_warn!("got c1 packet with invalid size");
                        continue;
                    }
                    const PACKET_LATENCY_DISCARD_THRESHOLD: Duration = Duration::from_millis(1000);
                    const LATENCY_BUFFER_MAX_SIZE: usize = 8;

                    let packet_send_time_utc: u64 = packet[pointer..pointer + BYTES8].load_le();
                    let packet_send_time: Duration = Duration::from_millis(packet_send_time_utc);
                    let tick_length: Duration = Duration::from_secs_f32(
                        1.0 / Engine::singleton().get_physics_ticks_per_second() as f32,
                    );

                    // initial time sync
                    let current_time: Duration =
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

                    // latency calculations
                    let latency: Duration = current_time - packet_send_time;
                    if latency > PACKET_LATENCY_DISCARD_THRESHOLD {
                        godot_warn!(
                            "ignoring packet with high latency: {:#?}ms",
                            latency.as_millis()
                        );
                        continue;
                    }
                    if client.latency_buffer.len() >= LATENCY_BUFFER_MAX_SIZE {
                        client.latency_buffer.pop_front();
                    }
                    client.latency_buffer.push_back(latency);
                    client.latency = Duration::from_millis(
                        client
                            .latency_buffer
                            .iter()
                            .map(|x| x.as_millis())
                            .sum::<u128>() as u64
                            / client.latency_buffer.len() as u64,
                    );
                    // handle buffers
                    let mut buffer_max_jitter: i128 = (client.packet_buffers.len()) as i128
                        * -(tick_length.as_millis() as i128 / 2);
                    let mut packet_accepted: bool = false;
                    let client_index = packet_tuple.1;
                    for buffer in client.packet_buffers.iter_mut() {
                        let buffer_min_jitter = buffer_max_jitter;
                        buffer_max_jitter += tick_length.as_millis() as i128;

                        if (latency.as_millis() as i128 - client.latency.as_millis() as i128)
                            <= buffer_max_jitter
                            && (latency.as_millis() as i128 - client.latency.as_millis() as i128)
                                >= buffer_min_jitter
                        {
                            buffer.push(packet_tuple);
                            packet_accepted = true;
                            break;
                        }
                    }
                    if packet_accepted {
                        *current_frame_hit_rates.get_mut(&client_index).unwrap() += 1;
                    } else {
                        godot_warn!("ignoring packet due to jitter",);

                        *current_frame_miss_rates.get_mut(&client_index).unwrap() += 1;
                    }
                }
                3 => {
                    if packet_number == client.next_c3_packet_number {
                        client.next_c3_packet_number += 1;
                        self.message_buffer.push_back(packet[pointer..].to_bitvec());
                        let message_type: u16 = packet[pointer..pointer + BYTES2].load_le();
                        pointer += BYTES2;
                        if let Some(handler) = self.message_handlers.get_mut(&message_type) {
                            handler
                                .bind_mut()
                                .handle_message(packet.as_bitslice(), &mut pointer);
                        } else {
                            godot_warn!(
                                "received unhandled message with type: {:#?}",
                                message_type
                            );
                        }
                        // since the next packet might of been buffered to wait for this one, we go through the buffer and check each packet
                        // easy optimisation would be storing the packet number and keeping the vec sorted so we only check once
                        let mut index: usize = 0;
                        loop {
                            let packet = client.c3_buffered_packets.get(index);
                            if packet.is_none() {
                                break;
                            }
                            let packet = packet.unwrap();
                            if packet.len() < PACKET_HEADER_SIZE {
                                break;
                            }
                            let mut pointer: usize = BYTES2;
                            let packet_number: u64 = packet[pointer..pointer + BYTES8].load_le();
                            pointer += BYTES8;
                            if packet_number == client.next_c3_packet_number {
                                client.next_c3_packet_number += 1;
                                self.message_buffer.push_back(packet[pointer..].to_bitvec());
                                let message_type: u16 = packet[pointer..pointer + BYTES2].load_le();
                                pointer += BYTES2;
                                if let Some(handler) = self.message_handlers.get_mut(&message_type)
                                {
                                    handler
                                        .bind_mut()
                                        .handle_message(packet.as_bitslice(), &mut pointer);
                                } else {
                                    godot_warn!(
                                        "received unhandled message with type: {:#?}",
                                        message_type
                                    );
                                }
                                client.c3_buffered_packets.swap_remove(index);
                                index = 0;
                            } else {
                                index += 1;
                            }
                        }
                    } else {
                        client.c3_buffered_packets.push(packet_tuple.0);
                    }
                }
                5 => {
                    let buffer: Vec<u8> = packet.chunks(BYTE).map(|x| x.load_le::<u8>()).collect();
                    if client.next_c5_packet_number <= packet_number {
                        client
                            .voice_packet_buffer
                            .push((packet_number, buffer[10..].into_iter().copied().collect()));
                    }
                }
                _ => {
                    godot_warn!("unhandled channel: {:#?}", channelid);
                }
            }
        }
        for value in current_frame_hit_rates {
            // dont grow unless packet loss history is partly full to avoid unneeded growth from initial variance
            // no real reason this is here specifically just need somewhere we can get client latency info
            const JITTER_BUFFER_INCREASE_THRESHOLD: f32 = 0.01;
            let client = networker.clients.get_mut(&value.0).unwrap();
            if client.c1_latency_info.c1_miss_rate_average_percent
                > JITTER_BUFFER_INCREASE_THRESHOLD
                && client.c1_latency_info.c1_miss_rate_last_frames.len()
                    >= HIT_RATE_HISTORY_LENGTH / 2
            {
                client.packet_buffers.push_back(Vec::new());
                client.c1_latency_info = LatencyInfo::default()
            }
            networker
                .clients
                .get_mut(&value.0)
                .unwrap()
                .c1_latency_info
                .c1_hit_rate_last_frames
                .push_back(value.1);
        }

        for value in current_frame_miss_rates {
            networker
                .clients
                .get_mut(&value.0)
                .unwrap()
                .c1_latency_info
                .c1_miss_rate_last_frames
                .push_back(value.1);
        }
        for hit_rates in networker
            .clients
            .values_mut()
            .map(|x| &mut x.c1_latency_info.c1_hit_rate_last_frames)
        {
            if hit_rates.len() > HIT_RATE_HISTORY_LENGTH {
                hit_rates.pop_front();
            }
        }
        for miss_rates in networker
            .clients
            .values_mut()
            .map(|x| &mut x.c1_latency_info.c1_miss_rate_last_frames)
        {
            if miss_rates.len() > HIT_RATE_HISTORY_LENGTH {
                miss_rates.pop_front();
            }
        }
        for latency_info in networker
            .clients
            .values_mut()
            .map(|x| &mut x.c1_latency_info)
        {
            latency_info.c1_hit_rate_average =
                latency_info.c1_hit_rate_last_frames.iter().sum::<u64>() as f32
                    / latency_info.c1_hit_rate_last_frames.len() as f32;
            latency_info.c1_miss_rate_average =
                latency_info.c1_miss_rate_last_frames.iter().sum::<u64>() as f32
                    / latency_info.c1_miss_rate_last_frames.len() as f32;
            latency_info.c1_miss_rate_average_percent = latency_info.c1_miss_rate_average
                / (latency_info.c1_hit_rate_average + latency_info.c1_miss_rate_average);
        }
    }
    fn process_voice_input(&mut self) {
        const DISTANCE_FALLOFF_START: f32 = 10.0;
        const DISTANCE_FALLOFF_END: f32 = 15.0;
        for client in self.server_networker.clients.values_mut() {
            if client.voice_input_stream.is_none() {
                client.voice_input_stream = Some(self.voice_manager.create_decoder());
            }

            if client.voice_packet_buffer.len() < 3 {
                // if buffer is small then we are consuming packets too fast for the client to keep up and need to slow down
                return;
            }
            let buffer: Vec<u8>;
            if let Some(packet_idx) = client
                .voice_packet_buffer
                .iter()
                .position(|x| x.0 == client.next_c5_packet_number)
            {
                buffer = client.voice_packet_buffer.swap_remove(packet_idx).1;
            } else {
                buffer = Vec::new();
            }
            client.next_c5_packet_number += 1;
            client.audio_input_buffer = self
                .voice_manager
                .decode_audio(client.voice_input_stream.unwrap(), &buffer);
        }
        // todo: would probably be a good idea to use an audio library to handle this for us
        // then we could properly spatialize audio with hrtf, model room dampening, and handle falloff better
        let audio_streams: Vec<(Vec<f32>, ClientIndex)> = self
            .server_networker
            .clients
            .values()
            .map(|x| (x.audio_input_buffer.clone(), x.index))
            .collect();
        let positions: Vec<(Vector3, ClientIndex)> = self
            .server_networker
            .clients
            .values()
            .map(|x| {
                (
                    {
                        if x.player_position_object.is_some() {
                            x.player_position_object.clone().unwrap().get_position()
                        } else {
                            Vector3::INF
                        }
                    },
                    x.index,
                )
            })
            .collect();
        let networker = &mut self.server_networker;
        let mut outputs: Vec<(bitvec::vec::BitVec<u64>, ClientIndex)> =
            Vec::with_capacity(networker.clients.len());
        // probably dont need this assert but just in case
        assert!(audio_streams.len() == positions.len());
        assert!(
            audio_streams
                .windows(2)
                .all(|x| x[0].0.len() == x[1].0.len())
        ); // asserts all audio streams are same length, should replace with proper handling for malformed data eventually
        for client in networker.clients.values_mut() {
            if client.player_position_object.is_none() {
                continue;
            }
            let listener_pos: Vector3 = client
                .player_position_object
                .as_ref()
                .unwrap()
                .get_position();
            let listener_rot: Quaternion = client
                .player_position_object
                .as_ref()
                .unwrap()
                .get_quaternion()
                .inverse();
            let mut final_audio: Vec<(f32, f32)> = Vec::new();
            for audio_source in 0..audio_streams.len() {
                if audio_streams[audio_source].1 == client.index {
                    continue;
                }
                let l_r_bias: f32; // directionality, -1.0 for fully left, 1.0 for fully right
                let mut volume: f32 = 1.0;
                let buffer: Vec<(f32, f32)>;
                let audio: &[f32] = &audio_streams[audio_source].0;
                let position: Vector3 = positions[audio_source].0;

                if position == Vector3::INF {
                    continue;
                }

                let relative_position: Vector3 = listener_rot * (position - listener_pos);

                let distance = relative_position.length();
                if distance > DISTANCE_FALLOFF_END {
                    continue;
                }
                if distance > DISTANCE_FALLOFF_START {
                    volume = 1.0
                        - ((distance - DISTANCE_FALLOFF_START)
                            / (DISTANCE_FALLOFF_END - DISTANCE_FALLOFF_START));
                }

                // pretty bad spatial audio, should probably do this better or replace it with a library
                l_r_bias = relative_position.normalized_or_zero().x;
                let right_bias = (l_r_bias / 2.0) + 0.5;
                let left_bias = ((-l_r_bias) / 2.0) + 0.5;
                buffer = audio
                    .iter()
                    .map(|x| (x * volume * left_bias, x * volume * right_bias))
                    .collect();

                if final_audio.is_empty() {
                    final_audio = buffer;
                } else {
                    assert!(final_audio.len() == buffer.len()); // might also be unneeded
                    for idx in 0..final_audio.len() {
                        let final_sample = final_audio[idx];
                        let buffer_sample = buffer[idx];
                        final_audio[idx] = (
                            (final_sample.0 + buffer_sample.0).clamp(-1.0, 1.0),
                            (final_sample.1 + buffer_sample.1).clamp(-1.0, 1.0),
                        )
                    }
                }
            }
            if final_audio.is_empty() {
                final_audio = vec![(0.0, 0.0); audio_streams[0].0.len()];
            }
            if client.audio_output_stream.is_none() {
                client.audio_output_stream = Some(self.voice_manager.create_stereo_encoder())
            }
            let tmp: Vec<f32> = final_audio.into_iter().flat_map(|x| [x.0, x.1]).collect();
            let output_buffer = self
                .voice_manager
                .encode_audio(client.audio_output_stream.unwrap(), &tmp);
            let mut buffer_bits: BitVec<u64, Lsb0> =
                BitVec::with_capacity(output_buffer.len() * BYTE);
            for byte in output_buffer {
                buffer_bits.extend(byte.view_bits::<Lsb0>());
            }
            outputs.push((buffer_bits, client.index));
        }
        for (buffer_bits, client) in outputs {
            networker.send(buffer_bits.as_bitslice(), 5, client);
        }
    }
}
#[godot_api]
impl INode for NetNodeServer {
    fn physics_process(&mut self, _delta: f64) {
        // cycle channel 1 packet buffers
        for client in self.server_networker.clients.iter_mut() {
            client.1.packet_buffers.pop_front();
            client.1.packet_buffers.push_back(Vec::new());
        }
        for client in self.server_networker.clients.values_mut() {
            let priorities = client.priorities.iter_mut();
            for priority in priorities {
                if priority.0.bind().owner_id != client.id {
                    let p = priority.0.bind().get_priority(client.id);
                    priority.1 += p;
                }
            }
        }
        self.tick_server();
        self.update_network_nodes();
        self.process_voice_input();
        self.send_packets_server();
    }
}

struct ServerNetworker {
    server: Server<NetcodeSocket>,
    start_time: Instant,
    packet_buffer: Vec<(BitVec<u64, Lsb0>, ClientIndex)>,
    next_client: u64,
    next_client_id: u16,
    clients: HashMap<ClientIndex, Client>,
}
impl Default for ServerNetworker {
    fn default() -> Self {
        ServerNetworker {
            server: Server::new(
                "127.0.0.1:0",
                build_time_utc!().as_bytes().iter().map(|x| *x as u64).sum(),
                netcode::generate_key(),
            )
            .unwrap(),
            start_time: Instant::now(),
            packet_buffer: Vec::new(),
            next_client: 0,
            next_client_id: 0,
            clients: HashMap::new(),
        }
    }
}
impl ServerNetworker {
    fn get_token(&mut self) -> ConnectToken {
        const TOKEN_EXPIREY_TIME: i32 = -1;
        const TOKEN_TIMEOUT_THRESHOLD: i32 = 30;
        self.next_client += 1;
        self.server
            .token(self.next_client)
            .expire_seconds(TOKEN_EXPIREY_TIME)
            .timeout_seconds(TOKEN_TIMEOUT_THRESHOLD)
            .generate()
            .unwrap()
    }
    fn send(&mut self, packet: &BitSlice<u64>, channel: u16, client_index: ClientIndex) {
        const PACKET_SPLIT_THRESHOLD: usize = 4800;
        let client = self.clients.get_mut(&client_index).unwrap();
        let reliable: bool;
        let mut packet_number: Option<u64> = None;
        match channel {
            1 => {
                reliable = false;
                packet_number = Some(client.packet_number_c1);
                client.packet_number_c1 += 1;
            }
            2 => {
                reliable = true;
                packet_number = Some(client.packet_number_c2);
                client.packet_number_c2 += 1;
            }
            3 => {
                reliable = true;
                packet_number = Some(client.packet_number_c3);
                client.packet_number_c3 += 1;
            }
            4 => {
                reliable = true;
                packet_number = Some(client.packet_number_c4);
                client.packet_number_c4 += 1;
            }
            5 => {
                reliable = false;
                packet_number = Some(client.packet_number_c5);
                client.packet_number_c5 += 1;
            }
            CHANNEL_CLIENT_ID => {
                reliable = true;
                packet_number = Some(0);
            }
            u16::MAX => reliable = false,
            _ => {
                godot_warn!("unhandled / invalid channel sent");
                reliable = false;
            }
        }
        if packet.len() + BYTES2 + BYTES8 > PACKET_SPLIT_THRESHOLD {
            let mut final_packet: BitVec<u64, Lsb0> =
                BitVec::with_capacity(packet.len() + BYTES2 + BYTES8);
            final_packet.extend(channel.view_bits::<Lsb0>());
            if packet_number.is_some() {
                final_packet.extend(packet_number.unwrap().view_bits::<Lsb0>());
            }
            final_packet.extend(packet);
            self.split_send(final_packet.as_bitslice(), client_index);
            return;
        }
        let mut final_packet: Vec<u8> = Vec::with_capacity(10 + packet.len());
        final_packet.extend(channel.to_le_bytes().iter());
        if packet_number.is_some() {
            final_packet.extend(packet_number.unwrap().to_le_bytes());
        }
        for bits in packet.chunks(BYTE) {
            final_packet.push(bits.load_le::<u8>());
        }
        self.server.send(&final_packet, client_index).unwrap();
        if packet_number.is_some() && reliable {
            client.reliable_packets.insert(
                (channel, packet_number.unwrap()),
                (final_packet, Instant::now()),
            );
        }
    }
    fn split_send(&mut self, packet: &BitSlice<u64>, client_index: ClientIndex) {
        const PACKET_SPLIT_THRESHOLD: usize = 4800 - (BYTES2 + BYTES8);
        let packet_chunks: Vec<&BitSlice<u64>> = packet.chunks(PACKET_SPLIT_THRESHOLD).collect();
        self.send(
            BitVec::<u64>::from_slice(&[packet_chunks.len() as u64]).as_bitslice(),
            4,
            client_index,
        );
        for chunk in packet_chunks {
            self.send(chunk, 4, client_index);
        }
    }
    fn poll(&mut self) -> Vec<u16> {
        self.server.update(self.start_time.elapsed().as_secs_f64());
        let mut new_players: Vec<u16> = Vec::new();
        while let Some(packet) = self.server.recv() {
            if self.clients.contains_key(&packet.1) {
                self.clients
                    .get_mut(&packet.1)
                    .unwrap()
                    .last_packet_send_time = Instant::now();
            } else {
                godot_warn!("new player packet");
                self.next_client_id += 1;
                self.clients.insert(
                    packet.1,
                    Client {
                        index: packet.1,
                        finished_sync: false,
                        remaining_bandwidth: 0,
                        packet_number_c1: 0,
                        packet_number_c2: 0,
                        packet_number_c3: 0,
                        packet_number_c4: 0,
                        packet_number_c5: 0,
                        player_position_object: None,
                        voice_input_stream: None,
                        audio_output_stream: None,
                        voice_packet_buffer: Vec::new(),
                        audio_input_buffer: vec![0.0; FRAME_LENGTH],
                        c4_remaining_packet_chunks: 0,
                        c4_packet_chunks: Vec::new(),
                        c4_waiting_packets: Vec::new(),
                        sync_progress: 0,
                        last_packet_send_time: Instant::now(),
                        reliable_packets: HashMap::new(),
                        latency: Duration::default(),
                        latency_buffer: VecDeque::new(),
                        waiting_acks: HashSet::new(),
                        id: self.next_client_id,
                        id_received: false,
                        message_buffer_position: 0,
                        priorities: Vec::new(),
                        c1_latency_info: LatencyInfo::default(),
                        next_c3_packet_number: 0,
                        next_c4_packet_number: 0,
                        next_c5_packet_number: 0,
                        c3_buffered_packets: Vec::new(),
                        packet_buffers: VecDeque::from_iter([Vec::new(), Vec::new()]),
                    },
                );
                new_players.push(self.next_client_id);
            }
            let client = self.clients.get_mut(&packet.1).unwrap();
            let channel: u16 = u16::from_le_bytes([packet.0[0], packet.0[1]]);
            if channel == CHANNEL_ACK {
                let mut pointer: usize = PACKET_HEADER_SIZE_ACK / BYTE;
                while pointer + ((BYTES8 + BYTES2) / BYTE) <= packet.0.len() {
                    let packet_channel =
                        u16::from_le_bytes([packet.0[pointer], packet.0[pointer + 1]]);
                    pointer += BYTES2 / BYTE;
                    let packet_num = u64::from_le_bytes([
                        packet.0[pointer],
                        packet.0[pointer + 1],
                        packet.0[pointer + 2],
                        packet.0[pointer + 3],
                        packet.0[pointer + 4],
                        packet.0[pointer + 5],
                        packet.0[pointer + 6],
                        packet.0[pointer + 7],
                    ]);
                    pointer += 8;
                    client
                        .reliable_packets
                        .remove(&(packet_channel, packet_num));
                }
                continue;
            }
            let packet_number: u64 = u64::from_le_bytes([
                packet.0[2],
                packet.0[3],
                packet.0[4],
                packet.0[5],
                packet.0[6],
                packet.0[7],
                packet.0[8],
                packet.0[9],
            ]);
            client.waiting_acks.insert((channel, packet_number));
            if channel == 4 {
                if packet_number == client.next_c4_packet_number {
                    client.next_c4_packet_number += 1;
                    if client.c4_remaining_packet_chunks == 0 {
                        client.c4_remaining_packet_chunks = u64::from_le_bytes([
                            packet.0[10],
                            packet.0[11],
                            packet.0[12],
                            packet.0[13],
                            packet.0[14],
                            packet.0[15],
                            packet.0[16],
                            packet.0[17],
                        ]);
                    } else {
                        client.c4_packet_chunks.push(packet.0);
                        client.c4_remaining_packet_chunks -= 1;
                        if client.c4_remaining_packet_chunks == 0 {
                            let mut packet: Vec<u8> = Vec::with_capacity(
                                client.c4_packet_chunks.iter().map(|x| x.len()).sum(),
                            );
                            for chunk in client.c4_packet_chunks.iter() {
                                packet.extend(chunk[10..].iter());
                            }
                            client.c4_packet_chunks.clear();
                            let channel: u16 = u16::from_le_bytes([packet[0], packet[1]]);
                            if channel == CHANNEL_ACK {
                                let mut pointer: usize = PACKET_HEADER_SIZE_ACK / BYTE;
                                while pointer + ((BYTES8 + BYTES2) / BYTE) <= packet.len() {
                                    let packet_channel =
                                        u16::from_le_bytes([packet[pointer], packet[pointer + 1]]);
                                    pointer += BYTES2 / BYTE;
                                    let packet_num = u64::from_le_bytes([
                                        packet[pointer],
                                        packet[pointer + 1],
                                        packet[pointer + 2],
                                        packet[pointer + 3],
                                        packet[pointer + 4],
                                        packet[pointer + 5],
                                        packet[pointer + 6],
                                        packet[pointer + 7],
                                    ]);
                                    pointer += 8;
                                    client
                                        .reliable_packets
                                        .remove(&(packet_channel, packet_num));
                                }
                                continue;
                            }
                            let mut packet_bits: BitVec<u64, Lsb0> =
                                BitVec::with_capacity(packet.len() * 8);
                            for byte in packet {
                                packet_bits.extend(byte.view_bits::<Lsb0>());
                            }
                            self.packet_buffer.push((packet_bits, client.index));
                        }
                    }
                } else {
                    client.c4_waiting_packets.push(packet.0);
                }
                let mut idx: usize = 0;
                loop {
                    let packet = client.c4_waiting_packets.get(idx);
                    if packet.is_none() {
                        break;
                    }
                    let packet = packet.unwrap();
                    let packet_number: u64 = u64::from_le_bytes([
                        packet[2], packet[3], packet[4], packet[5], packet[6], packet[7],
                        packet[8], packet[9],
                    ]);
                    if packet_number == client.next_c4_packet_number {
                        client.next_c4_packet_number += 1;
                        idx = 0;
                        if client.c4_remaining_packet_chunks == 0 {
                            client.c4_remaining_packet_chunks = u64::from_le_bytes([
                                packet[10], packet[11], packet[12], packet[13], packet[14],
                                packet[15], packet[16], packet[17],
                            ]);
                        } else {
                            client
                                .c4_packet_chunks
                                .push(client.c4_waiting_packets.swap_remove(idx));
                            client.c4_remaining_packet_chunks -= 1;
                            if client.c4_remaining_packet_chunks == 0 {
                                let mut packet: Vec<u8> = Vec::with_capacity(
                                    client.c4_packet_chunks.iter().map(|x| x.len()).sum(),
                                );
                                for chunk in client.c4_packet_chunks.iter() {
                                    packet.extend(chunk[10..].iter());
                                }
                                client.c4_packet_chunks.clear();
                                let channel: u16 = u16::from_le_bytes([packet[0], packet[1]]);
                                if channel == CHANNEL_ACK {
                                    let mut pointer: usize = PACKET_HEADER_SIZE_ACK / BYTE;
                                    while pointer + ((BYTES8 + BYTES2) / BYTE) <= packet.len() {
                                        let packet_channel = u16::from_le_bytes([
                                            packet[pointer],
                                            packet[pointer + 1],
                                        ]);
                                        pointer += BYTES2 / BYTE;
                                        let packet_num = u64::from_le_bytes([
                                            packet[pointer],
                                            packet[pointer + 1],
                                            packet[pointer + 2],
                                            packet[pointer + 3],
                                            packet[pointer + 4],
                                            packet[pointer + 5],
                                            packet[pointer + 6],
                                            packet[pointer + 7],
                                        ]);
                                        pointer += 8;
                                        client
                                            .reliable_packets
                                            .remove(&(packet_channel, packet_num));
                                    }
                                    continue;
                                }
                                let mut packet_bits: BitVec<u64, Lsb0> =
                                    BitVec::with_capacity(packet.len() * 8);
                                for byte in packet {
                                    packet_bits.extend(byte.view_bits::<Lsb0>());
                                }
                                self.packet_buffer.push((packet_bits, client.index));
                            }
                        }
                    } else {
                        idx += 1;
                    }
                }
                continue;
            }

            let mut packet_bits: BitVec<u64, Lsb0> = BitVec::with_capacity(packet.0.len() * 8);
            for byte in packet.0 {
                packet_bits.extend(byte.view_bits::<Lsb0>());
            }
            self.packet_buffer.push((packet_bits, packet.1));
        }
        let now = Instant::now();
        for client in self.clients.iter_mut() {
            for packet in client.1.reliable_packets.iter_mut() {
                if now - packet.1.1 > (client.1.latency + Duration::from_millis(32)) * 3 {
                    ServerNetworker::resend(&mut self.server, &packet.1.0, client.0.to_owned());
                    packet.1.1 = Instant::now();
                }
            }
        }
        new_players
    }
    fn resend(server: &mut Server<NetcodeSocket, ()>, final_packet: &[u8], client: ClientIndex) {
        server.send(final_packet, client).unwrap();
    }
}

struct Client {
    index: ClientIndex,
    finished_sync: bool,
    remaining_bandwidth: usize,
    packet_number_c1: u64,
    packet_number_c2: u64,
    packet_number_c3: u64,
    packet_number_c4: u64,
    packet_number_c5: u64,
    player_position_object: Option<Gd<Node3D>>,
    voice_input_stream: Option<usize>,
    audio_output_stream: Option<usize>,
    voice_packet_buffer: Vec<(u64, Vec<u8>)>,
    audio_input_buffer: Vec<f32>,
    c4_remaining_packet_chunks: u64,
    c4_packet_chunks: Vec<Vec<u8>>,
    c4_waiting_packets: Vec<Vec<u8>>,
    sync_progress: u64,
    last_packet_send_time: Instant,
    reliable_packets: HashMap<(u16, u64), (Vec<u8>, Instant)>,
    latency: Duration,
    latency_buffer: VecDeque<Duration>,
    waiting_acks: HashSet<(u16, u64)>,
    id: u16,
    id_received: bool,
    message_buffer_position: usize,
    priorities: Vec<(Gd<NetworkedNode>, i64)>,
    c1_latency_info: LatencyInfo,
    next_c3_packet_number: u64,
    next_c4_packet_number: u64,
    next_c5_packet_number: u64,
    c3_buffered_packets: Vec<BitVec<u64, Lsb0>>,
    packet_buffers: VecDeque<Vec<(BitVec<u64, Lsb0>, ClientIndex)>>,
}
#[derive(Debug, Default, Clone)]
struct LatencyInfo {
    c1_miss_rate_average_percent: f32,
    c1_miss_rate_average: f32,
    c1_hit_rate_average: f32,
    c1_miss_rate_last_frames: VecDeque<u64>,
    c1_hit_rate_last_frames: VecDeque<u64>,
}
