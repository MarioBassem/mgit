use anyhow::{bail, Ok, Result};
use std::{
    fmt::Display,
    fs::{self, DirEntry},
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    str::FromStr,
};

use crate::objects::{
    blob::write_blob,
    compress::{compress, decompress},
    write_object, OBJECTS_DIR,
};

use super::{
    hash::{Hash, HashHex},
    ObjectError,
};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entry {
    mode: EntryMode,
    name: String,
    hash: Hash,
}

impl Entry {
    pub fn from_dir_entry(dir_entry: DirEntry) -> Result<Entry> {
        let mut mode = EntryMode::RegularFile;
        if dir_entry.file_type()?.is_dir() {
            mode = EntryMode::Directory;
        } else if dir_entry.file_type()?.is_symlink() {
            mode = EntryMode::SymbolicLink;
        } else if dir_entry.file_type()?.is_file() {
            if dir_entry.metadata()?.permissions().mode() & 0111 != 0 {
                mode = EntryMode::ExecutableFile;
            }
        }

        let os_name = dir_entry.file_name();

        let hash = match mode {
            EntryMode::Directory => write_tree(dir_entry.path())?,
            _ => write_blob(dir_entry.path())?,
        };

        let name = os_name
            .into_string()
            .map_err(|name| ObjectError::ErrInvalidFileName(name))?;

        Ok(Entry { hash, mode, name })
    }
}

impl Into<Vec<u8>> for Entry {
    fn into(self) -> Vec<u8> {
        let mut v = format!("{} {}\0", self.mode, self.name).as_bytes().to_vec();

        let mut hash: Vec<u8> = self.hash.into();
        v.append(&mut hash);

        v
    }
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
enum EntryMode {
    RegularFile = 0o100644,
    ExecutableFile = 0o100755,
    SymbolicLink = 0o120000,
    Directory = 0o40000,
}

impl Display for EntryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryMode::Directory => write!(f, "040000"),
            EntryMode::ExecutableFile => write!(f, "100755"),
            EntryMode::RegularFile => write!(f, "100644"),
            EntryMode::SymbolicLink => write!(f, "120000"),
        }
    }
}
pub fn write_tree(path: PathBuf) -> Result<Hash> {
    // list current dir entries
    let paths = fs::read_dir("./")?;

    // for each entry call its write method
    let mut children_content = Vec::new();
    for path in paths {
        let dir_entry = path?;
        let entry = Entry::from_dir_entry(dir_entry)?;
        children_content.append(&mut entry.into());
    }

    // generate tree file content
    let mut tree_object_content = format!("tree {}\0", children_content.len())
        .as_bytes()
        .to_vec();

    tree_object_content.append(&mut children_content);
    // compress content
    let compressed_content = compress(tree_object_content)?;

    // write object
    let hash = write_object(compressed_content)?;

    // return hash
    Ok(hash)
}

pub fn read_tree(hash_hex: HashHex) -> Result<Vec<Entry>> {
    // get path from hashhex
    let (dir_name, file_name) = hash_hex.get_object_path();

    // read tree content
    let file = fs::File::open(PathBuf::from(OBJECTS_DIR).join(dir_name).join(file_name))?;

    // decompress
    let decompressed_content = decompress(file)?;

    // parse
    let entries = parse_tree(decompressed_content)?;

    // return list of entries
    Ok(entries)
}

fn parse_tree(data: String) -> Result<Vec<Entry>> {
    let mut rest = data;
    if !rest.starts_with("tree ") {
        bail!(ObjectError::ErrParse(String::from(
            "failed to read 'tree' type"
        )))
    }

    let rest = rest.split_off("blob ".len());
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

    let mut read_bytes = 0;
    let mut entries = Vec::<Entry>::new();
    while read_bytes < size {
        let (mode_str, content) = content
            .split_once(' ')
            .ok_or(ObjectError::ErrParse(format!(
                "failed to parse tree entry mode"
            )))?;

        read_bytes += mode_str.len() + 1;

        let mode = get_mode_from_bytes(mode_str)?;

        let (name_str, content) =
            content
                .split_once('\0')
                .ok_or(ObjectError::ErrParse(format!(
                    "failed to parse tree entry name"
                )))?;

        read_bytes += name_str.len() + 1;

        let (hash_str, content) = content.split_at(20);
        read_bytes += hash_str.len();

        entries.push(Entry {
            hash: Hash::from(hash_str),
            mode,
            name: String::from_str(name_str)?,
        });
    }

    Ok(entries)
}

fn get_mode_from_bytes(mode_str: &str) -> Result<EntryMode> {
    let mode = usize::from_str_radix(mode_str, 8)?;
    let entry_mode = match mode {
        0o40000 => EntryMode::Directory,
        0o120000 => EntryMode::SymbolicLink,
        0o100755 => EntryMode::ExecutableFile,
        0o100644 => EntryMode::RegularFile,
        _ => bail!(ObjectError::ErrInvalidMode(mode)),
    };

    Ok(entry_mode)
}
