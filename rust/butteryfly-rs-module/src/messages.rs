use bitvec::prelude::*;
use dyn_clone::DynClone;
use godot::prelude::*;
use std::any::Any;

const BYTE: usize = 8;
const BYTES2: usize = 16;
const BYTES8: usize = 64;

pub trait Message: DynClone {
    fn encode_message(&self) -> BitVec;
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>);
    fn get_message_type(&self) -> u8;
    fn get_message_contents(&self) -> VariantArray;
    fn get_player(&self) -> u16;
    fn as_any(&self) -> &dyn Any;
}

#[derive(Debug, Default, Clone)]
pub struct NetworkObjectCreation {
    pub node_type: u8,
    pub owner_id: u16,
    pub object_id: u16,
    pub node_path: String,
}
impl Message for NetworkObjectCreation {
    fn encode_message(&self) -> BitVec {
        let mut packet: BitVec = BitVec::with_capacity(1000); // temporary, replace when we have size hints
        packet.extend(self.node_type.view_bits::<Lsb0>());
        packet.extend(self.owner_id.view_bits::<Lsb0>());
        packet.extend(self.object_id.view_bits::<Lsb0>());
        for byte in self.node_path.as_bytes() {
            packet.extend(byte.view_bits::<Lsb0>());
        }
        return packet;
    }
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>) {
        self.node_type = data[*pointer..*pointer + BYTE].load_le();
        *pointer += BYTE;
        self.owner_id = data[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
        self.object_id = data[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
        let mut bytes: Vec<u8> = Vec::with_capacity((data[*pointer..].len() / BYTE) + 1);
        for bits in data[*pointer..].chunks(BYTE) {
            bytes.push(bits.load_le())
        }
        self.node_path = String::from_utf8_lossy(&bytes).to_string();
    }

    fn get_message_type(&self) -> u8 {
        return 0;
    }
    fn get_message_contents(&self) -> VariantArray {
        return Array::from_iter(
            [
                self.node_type.to_variant(),
                self.owner_id.to_variant(),
                self.object_id.to_variant(),
                self.node_path.to_variant(),
            ]
            .into_iter(),
        );
    }
    fn get_player(&self) -> u16 {
        return 0;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct PlayerPhysicsGrab {
    pub player: u16,
    pub target: String,
}
impl Message for PlayerPhysicsGrab {
    fn encode_message(&self) -> BitVec {
        let mut packet: BitVec = BitVec::with_capacity(1000); // temporary, replace when we have size hints
        packet.extend(self.player.view_bits::<Lsb0>());
        for byte in self.target.as_bytes() {
            packet.extend(byte.view_bits::<Lsb0>());
        }
        return packet;
    }
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>) {
        self.player = data[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
        let mut bytes: Vec<u8> = Vec::with_capacity((data[*pointer..].len() / BYTE) + 1);
        for bits in data[*pointer..].chunks(BYTE) {
            bytes.push(bits.load_le())
        }
        self.target = String::from_utf8_lossy(&bytes).to_string();
    }

    fn get_message_type(&self) -> u8 {
        return 1;
    }
    fn get_message_contents(&self) -> VariantArray {
        return Array::from_iter([self.target.to_variant()].into_iter());
    }
    fn get_player(&self) -> u16 {
        return self.player;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[derive(Debug, Default, Clone)]
pub struct PlayerPhysicsRelease {
    pub player: u16,
}
impl Message for PlayerPhysicsRelease {
    fn encode_message(&self) -> BitVec {
        let mut packet: BitVec = BitVec::with_capacity(1000); // temporary, replace when we have size hints
        packet.extend(self.player.view_bits::<Lsb0>());
        return packet;
    }
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>) {
        self.player = data[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
    }

    fn get_message_type(&self) -> u8 {
        return 2;
    }
    fn get_message_contents(&self) -> VariantArray {
        return Array::from_iter([].into_iter());
    }
    fn get_player(&self) -> u16 {
        return self.player;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct ChatBoxMessageSent {
    pub player: u16,
    pub message: String,
}
impl Message for ChatBoxMessageSent {
    fn encode_message(&self) -> BitVec {
        let mut packet: BitVec = BitVec::with_capacity(1000); // temporary, replace when we have size hints
        packet.extend(self.player.view_bits::<Lsb0>());
        for byte in self.message.as_bytes() {
            packet.extend(byte.view_bits::<Lsb0>());
        }
        return packet;
    }
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>) {
        self.player = data[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
        let mut bytes: Vec<u8> = Vec::with_capacity((data[*pointer..].len() / BYTE) + 1);
        for bits in data[*pointer..].chunks(BYTE) {
            bytes.push(bits.load_le())
        }
        self.message = String::from_utf8_lossy(&bytes).to_string();
    }
    fn get_message_type(&self) -> u8 {
        return 3;
    }

    fn get_message_contents(&self) -> VariantArray {
        return Array::from_iter([self.message.to_variant()].into_iter());
    }
    fn get_player(&self) -> u16 {
        return self.player;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[derive(Debug, Default, Clone)]
pub struct PlayerDc {
    pub player: u16,
}
impl Message for PlayerDc {
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>) {
        self.player = data[*pointer..*pointer + BYTES2].load_le();
    }
    fn encode_message(&self) -> BitVec {
        let mut packet: BitVec = BitVec::with_capacity(BYTES2); // temporary, replace when we have size hints
        packet.extend(self.player.view_bits::<Lsb0>());
        return packet;
    }
    fn get_message_type(&self) -> u8 {
        return 4;
    }

    fn get_message_contents(&self) -> VariantArray {
        return Array::from_iter([].into_iter());
    }
    fn get_player(&self) -> u16 {
        return self.player;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[derive(Debug, Default, Clone)]
pub struct PlayerAvatarChange {
    pub player: u16,
    pub avatar: u64,
}
impl Message for PlayerAvatarChange {
    fn encode_message(&self) -> BitVec {
        let mut packet: BitVec = BitVec::with_capacity(1000); // temporary, replace when we have size hints
        packet.extend(self.player.view_bits::<Lsb0>());
        packet.extend(self.avatar.view_bits::<Lsb0>());
        return packet;
    }
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>) {
        self.player = data[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
        self.avatar = data[*pointer..*pointer + BYTES8].load_le();
        *pointer += BYTES8;
    }

    fn get_message_type(&self) -> u8 {
        return 5;
    }
    fn get_message_contents(&self) -> VariantArray {
        return Array::from_iter([self.avatar.to_variant()].into_iter());
    }
    fn get_player(&self) -> u16 {
        return self.player;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[derive(Debug, Default, Clone)]
pub struct ChangeObjectOwnership {
    pub objectid: u16,
    pub player: u16,
}
impl Message for ChangeObjectOwnership {
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>) {
        self.player = data[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
        self.objectid = data[*pointer..*pointer + BYTES2].load_le();
    }
    fn encode_message(&self) -> BitVec {
        let mut packet: BitVec = BitVec::with_capacity(BYTES8);
        packet.extend(self.player.view_bits::<Lsb0>());
        packet.extend(self.objectid.view_bits::<Lsb0>());
        return packet;
    }
    fn get_message_type(&self) -> u8 {
        return 7;
    }

    fn get_message_contents(&self) -> VariantArray {
        return Array::from_iter([].into_iter());
    }
    fn get_player(&self) -> u16 {
        return self.player;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[derive(Debug, Default, Clone)]
pub struct PlayerInteract {
    pub interaction_type: u8,
    pub player: u16,
    pub interactable_path: String,
}
impl Message for PlayerInteract {
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>) {
        self.player = data[*pointer..*pointer + BYTES2].load_le();
        *pointer += BYTES2;
        self.interaction_type = data[*pointer..*pointer + BYTE].load_le();
        *pointer += BYTE;
        let mut bytes: Vec<u8> = Vec::with_capacity((data[*pointer..].len() / BYTE) + 1);
        for bits in data[*pointer..].chunks(BYTE) {
            bytes.push(bits.load_le())
        }
        self.interactable_path = String::from_utf8_lossy(&bytes).to_string();
    }
    fn encode_message(&self) -> BitVec {
        let mut packet: BitVec = BitVec::with_capacity(BYTES8);
        packet.extend(self.player.view_bits::<Lsb0>());
        packet.extend(self.interaction_type.view_bits::<Lsb0>());
        for byte in self.interactable_path.as_bytes() {
            packet.extend(byte.view_bits::<Lsb0>());
        }
        return packet;
    }
    fn get_message_type(&self) -> u8 {
        return 8;
    }

    fn get_message_contents(&self) -> VariantArray {
        return Array::from_iter(
            [
                self.interaction_type.to_variant(),
                self.interactable_path.to_variant(),
            ]
            .into_iter(),
        );
    }
    fn get_player(&self) -> u16 {
        return self.player;
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[derive(Debug, Default, Clone)]
pub struct PlayerJoin {
    pub player: u16,
}
impl Message for PlayerJoin {
    fn decode_message(&mut self, pointer: &mut usize, data: &BitSlice<u64>) {
        self.player = data[*pointer..*pointer + BYTES2].load_le();
    }
    fn encode_message(&self) -> BitVec {
        let mut packet: BitVec = BitVec::with_capacity(BYTES2);
        packet.extend(self.player.view_bits::<Lsb0>());
        packet
    }
    fn get_message_type(&self) -> u8 {
        6
    }

    fn get_message_contents(&self) -> VariantArray {
        Array::from_iter([])
    }
    fn get_player(&self) -> u16 {
        self.player
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
