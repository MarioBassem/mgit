use std::{error::Error, fmt::Display};

use bytes::{Buf, Bytes};

use anyhow::Result;
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
        }
    }
}

#[derive(Debug, Clone)]
pub enum PackObject {
    Simple {
        kind: ObjectKind,
        data: bytes::Bytes,
        // offset: u64,
    },
    RefDelta {
        base_name: String,
        instrs: Vec<DeltaInstr>,
        offset: u64,
    },
    #[allow(dead_code)]
    OfsDelta {
        base_offset: u64,
        instrs: Vec<DeltaInstr>,
        offset: u64,
    },
}

impl PackObject {
    pub fn new_simple(object_type: PackObjectType, data: Bytes) -> PackObject {
        todo!()
    }

    pub fn new_ofs_delta(data: Bytes) -> PackObject {
        todo!()
    }

    pub fn new_ref_delte(data: Bytes) -> PackObject {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub enum DeltaInstr {
    Copy { offset: u64, size: u64 },
    Insert { data: bytes::Bytes },
}

#[derive(Debug, PartialEq, Eq)]
pub enum CopyFields {
    Offset1,
    Offset2,
    Offset3,
    Offset4,
    Size1,
    Size2,
    Size3,
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

            let object_data = self.data.split_to(usize::try_from(object_size)?);

            let obj = match object_type {
                PackObjectType::Blob
                | PackObjectType::Commit
                | PackObjectType::Tag
                | PackObjectType::Tree => PackObject::new_simple(object_type, object_data),
                PackObjectType::OfsDelta => PackObject::new_ofs_delta(object_data),
                PackObjectType::RefDelta => PackObject::new_ref_delte(object_data),
            };

            pack_objects.push(obj)
        }

        Ok(pack_objects)
    }
}
