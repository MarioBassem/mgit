pub mod blob;
pub mod commit;
mod compress;
pub mod hash;
pub mod tag;
pub mod tree;

use std::{error::Error, ffi::OsString, fmt::Display, fs, io::Read, path::PathBuf};

use self::{
    compress::decompress,
    hash::{hash, Hash},
};

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
    pub data: Vec<u8>,
    pub kind: ObjectKind,
}

#[derive(Debug)]
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
        let hash = Hash::try_from(hash_hex.as_bytes())?;
        let (dir, file_name) = hash.get_object_path();
        let path = PathBuf::from(OBJECTS_DIR).join(dir).join(file_name);
        let file = fs::File::open(path)?;
        Self::read(file)
    }

    pub fn read<R: Read>(data: R) -> Result<Object> {
        // decompress content
        let data = decompress(data)?;

        let index = data
            .iter()
            .position(|c| *c == b'\0')
            .ok_or(anyhow!("invalid object data. failed to find NUL"))?;

        let (header_bytes, data) = data.split_at(index + 1);

        let header_str = String::from_utf8(header_bytes[0..index].to_vec())?;

        let (kind_str, length_str) = header_str
            .split_once(' ')
            .ok_or(anyhow!("invalid object header"))?;

        let kind = ObjectKind::try_from(kind_str)?;
        let length = usize::from_str_radix(length_str, 10)?;

        if length != data.len() {
            return Err(anyhow!("object size does not match"));
        }

        Ok(Object {
            data: data.to_vec(),
            kind,
        })
    }

    pub fn write(&self) -> Result<Hash> {
        let object_data = self.encode();

        // compress content
        let compressed_content = compress::compress(&object_data)?;

        // hash
        let hash = hash(&compressed_content);

        let (dir_name, file_name) = hash.get_object_path();

        fs::write(
            PathBuf::from(OBJECTS_DIR).join(dir_name).join(file_name),
            compressed_content,
        )?;

        // return hash
        Ok(hash)
    }

    /// encodes object content into a vector of bytes and adds the object header
    pub fn encode(&self) -> Vec<u8> {
        let mut data = Vec::new();
        data.append(&mut format!("{} {}\0", self.kind, self.data.len()).into_bytes());
        data.append(&mut self.data.clone());
        data
    }

    pub fn hash(&self) -> Result<Hash> {
        let blob_content = self.encode();

        // compress content
        let compressed_content = compress::compress(&blob_content)?;

        Ok(hash(&compressed_content))
    }
}

#[cfg(test)]
mod test {
    use crate::objects::{compress::compress, ObjectKind};

    use super::Object;

    #[test]
    fn test_read_blob() {
        let blob_data = String::from("hello world");

        let data = format!("blob {}\0{}", blob_data.len(), blob_data);
        let compressed_data = compress(&data.as_bytes().to_vec()).unwrap();

        let object = Object::read(&*compressed_data).unwrap();
        assert_eq!(object.kind.to_string(), ObjectKind::Blob.to_string());
        assert!(object.data == "hello world".as_bytes());
    }

    #[test]
    fn test_read_commit() {
        let commit_data = String::from("commit data");
        let data = format!("commit {}\0{}", commit_data.len(), commit_data);
        let compressed_data = compress(&data.as_bytes().to_vec()).unwrap();

        let object = Object::read(&*compressed_data).unwrap();
        assert_eq!(object.kind.to_string(), ObjectKind::Commit.to_string());
        assert!(object.data == "commit data".as_bytes());
    }

    #[test]
    fn test_read_tree() {
        let tree_data = String::from("tree data");
        let data = format!("tree {}\0{}", tree_data.len(), tree_data);
        let compressed_data = compress(&data.as_bytes().to_vec()).unwrap();

        let object = Object::read(&*compressed_data).unwrap();
        assert_eq!(object.kind.to_string(), ObjectKind::Tree.to_string());
        assert!(object.data == "tree data".as_bytes());
    }

    #[test]
    fn test_read_tag() {
        let tag_data = String::from("tag data");
        let data = format!("tag {}\0{}", tag_data.len(), tag_data);
        let compressed_data = compress(&data.as_bytes().to_vec()).unwrap();

        let object = Object::read(&*compressed_data).unwrap();
        assert_eq!(object.kind.to_string(), ObjectKind::Tag.to_string());
        assert!(object.data == "tag data".as_bytes());
    }
}
