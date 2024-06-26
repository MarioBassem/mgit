use std::{fs, io::Read, path::PathBuf};

use anyhow::{anyhow, Result};

use crate::objects::{hash::Hash, Object, ObjectKind};

pub fn hash_object(path: PathBuf, write: bool) -> Result<()> {
    if !path.is_file() {
        return Err(anyhow!("path is not a file"));
    }

    let mut file_contents = fs::File::open(path)?;
    let mut data = Vec::new();
    file_contents.read(&mut data)?;

    let object = Object {
        data,
        kind: ObjectKind::Blob,
    };

    let hash: Hash;
    if write {
        hash = object.write()?;
    } else {
        hash = object.hash()?;
    }

    println!("{:x}", hash);

    Ok(())
}
