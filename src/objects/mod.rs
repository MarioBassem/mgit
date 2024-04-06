mod blob;
mod commit;
mod compress;
pub mod hash;
mod tag;
mod tree;

use std::{
    error::Error,
    ffi::OsString,
    fmt::Display,
    fs::{self, create_dir_all},
    path::PathBuf,
};

use self::{
    compress::compress,
    hash::{hash, Hash, HashHex},
};

use anyhow::Result;
use bytes::Bytes;

const OBJECTS_DIR: &str = ".git/objects";

#[derive(Debug)]
enum ObjectError {
    /// indicates a parsing error
    ErrParse(String),
    /// indicates an invalid mode
    ErrInvalidMode(usize),
    /// indicates an invalid file name
    ErrInvalidFileName(OsString),
}

impl Error for ObjectError {}

impl Display for ObjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectError::ErrParse(err) => write!(f, "object parsing error: {}", err),
            ObjectError::ErrInvalidMode(mode) => write!(f, "invalid tree entry mode: {}", mode),
            ObjectError::ErrInvalidFileName(name) => {
                write!(f, "file name is not Unicode: {:?}", name)
            }
        }
    }
}

pub enum Object {
    Blob { data: Bytes },
    Commit { data: Bytes },
    Tree { data: Bytes },
    Tag { data: Bytes },
}

pub trait ObjectTrait {
    /// writes object to appropriate place
    fn write(&self) -> Result<()>;
    /// hashes object data
    fn hash(&self) -> Result<Vec<u8>> {
        hash(self.data())
    }
    /// gets 40 byte hash in hexadecimal format
    fn hash_hex(&self) -> String {
        format!("{:02x?}", self.data())
    }
    /// compresses object data
    fn compress(&self) -> Result<Vec<u8>> {
        compress(self.data())
    }
    /// returns decompressed object data
    fn data(&self) -> &Vec<u8>;

    fn size(&self) -> usize {
        self.data().len()
    }
}

impl Object {
    pub fn new(data: Vec<u8>) -> Result<Object> {
        todo!()
    }

    pub fn hash(&self) -> Result<Vec<u8>> {
        todo!()
    }

    pub fn size(&self) -> usize {
        todo!()
    }
}

fn write_object(compressed_content: Vec<u8>) -> Result<Hash> {
    // hash
    let hash = hash(&compressed_content)?;

    // write blob to path from hashhex
    let hash_hex = HashHex::from(&hash);

    let (dir_name, file_name) = hash_hex.get_object_path();

    create_dir_all(PathBuf::from(OBJECTS_DIR).join(dir_name))?;
    fs::write(
        PathBuf::from(OBJECTS_DIR).join(dir_name).join(file_name),
        compressed_content,
    )?;

    // return hash
    Ok(hash)
}
