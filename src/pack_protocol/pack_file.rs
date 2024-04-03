use std::{error::Error, fmt::Display, io::Read};

use bytes::{Buf, Bytes};

use anyhow::Result;
use flate2::read::ZlibDecoder;
use reqwest::header;

use crate::objects::ObjectKind;
use bit_set::{self, BitSet};
use num_bigint::{self, BigInt};

pub struct PackFile {
    data: Bytes,
    // objects_read: u32,
    items_expected: u32,
}

// #[derive(Debug)]
// pub struct PackHeader {
//     signature: bytes::Bytes,
//     version: u32,
//     entries: u32,
// }

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
        }
    }
}

#[derive(Debug, Clone)]
pub enum PackObject {
    Simple {
        // kind: ObjectKind,
        data: bytes::Bytes,
        // offset: u64,
    },
    RefDelta {
        base_name: Vec<u8>,
        instrs: Vec<DeltaInstruction>,
        base_size: u64,
        reconstructed_size: u64,
    },
    #[allow(dead_code)]
    OfsDelta {
        base_offset: u64,
        instrs: Vec<DeltaInstruction>,
    },
}

impl PackObject {
    pub fn new_simple(data: &mut Bytes, size: u64) -> Result<PackObject> {
        // data is the compressed object data
        let mut decompressed = Vec::new();
        let read = ZlibDecoder::new(&data[..]).read_to_end(&mut decompressed)?;
        if read as u64 != size {
            return Err(PackFileError::ErrPackObjectLengthMistmatch.into());
        }
        data.advance(read);

        Ok(PackObject::Simple {
            data: Bytes::from(decompressed),
        })
    }

    pub fn new_ofs_delta(data: &mut Bytes, size: u64) -> Result<PackObject> {
        /*
            data:
                negative relative offset from the delta object's position in the pack
                compressed delta data
        */

        todo!()
    }

    pub fn new_ref_delte(data: &mut Bytes, size: u64) -> Result<PackObject> {
        /*
           data:
               base object name
               compressed delta data
        */
        let base_obj_name = data.split_to(20);

        let mut decompressed = Vec::new();
        let read = ZlibDecoder::new(&data[..]).read_to_end(&mut decompressed)?;
        if read as u64 != size {
            return Err(PackFileError::ErrPackObjectLengthMistmatch.into());
        }
        data.advance(read);

        let base_size = Self::read_size(&mut decompressed);
        let reconstructed_size = Self::read_size(&mut decompressed);

        let instructions = Self::parse_delta_instructions(Bytes::from(decompressed))?;

        Ok(PackObject::RefDelta {
            base_name: base_obj_name.to_vec(),
            instrs: instructions,
            base_size,
            reconstructed_size,
        })
    }

    fn read_size(data: &mut Vec<u8>) -> u64 {
        let mut size: u64 = 0;
        for (i, b) in data.iter().enumerate() {
            size |= ((b & 0b0111_1111) as u64) << (7 * i);

            if b & (1 << 7) == 0 {
                break;
            }
        }

        size
    }

    fn parse_delta_instructions(mut data: Bytes) -> Result<Vec<DeltaInstruction>> {
        let mut instructions = Vec::new();
        while !data.is_empty() {
            let b = data.get_u8();

            if b == 0 {
                return Err(PackFileError::ErrInvalidDeltaInstruction(format!(
                    "the 0 instruction is reserved for future expansion"
                ))
                .into());
            }

            if b & (1 << 7) == 0 {
                // add instruction
                let size = b;
                let add = data.split_to(size.into());

                instructions.push(DeltaInstruction::Insert {
                    data: Bytes::from(add),
                })
            } else {
                // copy instruction
                let mut offset: u64 = 0;
                let mut size: u64 = 0;
                for i in 0..4 {
                    if b & (1 << i) == 1 {
                        let next = data.get_u8();
                        offset |= (next << (8 * i)) as u64
                    }
                }

                for i in 0..3 {
                    if b & (1 << (i + 4)) == 1 {
                        let next = data.get_u8();
                        size |= (next << (8 * i)) as u64
                    }
                }

                if size == 0 {
                    size = 0x10000;
                }

                instructions.push(DeltaInstruction::Copy { offset, size })
            }
        }
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum DeltaInstruction {
    Copy { offset: u64, size: u64 },
    Insert { data: bytes::Bytes },
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PackObjectType {
    Commit,
    Tree,
    Blob,
    Tag,
    OfsDelta,
    RefDelta,
}

impl TryFrom<u8> for PackObjectType {
    type Error = PackFileError;
    fn try_from(value: u8) -> std::prelude::v1::Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Commit),
            2 => Ok(Self::Tree),
            3 => Ok(Self::Blob),
            4 => Ok(Self::Tag),
            6 => Ok(Self::OfsDelta),
            7 => Ok(Self::RefDelta),
            _ => Err(PackFileError::ErrInvalidPackObjectType),
        }
    }
}

// impl TryFrom<&str> for ObjectKind {
//     type Error = anyhow::Error;
//     fn try_from(value: &str) -> Result<Self, Self::Error> {
//         match value {
//             "blob" => Ok(Self::Blob),
//             "tree" => Ok(Self::Tree),
//             "commit" => Ok(Self::Commit),
//             _ => Err(anyhow::anyhow!("invalid object kind: '{value}'")),
//         }
//     }
// }

impl PackFile {
    pub fn new(mut data: Bytes) -> Result<PackFile> {
        let items_expected = Self::read_header(&mut data)?;

        Ok(PackFile {
            data,
            items_expected,
            // objects_read: 0,
        })
    }

    pub fn read_header(data: &mut Bytes) -> Result<u32> {
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

    fn read_objects(&mut self) -> Result<Vec<PackObject>> {
        let mut pack_objects = Vec::new();
        for _ in 0..self.items_expected {
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
                | PackObjectType::Tree => PackObject::new_simple(&mut self.data, object_size)?,
                PackObjectType::OfsDelta => PackObject::new_ofs_delta(&mut self.data, object_size)?,
                PackObjectType::RefDelta => PackObject::new_ref_delte(&mut self.data, object_size)?,
            };

            pack_objects.push(obj)
        }

        Ok(pack_objects)
    }
}
