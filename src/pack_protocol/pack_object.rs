use crate::{objects::Object, pack_protocol::pack_file::PackFileError};
use std::io::Read;

use bytes::{Buf, Bytes};

use anyhow::Result;
use flate2::read::ZlibDecoder;

#[derive(Debug, Clone)]
pub enum PackObject {
    Simple {
        // kind: ObjectKind,
        data: bytes::Bytes,
        offset: usize,
    },
    RefDelta {
        offset: usize,
        base_name: Vec<u8>,
        instructinos: Vec<DeltaInstruction>,
        base_size: usize,
        reconstructed_size: usize,
    },
    OfsDelta {
        offset: usize,
        base_offset: usize,
        instructions: Vec<DeltaInstruction>,
        base_size: usize,
        reconstructed_size: usize,
    },
}

impl PackObject {
    pub fn new_simple(data: &mut Bytes, size: usize, obj_offset: usize) -> Result<PackObject> {
        // data is the compressed object data
        let mut decompressed = Vec::new();
        let read = ZlibDecoder::new(&data[..]).read_to_end(&mut decompressed)?;
        if decompressed.len() != size {
            return Err(PackFileError::ErrPackObjectLengthMistmatch.into());
        }
        data.advance(read);

        Ok(PackObject::Simple {
            data: Bytes::from(decompressed),
            offset: obj_offset,
        })
    }

    pub fn new_ofs_delta(data: &mut Bytes, size: usize, obj_offset: usize) -> Result<PackObject> {
        /*
            data:
                negative relative offset from the delta object's position in the pack
                compressed delta data
        */
        let offset = Self::read_variable_length(data);

        let mut decompressed = Vec::new();
        let read = ZlibDecoder::new(&data[..]).read_to_end(&mut decompressed)?;
        if decompressed.len() != size {
            return Err(PackFileError::ErrPackObjectLengthMistmatch.into());
        }
        data.advance(read);

        let mut decompressed_bytes = Bytes::from(decompressed);
        let base_size = usize::try_from(Self::read_variable_length(&mut decompressed_bytes))?;
        let reconstructed_size =
            usize::try_from(Self::read_variable_length(&mut decompressed_bytes))?;

        let instructions = Self::parse_delta_instructions(decompressed_bytes)?;

        Ok(PackObject::OfsDelta {
            offset: obj_offset,
            base_offset: obj_offset - usize::try_from(offset)?,
            instructions,
            base_size,
            reconstructed_size,
        })
    }

    pub fn new_ref_delte(data: &mut Bytes, size: u64, obj_offset: usize) -> Result<PackObject> {
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

        let mut decompressed_bytes = Bytes::from(decompressed);
        let base_size = usize::try_from(Self::read_variable_length(&mut decompressed_bytes))?;
        let reconstructed_size =
            usize::try_from(Self::read_variable_length(&mut decompressed_bytes))?;

        let instructions = Self::parse_delta_instructions(decompressed_bytes)?;

        Ok(PackObject::RefDelta {
            offset: obj_offset,
            base_name: base_obj_name.to_vec(),
            instructinos: instructions,
            base_size,
            reconstructed_size,
        })
    }

    fn read_variable_length(data: &mut Bytes) -> u64 {
        let mut size: u64 = 0;
        for i in 0.. {
            let b = data.get_u8();
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

        Ok(instructions)
    }

    pub fn apply_delta_instructions(
        pack_obj: &PackObject,
        base_obj: &Object,
        instructions: &Vec<DeltaInstruction>,
    ) -> Result<Object> {
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
