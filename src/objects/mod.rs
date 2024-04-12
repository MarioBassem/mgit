pub mod blob;
pub mod commit;
mod compress;
pub mod hash;
pub mod tag;
pub mod tree;

use std::{
    error::Error,
    ffi::OsString,
    fmt::Display,
    fs::{self, create_dir_all},
    path::PathBuf,
};

use self::hash::{hash, Hash};

use anyhow::{anyhow, Result};

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

pub struct Object {
    data: Vec<u8>,
    kind: ObjectKind,
}

pub enum ObjectKind {
    Blob,
    Commit,
    Tree,
    Tag,
}

impl TryFrom<&str> for ObjectKind {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        let val = match value {
            "blob" => ObjectKind::Blob,
            "commit" => ObjectKind::Commit,
            "tag" => ObjectKind::Tag,
            "tree" => ObjectKind::Tree,
            _ => return Err(anyhow!("invalid object kind {}", value)),
        };

        Ok(val)
    }
}

impl Display for ObjectKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObjectKind::Blob => write!(f, "blob"),
            ObjectKind::Commit => write!(f, "commit"),
            ObjectKind::Tag => write!(f, "tag"),
            ObjectKind::Tree => write!(f, "tree"),
        }
    }
}

impl Object {
    pub fn read_from_hash(hash_hex: String) -> Result<Object> {
        todo!()
    }

    pub fn read_from_path(path: PathBuf) -> Result<Object> {
        todo!()
    }

    pub fn write(&self) -> Result<Hash> {
        let mut object_data = self.encode();

        // compress content
        let compressed_content = compress::compress(&object_data)?;

        // hash
        let hash = hash(&compressed_content);

        let (dir_name, file_name) = hash.get_object_path();

        create_dir_all(PathBuf::from(OBJECTS_DIR).join(dir_name))?;
        fs::write(
            PathBuf::from(OBJECTS_DIR).join(dir_name).join(file_name),
            compressed_content,
        )?;

        // return hash
        Ok(hash)
    }

    /// encodes object content into a vector of bytes and adds the object header
    pub fn encode(&self) -> Vec<u8> {
        todo!()
    }

    pub fn hash(&self) -> Result<Hash> {
        let mut blob_content = self.encode();

        // compress content
        let compressed_content = compress::compress(&blob_content)?;

        Ok(hash(&compressed_content))
    }
    // /// gets 40 byte hash in hexadecimal format
    // fn hash_hex(&self) -> Result<HashHex> {
    //     Ok(HashHex::from(&self.hash()?))
    // }

    /// returns decompressed object data
    fn data(&self) -> &Vec<u8> {
        todo!()
    }

    // pub fn size(&self) -> usize {
    //     self.data.len()
    // }
}
