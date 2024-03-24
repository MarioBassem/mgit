use std::{
    fs::{self},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{bail, Result};

use crate::objects::{
    compress::{self, decompress},
    hash::Hash,
    ObjectError,
};

use super::{hash::HashHex, write_object, OBJECTS_DIR};

/// write file as blob
pub fn write_blob(path: PathBuf) -> Result<Hash> {
    // read file content
    let file_content = fs::read_to_string(path)?;
    let blob_content = format!("blob {}\0{}", file_content.len(), file_content);

    // compress content
    let compressed_content = compress::compress(blob_content.into_bytes())?;

    // write object
    let hash = write_object(compressed_content)?;

    // return hash
    Ok(hash)
}

/// read blob contents
pub fn read_blob(hash_hex: HashHex) -> Result<String> {
    // get path from hashhex
    let (dir_name, file_name) = hash_hex.get_object_path();

    // read blob content
    let file = fs::File::open(PathBuf::from(OBJECTS_DIR).join(dir_name).join(file_name))?;

    // decompress content
    let decompressed_content = decompress(file)?;

    // parse content
    let content = parse_blob(decompressed_content)?;

    // return content
    Ok(content)
}

fn parse_blob(data: String) -> Result<String> {
    let mut rest = data;
    if !rest.starts_with("blob ") {
        bail!(ObjectError::ErrParse(String::from(
            "failed to read 'blob' type"
        )))
    }

    let mut rest = rest.split_off("blob ".len());
    let (size_str, content) = rest
        .split_once('\0')
        .ok_or(ObjectError::ErrParse(String::from(
            "failed to find null byte",
        )))?;

    // match size to content length
    let size = size_str.parse::<usize>()?;

    if content.len() != size {
        bail!(ObjectError::ErrParse(format!(
            "size is incorrect: found {} expected {}",
            content.len(),
            size
        )))
    }

    Ok(String::from_str(content)?)
}
