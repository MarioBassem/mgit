pub mod blob;
pub mod commit;
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
    commit::Author,
    compress::compress,
    hash::{hash, Hash, HashHex},
    tree::Entry,
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
    Blob {
        data: Vec<u8>,
    },
    Commit {
        tree: HashHex,
        author: Author,
        committer: Option<Author>,
        parents: Vec<HashHex>,
        message: String,
    },
    Tree {
        entries: Vec<Entry>,
    },
    Tag {
        object: HashHex,
        object_type: ObjectKind,
        tag_name: String,
        tagger: Author,
        commit_message: Option<String>,
        signature: Option<String>,
    },
}

pub enum ObjectKind {
    Blob,
    Commit,
    Tree,
    Tag,
}

impl Object {
    pub fn read_from_hash(hash_hex: HashHex) -> Result<Object> {
        todo!()
    }

    pub fn read_from_path(path: PathBuf) -> Result<Object> {
        todo!()
    }

    pub fn write(&self) -> Result<Hash> {
        let mut object_data = self.prep_content();

        // compress content
        let compressed_content = compress::compress(&object_data)?;

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

    fn prep_content(&self) -> Vec<u8> {
        let content = Vec::new();

        let pre = match self.kind {
            ObjectKind::Blob => "blob",
            ObjectKind::Commit => "commit",
            ObjectKind::Tag => "tag",
            ObjectKind::Tree => "tree",
        };

        content.append(&mut format!("{} {}\0", pre, self.data.len()).as_bytes());
        content.append(&mut self.data.clone());

        content
    }

    pub fn hash(&self) -> Result<Hash> {
        let mut blob_content = self.prep_content();

        // compress content
        let compressed_content = compress::compress(&blob_content)?;

        Ok(hash(&compressed_content))
    }
    /// gets 40 byte hash in hexadecimal format
    fn hash_hex(&self) -> Result<HashHex> {
        Ok(HashHex::from(&self.hash()?))
    }

    /// returns decompressed object data
    fn data(&self) -> &Vec<u8> {
        todo!()
    }

    // pub fn size(&self) -> usize {
    //     self.data.len()
    // }

    fn write_object(&self, compressed_content: Vec<u8>) -> Result<Hash> {}
}
