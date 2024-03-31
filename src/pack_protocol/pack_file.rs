use bytes::{Buf, Bytes};

use anyhow::Result;
use reqwest::header;

use crate::objects::ObjectKind;

pub struct PackFile {
    data: Bytes,
    objects_read: u32,
    items_expected: Option<u32>,
}

#[derive(Debug)]
pub struct PackHeader {
    signature: bytes::Bytes,
    version: u32,
    entries: u32,
}

/*
    header:
        - 4 byte signature "PACK"
        - 4 byte version number (Git currently accepts version number 2 or 3 but generates version 2 only.)
        - 4-byte number of objects contained in the pack (network byte order)


*/

#[derive(Debug, Clone)]
pub enum PackObject {
    Simple {
        kind: ObjectKind,
        data: bytes::Bytes,
        offset: u64,
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

impl TryFrom<&str> for ObjectKind {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "blob" => Ok(Self::Blob),
            "tree" => Ok(Self::Tree),
            "commit" => Ok(Self::Commit),
            _ => Err(anyhow::anyhow!("invalid object kind: '{value}'")),
        }
    }
}

impl PackFile {
    pub fn new(data: Bytes) -> PackFile {
        PackFile {
            data,
            items_expected: None,
            objects_read: 0,
        }
    }

    pub fn read_header(&mut self) -> Result<PackHeader> {
        let signature = self.data.split_to(4);
        anyhow::ensure!(*signature == *b"PACK", "invalid signature: {:?}", signature);

        let version = self.data.split_to(4);
        let version = u32::from_be_bytes([version[0], version[1], version[2], version[3]]);
        anyhow::ensure!(version == 2, "only pack version 2 is supported");

        let entries = self.data.split_to(4);
        let entries = u32::from_be_bytes([entries[0], entries[1], entries[2], entries[3]]);

        self.items_expected = Some(entries);

        Ok(PackHeader {
            entries,
            signature,
            version,
        })
    }

    pub fn read_item(&mut self) -> Result<Option<PackObject>> {
        anyhow::ensure!(
            self.items_expected.is_none(),
            "header must be read before reading items"
        );

        if let Some(expected) = self.items_expected {
            // should always be the case
            if self.objects_read == expected {
                return Ok(None);
            }
        }

        let mut header_bytes = Vec::new();
        loop {
            let b = self.data.get_u8();
            header_bytes.push(b);

            if b & 0b1000_0000 == 0 {
                break;
            }
        }

        anyhow::ensure!(
            header_bytes.len() <= 8,
            "headers with more than 8 bytes are not supported"
        );

        let object_type: PackObjectType = ((header_bytes[0] & 0b0111_0000) >> 4).try_into()?;
        let mut object_size = (header_bytes[0] & 0b0000_1111) as u64;

        for (i, b) in header_bytes[1..].iter().enumerate() {
            object_size |= ((b & 0b0111_1111) as u64) << (7 * i + 4);
        }

        // let resut =
        todo!()
    }
}
