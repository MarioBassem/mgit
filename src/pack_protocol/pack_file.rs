use crate::{
    objects::{hash::Hash, Object},
    pack_protocol::pack_object::{PackObject, PackObjectType},
};
use anyhow::Result;
use bytes::{Buf, Bytes};
use std::{collections::HashMap, error::Error, fmt::Display};

pub struct PackFile {
    data: Bytes,
    // objects_read: u32,
    items_expected: u32,
}

#[derive(Debug)]
pub enum PackFileError {
    /// indicates an invalid packfile signature
    ErrInvalidSignature,
    /// indicates an unsupported packfile version
    ErrVersionNotSupported,
    /// indicates an invalid pack object type
    ErrInvalidPackObjectType,
    /// indicates an invalid pack object length
    ErrInvalidPackObjectLength,
    /// indicates a mismatch between pack object size
    ErrPackObjectLengthMistmatch,
    /// indicates an invalid delta instruction
    ErrInvalidDeltaInstruction(String),
    /// indicates an error while referring to base object for offset delta object
    ErrOffsetDeltaBaseObject(String),
    /// indicates an error while referring to base object for ref delta object
    ErrRefDeltaBaseObject(String),
}

impl Error for PackFileError {}

impl Display for PackFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ErrInvalidSignature => write!(f, "invalid pack file signature"),
            Self::ErrVersionNotSupported => write!(f, "only pack version 2 is only supported"),
            Self::ErrInvalidPackObjectType => write!(f, "invalid pack object type"),
            Self::ErrInvalidPackObjectLength => write!(
                f,
                "pack object type and length must be less than or equal to 8 bytes"
            ),
            Self::ErrPackObjectLengthMistmatch => write!(
                f,
                "object length after decompression does not match with expected length in header"
            ),
            Self::ErrInvalidDeltaInstruction(err) => {
                write!(f, "invalid delta instruction: {}", err)
            }
            Self::ErrOffsetDeltaBaseObject(err) => {
                write!(f, "failed to get offset delta base object: {}", err)
            }
            Self::ErrRefDeltaBaseObject(err) => {
                write!(f, "failed to get ref delta base object: {}", err)
            }
        }
    }
}

impl PackFile {
    pub fn new(mut data: Bytes) -> Result<PackFile> {
        let items_expected = Self::read_header(&mut data)?;

        Ok(PackFile {
            data,
            items_expected,
            // objects_read: 0,
        })
    }

    fn read_header(data: &mut Bytes) -> Result<u32> {
        let signature = data.split_to(4);
        if *signature != *b"PACK" {
            return Err(PackFileError::ErrInvalidSignature.into());
        }

        let version = data.split_to(4);
        let version = u32::from_be_bytes([version[0], version[1], version[2], version[3]]);
        if version != 2 {
            return Err(PackFileError::ErrVersionNotSupported.into());
        }

        let items_expexted_bytes = data.split_to(4);
        let items_expected = u32::from_be_bytes([
            items_expexted_bytes[0],
            items_expexted_bytes[1],
            items_expexted_bytes[2],
            items_expexted_bytes[3],
        ]);

        Ok(items_expected)
    }

    pub fn read_objects(&mut self) -> Result<Vec<PackObject>> {
        let original_data_size = self.data.len();
        let mut pack_objects = Vec::new();

        for _ in 0..self.items_expected {
            let cur_data_size = self.data.len();
            let mut object_header_bytes = Vec::new();
            loop {
                let b = self.data.get_u8();

                object_header_bytes.push(b);

                if b & (1 << 7) == 0 {
                    break;
                }
            }

            let object_type: PackObjectType =
                ((object_header_bytes[0] & 0b0111_0000) >> 4).try_into()?;

            if object_header_bytes.len() > 8 {
                return Err(PackFileError::ErrInvalidPackObjectLength.into());
            }

            let mut object_size = (object_header_bytes[0] & 0b0000_1111) as u64;

            for (i, b) in object_header_bytes[1..].iter().enumerate() {
                object_size |= ((b & 0b0111_1111) as u64) << (7 * i + 4);
            }

            let obj = match object_type {
                PackObjectType::Blob
                | PackObjectType::Commit
                | PackObjectType::Tag
                | PackObjectType::Tree => PackObject::new_simple(
                    &mut self.data,
                    object_size.try_into()?,
                    original_data_size - cur_data_size,
                )?,
                PackObjectType::OfsDelta => PackObject::new_ofs_delta(
                    &mut self.data,
                    object_size.try_into()?,
                    original_data_size - cur_data_size,
                )?,
                PackObjectType::RefDelta => PackObject::new_ref_delte(
                    &mut self.data,
                    object_size.try_into()?,
                    original_data_size - cur_data_size,
                )?,
            };

            pack_objects.push(obj);
        }

        Ok(pack_objects)
    }

    pub fn build_objects(&self, mut pack_objs: Vec<PackObject>) -> Result<Vec<Object>> {
        let mut offset_index = HashMap::new();
        let mut hash_index = HashMap::new();
        let mut objs = Vec::new();
        for (_, pack_obj) in pack_objs.iter_mut().enumerate() {
            match pack_obj {
                PackObject::OfsDelta {
                    offset,
                    base_offset,
                    instructions,
                    base_size,
                    reconstructed_size,
                } => {
                    let base_index = offset_index.get(base_offset).ok_or(
                        PackFileError::ErrOffsetDeltaBaseObject(format!(
                            "base object not found at offset {}",
                            base_offset
                        )),
                    )?;

                    let base_obj: &Object = objs.get(*base_index).unwrap(); // should always succeed
                    assert_eq!(*base_size, base_obj.data.len());

                    let new_obj = PackObject::apply_delta_instructions(base_obj, instructions)?;
                    assert_eq!(*reconstructed_size, new_obj.data.len());

                    let hash = new_obj.hash()?;
                    objs.push(new_obj);
                    offset_index.insert(offset, objs.len() - 1);
                    hash_index.insert(hash.to_hex(), objs.len() - 1);
                }
                PackObject::RefDelta {
                    offset,
                    base_name,
                    instructinos,
                    base_size,
                    reconstructed_size,
                } => {
                    let base_index = hash_index.get(&Hash(base_name.clone()).to_hex()).ok_or(
                        PackFileError::ErrRefDeltaBaseObject(format!(
                            "base object not found: {:02x?}",
                            base_name
                        )),
                    )?;

                    let base_obj = objs.get(*base_index).unwrap(); // should always succeed
                    assert_eq!(*base_size, base_obj.data.len());

                    let new_obj = PackObject::apply_delta_instructions(base_obj, instructinos)?;
                    assert_eq!(*reconstructed_size, new_obj.data.len());

                    let hash = new_obj.hash()?;
                    objs.push(new_obj);
                    offset_index.insert(offset, objs.len() - 1);
                    hash_index.insert(hash.to_hex(), objs.len() - 1);
                }
                PackObject::Simple { data, offset } => {
                    let object = Object::read(data.reader())?;
                    let hash = object.hash()?;
                    objs.push(object);
                    offset_index.insert(offset, objs.len() - 1);
                    hash_index.insert(hash.to_hex(), objs.len() - 1);
                }
            }
        }

        Ok(objs)
    }
}
