// functionallity for the NetNodeManager server
use crate::messages::*;
use crate::net_nodes::NetworkedNode;
use crate::serializer::*;
use bitvec::prelude::*;
use build_time::build_time_utc;
use std::cell::Cell;
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
    server_networker: ServerNetworker,
    message_buffer: VecDeque<Box<dyn Message>>,
    incoming_message_buffer: VecDeque<Box<dyn Message>>,
    base: Base<Node>,
}

#[godot_api]
pub impl NetNodeServer {
    pub fn register_node(&mut self, new_node_ref: Gd<NetworkedNode>, new_node: &mut NetworkedNode) {
        self.next_id += 1;
        new_node.objectid = self.next_id;
        let message: NetworkObjectCreation = NetworkObjectCreation {
            node_type: new_node.object_type,
            owner_id: new_node.owner_id,
            object_id: new_node.objectid,
            node_path: new_node_ref.get_path().to_string(),
        };
        self.message_buffer.push_back(Box::new(message));
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
    pub fn has_message(&self) -> bool {
        self.incoming_message_buffer.front().is_some()
    }
    pub fn peek_message(&self) -> &dyn Message {
        self.incoming_message_buffer[0].as_ref()
    }
    pub fn pop_message(&mut self) {
        self.incoming_message_buffer.pop_front();
    }
    pub fn get_message_type(&self) -> u8 {
        self.incoming_message_buffer[0].get_message_type()
    }
    pub fn get_message_player(&self) -> u16 {
        self.incoming_message_buffer[0].get_player()
    }
    pub fn change_object_ownership(&mut self, objectid: u16, player: u16) {
        let message: ChangeObjectOwnership = ChangeObjectOwnership {
            objectid: objectid,
            player: player,
        };
        self.message_buffer.push_back(Box::new(message));
        let message: ChangeObjectOwnership = ChangeObjectOwnership {
            objectid: objectid,
            player: player,
        };
        self.incoming_message_buffer.push_back(Box::new(message));
    }
    pub fn trigger_interaction(
        &mut self,
        player: u16,
        interaction_type: u8,
        interactable_path: String,
    ) {
        let message = PlayerInteract {
            player: player,
            interaction_type: interaction_type,
            interactable_path: interactable_path,
        };
        self.message_buffer.push_back(Box::new(message.clone()));
        self.incoming_message_buffer.push_back(Box::new(message));
    }
    fn update_network_nodes(&mut self) {
        for client in self.server_networker.clients.iter() {
            for packet_tuple in client
                .1
                .packet_buffers
                .first()
                .or(Some(&Cell::new(Vec::new())))
                .map(|x| x.take())
                .unwrap()
            {
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
            let mut remaining_bandwidth = bandwidth_per_tick;
            'outer: while remaining_bandwidth > PACKET_MAX_SIZE_THRESHOLD {
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
                    remaining_bandwidth = remaining_bandwidth.saturating_sub(packet.len());
                    buffer.push((*client.0, packet, CHANNEL_ACK));
                }
                // channel 3 (messages)
                while self.message_buffer.len() > client.1.message_buffer_position {
                    let mut packet: BitVec<u64> =
                        BitVec::with_capacity(MAX_SINGLE_PACKET_PAYLOAD_LENGTH);
                    let message: &Box<dyn Message> =
                        &self.message_buffer[client.1.message_buffer_position];
                    packet.extend(message.get_message_type().view_bits::<Lsb0>());
                    packet.extend(message.encode_message());
                    if (remaining_bandwidth as i64 - packet.len() as i64) < 0 {
                        break 'outer;
                    }
                    remaining_bandwidth -= packet.len();
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
                            > cmp::min(remaining_bandwidth, MAX_SINGLE_PACKET_PAYLOAD_LENGTH)
                        {
                            break;
                        }
                        client.1.sync_progress = index;
                        packet.extend(tmp);
                    }

                    if packet.len() > CHANNEL1_HEADER_SIZE {
                        remaining_bandwidth -= packet.len();
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
                            > cmp::min(remaining_bandwidth, MAX_SINGLE_PACKET_PAYLOAD_LENGTH)
                        {
                            break;
                        }
                        packet.extend(tmp);
                        value.1 = 0;
                    }
                }
                if packet.len() > CHANNEL1_HEADER_SIZE {
                    remaining_bandwidth -= packet.len();
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
                self.incoming_message_buffer
                    .push_back(Box::new(PlayerJoin { player: player }));
            }
        }
        let networker = &mut self.server_networker;
        // check for and handle disconnected clients
        loop {
            let mut dc_client: Option<ClientIndex> = None;
            for client in networker.clients.iter() {
                if networker.server.client_id(*client.0).is_none() {
                    self.message_buffer.push_back(Box::new(PlayerDc {
                        player: client.1.id,
                    }));
                    for node in self.networked_nodes.iter_mut() {
                        if node.bind().owner_id == client.1.id {
                            node.bind_mut().on_owner_dc();
                            node.bind_mut().owner_id = 0;
                        }
                    }
                    dc_client = Some(client.0.clone());
                    break;
                }
            }
            if dc_client.is_some() {
                networker.clients.remove(&dc_client.unwrap());
            } else {
                break;
            }
        }
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
                            buffer.get_mut().push(packet_tuple);
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
                        let message_type: u8 = packet[pointer..pointer + BYTE].load_le();
                        pointer += BYTE;
                        let mut message: Box<dyn Message>;
                        match message_type {
                            0 => message = Box::new(NetworkObjectCreation::default()),
                            1 => message = Box::new(PlayerPhysicsGrab::default()),
                            2 => message = Box::new(PlayerPhysicsRelease::default()),
                            3 => message = Box::new(ChatBoxMessageSent::default()),
                            5 => message = Box::new(PlayerAvatarChange::default()),
                            7 => message = Box::new(ChangeObjectOwnership::default()),
                            8 => message = Box::new(PlayerInteract::default()),
                            _ => panic!("got unhandled message type {:#?}", message_type),
                        }
                        message.decode_message(&mut pointer, packet.as_bitslice());
                        self.message_buffer
                            .push_back(dyn_clone::clone_box(&*message));
                        self.incoming_message_buffer.push_back(message);
                        // since the next packet might of been buffered to wait for this one, we go through the buffer and check each packet
                        // easy optimisation would be storing the packet number and keeping the vec sorted so we only check once
                        let mut index: usize = 0;
                        loop {
                            let packet = client.c3_buffered_packets.get(index);
                            if packet.is_none() {
                                break;
                            }
                            let packet = packet.unwrap();
                            if packet.len() < PACKET_HEADER_SIZE + BYTES8 {
                                break;
                            }
                            let mut pointer: usize = PACKET_HEADER_SIZE;
                            let packet_number: u64 = packet[pointer..pointer + BYTES8].load_le();
                            pointer += BYTES8;
                            if packet_number == client.next_c3_packet_number {
                                client.next_c3_packet_number += 1;
                                let message_type: u8 = packet[pointer..pointer + BYTE].load_le();
                                pointer += BYTE;
                                let mut message: Box<dyn Message>;
                                match message_type {
                                    0 => message = Box::new(NetworkObjectCreation::default()),
                                    1 => message = Box::new(PlayerPhysicsGrab::default()),
                                    2 => message = Box::new(PlayerPhysicsRelease::default()),
                                    3 => message = Box::new(ChatBoxMessageSent::default()),
                                    5 => message = Box::new(PlayerAvatarChange::default()),
                                    7 => message = Box::new(ChangeObjectOwnership::default()),
                                    8 => message = Box::new(PlayerInteract::default()),
                                    _ => panic!("got unhandled message type {:#?}", message_type),
                                }
                                message.decode_message(&mut pointer, packet.as_bitslice());
                                self.message_buffer
                                    .push_back(dyn_clone::clone_box(&*message));
                                self.incoming_message_buffer.push_back(message);
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
                client.packet_buffers.push(Cell::new(Vec::new()));
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
}
#[godot_api]
impl INode for NetNodeServer {
    fn physics_process(&mut self, _delta: f64) {
        // cycle channel 1 packet buffers
        for client in self.server_networker.clients.iter() {
            for x in client.1.packet_buffers.windows(2) {
                x[0].set(x[1].take());
            }
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
        self.send_packets_server();

        // handle ownership events
        if self.has_message() && self.get_message_type() == 7 {
            let objectid: u16 = self
                .peek_message()
                .as_any()
                .downcast_ref::<ChangeObjectOwnership>()
                .unwrap()
                .objectid;
            self.networked_nodes
                .iter_mut()
                .find(|x| x.bind().objectid == objectid)
                .unwrap()
                .bind_mut()
                .owner_id = self.get_message_player();
            self.pop_message();
        }
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
                self.next_client_id += 1;
                self.clients.insert(
                    packet.1,
                    Client {
                        index: packet.1,
                        finished_sync: false,
                        packet_number_c1: 0,
                        packet_number_c2: 0,
                        packet_number_c3: 0,
                        packet_number_c4: 0,
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
                        c3_buffered_packets: Vec::new(),
                        packet_buffers: Vec::from_iter([
                            Cell::new(Vec::new()),
                            Cell::new(Vec::new()),
                        ]),
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
    packet_number_c1: u64,
    packet_number_c2: u64,
    packet_number_c3: u64,
    packet_number_c4: u64,
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
    c3_buffered_packets: Vec<BitVec<u64, Lsb0>>,
    packet_buffers: Vec<Cell<Vec<(BitVec<u64, Lsb0>, ClientIndex)>>>,
}
#[derive(Debug, Default, Clone)]
struct LatencyInfo {
    c1_miss_rate_average_percent: f32,
    c1_miss_rate_average: f32,
    c1_hit_rate_average: f32,
    c1_miss_rate_last_frames: VecDeque<u64>,
    c1_hit_rate_last_frames: VecDeque<u64>,
}
