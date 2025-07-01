// wrapper for either a client or server
mod client;
mod messages;
mod net_nodes;
mod serializer;
mod server;
mod voice;
use crate::client::*;
use crate::net_nodes::*;
use crate::server::*;

use godot::prelude::*;

struct MyExtension;

#[gdextension]
unsafe impl ExtensionLibrary for MyExtension {}

#[derive(GodotClass)]
#[class(init, base=Node)]
struct NetNodeManager {
    client: Option<Gd<NetNodeClient>>,
    server: Option<Gd<NetNodeServer>>,
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
        self.server
            .as_mut()
            .unwrap()
            .bind_mut()
            .start_server(bind_addr, private_key);
    }
    #[func]
    fn stop(&mut self) {
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
    fn network_grab(&mut self, target: Gd<Node>) {
        if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .network_grab(target);
        } else {
            godot_warn!("tried to network_grab but we are not a client")
        }
    }
    #[func]
    fn network_release(&mut self) {
        if self.client.is_some() {
            self.client.as_mut().unwrap().bind_mut().network_release();
        } else {
            godot_warn!("tried to network_grab but we are not a client")
        }
    }
    #[func]
    fn network_message_send(&mut self, message: String) {
        if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .network_message_send(message);
        } else {
            godot_warn!("tried to network_message_send but we are not a client")
        }
    }
    #[func]
    fn change_avatar(&mut self, avatar: u64) {
        if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .change_avatar(avatar);
        } else {
            godot_warn!("tried to change_avatar but we are not a client")
        }
    }
    #[func]
    fn change_object_owner(&mut self, objectid: u16, player: u16) {
        if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .change_object_ownership(objectid, player);
        } else if self.server.is_some() {
            self.server
                .as_mut()
                .unwrap()
                .bind_mut()
                .change_object_ownership(objectid, player);
        } else {
            godot_warn!("tried to change_object_owner but we are not a client or server")
        }
    }
    #[func]
    fn become_object_owner(&mut self, objectid: u16) {
        if self.client.is_some() {
            let id: u16 = self.client.as_ref().unwrap().bind().get_id();
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .change_object_ownership(objectid, id);
        } else if self.server.is_some() {
            let id: u16 = self.server.as_ref().unwrap().bind().get_id();
            self.server
                .as_mut()
                .unwrap()
                .bind_mut()
                .change_object_ownership(objectid, id);
        } else {
            godot_warn!("tried to become_object_owner but we are not a client or server")
        }
    }
    #[func]
    fn release_object_owner(&mut self, objectid: u16) {
        if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .change_object_ownership(objectid, 0);
        } else if self.server.is_some() {
            self.server
                .as_mut()
                .unwrap()
                .bind_mut()
                .change_object_ownership(objectid, 0);
        } else {
            godot_warn!("tried to release_object_owner but we are not a client or server")
        }
    }

    #[func]
    fn trigger_interaction(
        &mut self,
        player: u16,
        interaction_type: u8,
        interactable_path: String,
    ) {
        if self.client.is_some() {
            self.client
                .as_mut()
                .unwrap()
                .bind_mut()
                .trigger_interaction(player, interaction_type, interactable_path);
        } else if self.server.is_some() {
            self.server
                .as_mut()
                .unwrap()
                .bind_mut()
                .trigger_interaction(player, interaction_type, interactable_path);
        } else {
            godot_warn!("tried to trigger_interaction but we are not a client or server")
        }
    }
    #[func]
    fn peek_message(&mut self) -> VariantArray {
        if self.client.is_some() {
            let c = self.client.as_mut().unwrap().bind_mut();
            return c.peek_message().get_message_contents();
        } else if self.server.is_some() {
            let s = self.server.as_mut().unwrap().bind_mut();
            return s.peek_message().get_message_contents();
        } else {
            panic!("called peek_message but no client or server is running");
        }
    }
    #[func]
    fn pop_message(&mut self) {
        if self.client.is_some() {
            self.client.as_mut().unwrap().bind_mut().pop_message();
        } else if self.server.is_some() {
            self.server.as_mut().unwrap().bind_mut().pop_message();
        } else {
            godot_warn!("called pop_message but no client or server is running");
        }
    }
    #[func]
    fn has_message(&mut self) -> bool {
        if self.client.is_some() {
            return self.client.as_mut().unwrap().bind_mut().has_message();
        } else if self.server.is_some() {
            return self.server.as_mut().unwrap().bind_mut().has_message();
        } else {
            panic!("called peek_message but no client or server is running");
        }
    }
    #[func]
    fn get_message_type(&mut self) -> u8 {
        if self.client.is_some() {
            return self.client.as_mut().unwrap().bind_mut().get_message_type();
        } else if self.server.is_some() {
            return self.server.as_mut().unwrap().bind_mut().get_message_type();
        } else {
            panic!("called peek_message but no client or server is running");
        }
    }
    #[func]
    fn get_message_player(&mut self) -> u16 {
        if self.client.is_some() {
            return self
                .client
                .as_mut()
                .unwrap()
                .bind_mut()
                .get_message_player();
        } else if self.server.is_some() {
            return self
                .server
                .as_mut()
                .unwrap()
                .bind_mut()
                .get_message_player();
        } else {
            panic!("called peek_message but no client or server is running");
        }
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
}
