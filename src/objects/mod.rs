mod blob;
mod commit;
mod compress;
mod hash;
mod tree;

use std::{
    error::Error,
    ffi::OsString,
    fmt::Display,
    fs::{self, create_dir_all},
    path::PathBuf,
};

use self::hash::{hash, Hash, HashHex};

use anyhow::Result;

const OBJECTS_DIR: String = String::from(".git/objects");

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

fn write_object(compressed_content: Vec<u8>) -> Result<Hash> {
    // hash
    let hash = hash(&compressed_content)?;

    // write blob to path from hashhex
    let hash_hex = HashHex::from(hash);

    let (dir_name, file_name) = hash_hex.get_object_path();

    create_dir_all(PathBuf::from(OBJECTS_DIR).join(dir_name))?;
    fs::write(
        PathBuf::from(OBJECTS_DIR).join(dir_name).join(file_name),
        compressed_content,
    )?;

    // return hash
    Ok(hash)
}
