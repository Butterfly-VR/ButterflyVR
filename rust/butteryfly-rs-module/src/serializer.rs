// serialization functions for networkednode values
use bitvec::prelude::*;
use godot::prelude::*;
use std::{borrow::Cow, i64};

const BYTE: usize = 8;
const BYTES2: usize = 16;
const BYTES4: usize = 32;
const BYTES8: usize = 64;
// all possible ways a value can be encoded for the network
#[derive(PartialEq, Debug, Copy, Clone)]
pub enum NetworkedValueTypes {
    Nil,
    Bool,
    Unsigned8,
    Unsigned16,
    Signed64,
    Float32,
    Vector3,
    String,
    ByteArray,
}
impl TryFrom<i64> for NetworkedValueTypes {
    type Error = Cow<'static, str>;
    fn try_from(value: i64) -> Result<Self, Self::Error> {
        match value {
            -2 => Ok(NetworkedValueTypes::Nil),
            -1 => Err(Cow::Borrowed("invalid type")),
            0 => Ok(NetworkedValueTypes::Bool),
            1 => Ok(NetworkedValueTypes::Unsigned8),
            2 => Ok(NetworkedValueTypes::Unsigned16),
            3 => Ok(NetworkedValueTypes::Signed64),
            4 => Ok(NetworkedValueTypes::Float32),
            5 => Ok(NetworkedValueTypes::Vector3),
            6 => Ok(NetworkedValueTypes::String),
            7 => Ok(NetworkedValueTypes::ByteArray),
            _ => Err(Cow::Owned(format!(
                "tried to parse nonexistent type {:#?}",
                value
            ))),
        }
    }
}
pub fn decode_with_known_type(
    data: &BitSlice<u64>,
    pointer: &mut usize,
    object_type: &NetworkedValueTypes,
) -> Option<Variant> {
    match object_type {
        NetworkedValueTypes::Nil => Some(Variant::nil()),
        NetworkedValueTypes::Bool => {
            if *pointer + 1 > data.len() {
                return None;
            }
            let data = Some(data[*pointer].to_variant());
            *pointer += 1;
            data
        }
        NetworkedValueTypes::Unsigned8 => {
            if *pointer + BYTE > data.len() {
                return None;
            }
            let data = Some(data[*pointer..*pointer + BYTE].load_le::<u8>().to_variant());
            *pointer += BYTE;
            data
        }
        NetworkedValueTypes::Unsigned16 => {
            if *pointer + BYTES2 > data.len() {
                return None;
            }
            let data = Some(
                data[*pointer..*pointer + BYTES2]
                    .load_le::<u16>()
                    .to_variant(),
            );
            *pointer += BYTES2;
            data
        }
        NetworkedValueTypes::Signed64 => {
            if *pointer + BYTES8 > data.len() {
                return None;
            }
            let data = Some(
                data[*pointer..*pointer + BYTES8]
                    .load_le::<i64>()
                    .to_variant(),
            );
            *pointer += BYTES8;
            data
        }
        NetworkedValueTypes::Float32 => {
            if *pointer + BYTES4 > data.len() {
                return None;
            }
            let data = Some(
                f32::from_ne_bytes(
                    data[*pointer..*pointer + BYTES4]
                        .load_le::<u32>()
                        .to_ne_bytes(),
                )
                .to_variant(),
            );
            *pointer += BYTES4;
            data
        }
        NetworkedValueTypes::Vector3 => {
            if *pointer + (BYTES4 * 3) > data.len() {
                return None;
            }
            let x = f32::from_ne_bytes(
                data[*pointer..*pointer + BYTES4]
                    .load_le::<u32>()
                    .to_ne_bytes(),
            );
            *pointer += BYTES4;
            let y = f32::from_ne_bytes(
                data[*pointer..*pointer + BYTES4]
                    .load_le::<u32>()
                    .to_ne_bytes(),
            );
            *pointer += BYTES4;
            let z = f32::from_ne_bytes(
                data[*pointer..*pointer + BYTES4]
                    .load_le::<u32>()
                    .to_ne_bytes(),
            );
            *pointer += BYTES4;
            Some(Vector3::new(x, y, z).to_variant())
        }
        NetworkedValueTypes::String => {
            if *pointer + BYTES4 > data.len() {
                return None;
            }
            let length = data[*pointer..*pointer + BYTES4].load_le::<u32>() as usize;
            *pointer += BYTES4;
            if *pointer + (length * BYTE) <= data.len() {
                let result = Some(
                    str::from_utf8(
                        &data[*pointer..*pointer + (length * BYTE)]
                            .chunks_exact(BYTE)
                            .map(|x| x.load_le::<u8>())
                            .collect::<Vec<u8>>(),
                    )
                    .unwrap_or("")
                    .to_variant(),
                );
                *pointer += length * BYTE;
                return result;
            }
            return None;
        }

        NetworkedValueTypes::ByteArray => {
            if *pointer + BYTES4 > data.len() {
                return None;
            }
            let length = data[*pointer..*pointer + BYTES4].load_le::<u32>() as usize;
            *pointer += BYTES4;
            if *pointer + (length * BYTE) <= data.len() {
                let result = Some(
                    data[*pointer..*pointer + (length * BYTE)]
                        .chunks_exact(BYTE)
                        .map(|x| x.load_le::<u8>())
                        .collect::<Vec<u8>>()
                        .to_variant(),
                );
                *pointer += length * BYTE;
                return result;
            }
            return None;
        }
    }
}
pub fn encode_with_known_type(
    object: &Variant,
    object_type: &NetworkedValueTypes,
) -> BitVec<u64, Lsb0> {
    match object_type {
        NetworkedValueTypes::Nil => BitVec::new(),
        NetworkedValueTypes::Bool => {
            let value: bool = bool::from_variant(object);
            let mut bitvec = BitVec::with_capacity(1);
            bitvec.push(value);
            bitvec
        }
        NetworkedValueTypes::Unsigned8 => {
            let value = u8::from_variant(object);
            let mut bitvec = BitVec::with_capacity(BYTE);
            bitvec.extend(value.view_bits::<Lsb0>());
            bitvec
        }
        NetworkedValueTypes::Unsigned16 => {
            let value = u16::from_variant(object);
            let mut bitvec = BitVec::with_capacity(BYTES2);
            bitvec.extend(value.view_bits::<Lsb0>());
            bitvec
        }
        NetworkedValueTypes::Signed64 => {
            let value = i64::from_variant(object) as u64;
            let mut bitvec = BitVec::with_capacity(BYTES8);
            bitvec.extend(value.view_bits::<Lsb0>());
            bitvec
        }
        NetworkedValueTypes::Float32 => {
            let value: u32 = u32::from_ne_bytes(f32::from_variant(object).to_ne_bytes());
            let mut bitvec = BitVec::with_capacity(BYTES4);
            bitvec.extend(value.view_bits::<Lsb0>());
            bitvec
        }
        NetworkedValueTypes::Vector3 => {
            let vector = Vector3::from_variant(object);
            let mut bitvec = BitVec::with_capacity(BYTES4 * 3);
            let x = u32::from_ne_bytes(vector.x.to_ne_bytes());
            bitvec.extend(x.view_bits::<Lsb0>());
            let y = u32::from_ne_bytes(vector.y.to_ne_bytes());
            bitvec.extend(y.view_bits::<Lsb0>());
            let z = u32::from_ne_bytes(vector.z.to_ne_bytes());
            bitvec.extend(z.view_bits::<Lsb0>());
            bitvec
        }
        NetworkedValueTypes::String => {
            let string = String::from_variant(object);
            let length = string.len() as u32;
            let mut bitvec = BitVec::with_capacity(BYTES4 + (BYTE * length as usize));
            bitvec.extend(length.view_bits::<Lsb0>());
            for char in string.bytes() {
                bitvec.extend(char.view_bits::<Lsb0>());
            }
            bitvec
        }
        NetworkedValueTypes::ByteArray => {
            let bytes: Array<i64> = Array::from_variant(object);
            let length = bytes.len() as u32;
            let mut bitvec = BitVec::with_capacity(BYTES4 + (BYTE * length as usize));
            bitvec.extend(length.view_bits::<Lsb0>());
            for byte in bytes.iter_shared().map(|x| x.try_into().unwrap_or(0u8)) {
                bitvec.extend(byte.view_bits::<Lsb0>());
            }
            bitvec
        }
    }
}
