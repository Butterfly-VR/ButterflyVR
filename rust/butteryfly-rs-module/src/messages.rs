use crate::{
    NetNodeManager,
    net_nodes::NetworkedNode,
    serializer::{self, NetworkedValueTypes},
};
use bitvec::prelude::*;
use godot::prelude::*;

const BYTE: usize = 8;
const BYTES2: usize = 16;

#[derive(GodotClass)]
#[class(init, base=Node)]
pub struct MessageHandler {
    #[export]
    pub message_type: u16,
    network_manager: Option<Gd<NetNodeManager>>,
    base: Base<Node>,
}
#[godot_api]
pub impl MessageHandler {
    #[func(virtual)]
    fn get_value_type(&mut self, _previous_value: Variant, _idx: i64) -> i64 {
        unimplemented!()
    }
    #[func(virtual)]
    fn process_message(&mut self, _values: VariantArray) {
        unimplemented!()
    }
    #[func]
    fn send_message_final(&mut self, values: VariantArray, types: Array<i64>) {
        if values.len() != types.len() {
            godot_warn!("invalid call to send_message_final");
            return;
        }
        if types
            .iter_shared()
            .any(|x| NetworkedValueTypes::try_from(x).is_err())
        {
            godot_warn!("invalid call to send_message_final");
            return;
        }

        let types = types
            .iter_shared()
            .map(|x| NetworkedValueTypes::try_from(x).unwrap())
            .collect::<Vec<NetworkedValueTypes>>();
        let mut packet: BitVec<u64, Lsb0> = BitVec::new();
        packet.extend(self.message_type.view_bits::<Lsb0>());
        for value in values.iter_shared().enumerate() {
            packet.extend(serializer::encode_with_known_type(
                &value.1,
                &types[value.0],
            ));
        }

        if self
            .network_manager
            .as_mut()
            .unwrap()
            .bind_mut()
            .is_server()
        {
            godot_warn!("handling on server");
            self.handle_message(packet.clone().as_bitslice(), &mut BYTES2.clone());
        }
        self.network_manager
            .as_mut()
            .unwrap()
            .bind_mut()
            .queue_message(packet);
    }
    pub fn handle_message(&mut self, packet: &BitSlice<u64, Lsb0>, pointer: &mut usize) {
        let mut idx = 0;
        let mut last_value = Variant::nil();
        let mut values: VariantArray = VariantArray::new();
        while *pointer < packet.len() {
            let value_type =
                &NetworkedValueTypes::try_from(self.get_value_type(last_value, idx)).unwrap();
            last_value = serializer::decode_with_known_type(packet, pointer, value_type).unwrap();
            values.push(&last_value);
            idx += 1;
        }
        self.apply_deferred(|this| this.process_message(values));
    }
    pub fn create_id_sync_message(
        object: Gd<Node>,
        object_id: u16,
        owner_id: Option<u16>,
    ) -> BitVec<u64, Lsb0> {
        let mut packet: BitVec<u64, Lsb0> = BitVec::new();
        packet.extend(0u16.view_bits::<Lsb0>());
        packet.extend(object_id.view_bits::<Lsb0>());
        packet.extend(owner_id.unwrap_or(0).view_bits::<Lsb0>());
        let mut index_path: Vec<u8> = Vec::with_capacity(8);
        index_path.push(object.get_index() as u8);
        let mut last_parent: Option<Gd<Node>>;
        last_parent = object.get_parent();
        loop {
            if let Some(parent) = last_parent {
                last_parent = parent.get_parent();
                index_path.push(parent.get_index() as u8);
            } else {
                break;
            }
        }
        index_path.pop();
        for item in index_path.iter().rev() {
            packet.extend(item.view_bits::<Lsb0>())
        }
        packet
    }
    pub fn handle_id_sync_message(
        message: &BitSlice<u64, Lsb0>,
        pointer: &mut usize,
        root_object: Gd<Node>,
    ) {
        let mut object = Some(root_object);
        let id: u16 = message[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
        let owner_id: u16 = message[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
        while let Some(index) = message.get(*pointer..*pointer + BYTE) {
            let index: u8 = index.load_le();
            object = object.unwrap().get_child(index as i32);
            if object.is_none() {
                godot_warn!("failed to apply id to object");
                return;
            }
            *pointer += BYTE;
        }
        let mut object = object.unwrap();
        let casted_object = object.try_cast::<NetworkedNode>();
        if let Ok(mut object) = casted_object {
            object.bind_mut().objectid = id;
            object.bind_mut().owner_id = owner_id;
        } else {
            object = casted_object.unwrap_err();
            let casted_object = object.try_cast::<MessageHandler>();
            if let Ok(mut object) = casted_object {
                object.bind_mut().message_type = id;
            }
        }
    }
}
#[godot_api]
impl INode for MessageHandler {
    fn enter_tree(&mut self) {
        self.network_manager = Some(
            self.base()
                .get_node_as::<NetNodeManager>("/root/NetworkManager"),
        );
        self.base()
            .get_node_as::<NetNodeManager>("/root/NetworkManager")
            .bind_mut()
            .register_message_handler(self.to_gd(), self.message_type);
    }
    fn exit_tree(&mut self) {
        self.network_manager
            .as_mut()
            .unwrap()
            .bind_mut()
            .unregister_message_handler(self.message_type);
    }
}
