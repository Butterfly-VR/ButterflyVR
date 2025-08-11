// wrapper for either a client or server
mod client;
mod messages;
mod net_nodes;
mod serializer;
mod server;
mod voice;

use crate::client::*;
use crate::messages::MessageHandler;
use crate::net_nodes::*;
use crate::server::*;
use bitvec::prelude::*;
use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(init, base=Node)]
struct NetNodeManager {
    client: Option<Gd<NetNodeClient>>,
    server: Option<Gd<NetNodeServer>>,
    is_server: bool,
    base: Base<Node>,
}

#[godot_api]
impl NetNodeManager {
    fn register_node(&mut self, node_ref: Gd<NetworkedNode>, node: &mut NetworkedNode) {
        if self.server.is_some() {
            self.server
                .as_mut()
                .unwrap()
                .bind_mut()
                .register_node(node_ref, node);
        } else if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .register_node(node_ref, node);
        } else {
            godot_warn!("called register_node but no client or server is running");
        }
    }
    fn unregister_node(&mut self, node_ref: Gd<NetworkedNode>) {
        if self.server.is_some() {
            self.server
                .as_mut()
                .unwrap()
                .bind_mut()
                .unregister_node(node_ref);
        } else if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .unregister_node(node_ref);
        } // can get called when no client or server is active after client dc so we ignore that case here
    }
    #[func]
    fn unregister_all(&mut self) {
        if self.server.is_some() {
            self.server.as_mut().unwrap().bind_mut().unregister_all();
        } else if self.client.is_some() {
            self.client.as_mut().unwrap().bind_mut().unregister_all();
        } else {
            godot_warn!("called unregister_all but no client or server is running");
        }
    }
    #[func]
    fn get_next_object_id(&mut self) -> u16 {
        if self.server.is_some() {
            return self
                .server
                .as_mut()
                .unwrap()
                .bind_mut()
                .get_next_object_id();
        } else {
            panic!("called get_next_object_id but we are not a server");
        }
    }
    #[func]
    fn start_client(&mut self, arr: PackedByteArray) {
        let c = NetNodeClient::new_alloc();
        self.base_mut().add_child(&c);
        self.client = Some(c);
        self.client.as_mut().unwrap().bind_mut().start_client(arr);
    }
    #[func]
    fn start_server(&mut self, bind_addr: String, private_key: [u8; 32]) {
        let s = NetNodeServer::new_alloc();
        self.base_mut().add_child(&s);
        self.server = Some(s);
        let selfref = self.to_gd();
        self.server
            .as_mut()
            .unwrap()
            .signals()
            .player_joined()
            .connect_other(&selfref, NetNodeManager::propogate_player_joined);
        self.server
            .as_mut()
            .unwrap()
            .signals()
            .player_left()
            .connect_other(&selfref, NetNodeManager::propogate_player_left);
        self.server
            .as_mut()
            .unwrap()
            .bind_mut()
            .start_server(bind_addr, private_key);
        self.is_server = true;
    }
    #[func]
    fn stop(&mut self) {
        self.is_server = false;
        if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .disconnect()
                .unwrap();
            self.client.as_mut().unwrap().queue_free();
            self.client = None;
        } else if self.server.is_some() {
            todo!(
                "server does not have graceful stop functionality yet, ensure clients have disconnected then kill the server process"
            )
        }
    }
    #[func]
    fn get_next_client(&mut self) -> PackedByteArray {
        if self.server.is_some() {
            self.server.as_mut().unwrap().bind_mut().get_next_client()
        } else {
            panic!("called get_next_client() but we are not a server");
        }
    }
    #[func]
    fn get_id(&self) -> u16 {
        if self.server.is_some() {
            self.server.as_ref().unwrap().bind().id
        } else if self.client.is_some() {
            self.client.as_ref().unwrap().bind().id
        } else {
            panic!("called get_id but no client or server is running");
        }
    }
    #[func]
    fn id_ready(&self) -> bool {
        if self.client.is_none() && self.server.is_none() {
            return false;
        }
        if self.client.is_some()
            && self.client.as_ref().unwrap().bind().client_networker.state
                == ClientState::AwaitingID
        {
            return false;
        }
        return true;
    }
    #[func]
    pub fn is_server(&self) -> bool {
        self.is_server
    }
    #[func]
    fn get_networked_nodes(&self) -> Vec<Gd<NetworkedNode>> {
        if self.client.is_some() {
            return self.client.as_ref().unwrap().bind().networked_nodes.clone();
        } else if self.server.is_some() {
            return self.server.as_ref().unwrap().bind().networked_nodes.clone();
        } else {
            panic!("tried to get_networked_nodes but no client or server is running");
        }
    }
    #[func]
    fn transmit_audio(&mut self, sample_buffer: PackedVector2Array) {
        if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .transmit_audio(sample_buffer);
        } else {
            godot_warn!("tried to transmit_audio but we are not a client")
        }
    }
    #[func]
    fn get_audio(&mut self) -> Vec<f32> {
        if self.client.is_some() {
            self.client.as_mut().unwrap().bind_mut().get_audio()
        } else {
            panic!("tried to get_audio but we are not a client")
        }
    }
    #[func]
    fn register_player_object(&mut self, player: u16, object: Gd<Node3D>) {
        if self.server.is_some() {
            return self
                .server
                .as_mut()
                .unwrap()
                .bind_mut()
                .register_player_object(player, object);
        } else {
            panic!("tried to register_player_object but we are not a server");
        }
    }
    fn register_message_handler(&mut self, handler: Gd<MessageHandler>, message_type: u16) {
        if self.client.is_some() {
            return self
                .client
                .as_mut()
                .unwrap()
                .bind_mut()
                .register_message(handler, message_type);
        } else if self.server.is_some() {
            return self
                .server
                .as_mut()
                .unwrap()
                .bind_mut()
                .register_message(handler, message_type);
        } else {
            panic!("tried to register_message_handler but no client or server is running");
        }
    }
    fn unregister_message_handler(&mut self, message_type: u16) {
        if self.client.is_some() {
            return self
                .client
                .as_mut()
                .unwrap()
                .bind_mut()
                .unregister_message(message_type);
        } else if self.server.is_some() {
            return self
                .server
                .as_mut()
                .unwrap()
                .bind_mut()
                .unregister_message(message_type);
        } else {
            panic!("tried to unregister_message_handler but no client or server is running");
        }
    }
    fn queue_message(&mut self, message: BitVec<u64, Lsb0>) {
        if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .queue_message(message);
        } else if self.server.is_some() {
            self.server
                .as_mut()
                .unwrap()
                .bind_mut()
                .queue_message(message);
        } else {
            godot_warn!("tried to queue_message but no client or server is running");
        }
    }
    fn propogate_player_joined(&mut self, player: u16) {
        godot_warn!("new player propogated");
        self.signals().player_joined().emit(player);
    }
    fn propogate_player_left(&mut self, player: u16) {
        self.signals().player_left().emit(player);
    }
    #[signal]
    pub fn player_joined(player: u16);
    #[signal]
    pub fn player_left(player: u16);
}
