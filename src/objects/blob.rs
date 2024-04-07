use std::{
    fs::{self},
    path::PathBuf,
    str::FromStr,
};

use anyhow::{bail, Result};

use crate::objects::{compress::decompress, ObjectError};

use super::{hash::HashHex, Object, OBJECTS_DIR};

pub fn new_blob(data: Vec<u8>) -> Object {
    Object::Blob { data }
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
