use super::{hash::Hash, Object, ObjectError};
use anyhow::{anyhow, bail, Ok, Result};
use std::{
    fmt::Display,
    fs::{self, DirEntry},
    os::unix::fs::PermissionsExt,
    str::FromStr,
};

pub struct Tree {
    entries: Vec<Entry>,
}

pub fn new_tree(entries: Vec<Entry>) -> Tree {
    todo!()
}

pub fn decode_tree(mut data: Vec<u8>) -> Result<Tree> {
    let mut entries = Vec::<Entry>::new();
    while data.len() > 0 {
        let null_byte_index = data
            .iter()
            .position(|c| *c == b'\0')
            .ok_or(anyhow!("invalid tree entry data"))?;
        let mut tree_entry_info = String::from_utf8(data[..null_byte_index].to_vec())?;
        data.drain(0..null_byte_index + 1);
        let mut hash = data.split_off(20);

        let (mode_str, name_str) = tree_entry_info
            .split_once(' ')
            .ok_or(anyhow!("invalid tree entry information"))?;
        let mode = EntryMode::try_from(mode_str)?;

        entries.push(Entry {
            hash: Hash::try_from(hash.as_ref())?,
            mode,
            name: String::from_str(name_str)?,
        });
    }

    Ok(Tree { entries })
}

pub fn encode_tree(tree: Tree) -> Vec<u8> {
    let mut data = Vec::new();
    for entry in tree.entries {
        data.append(&mut format!("{} {}\0{:x}", entry.mode, entry.name, entry.hash).into_bytes());
    }

    data
}

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
        let file = fs::File::open(dir_entry.path())?;
        let object = Object::read(file)?;
        let hash = object.hash()?;

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

impl TryFrom<&str> for EntryMode {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        let mode = usize::from_str_radix(value, 8)?;
        let entry_mode = match mode {
            0o40000 => EntryMode::Directory,
            0o120000 => EntryMode::SymbolicLink,
            0o100755 => EntryMode::ExecutableFile,
            0o100644 => EntryMode::RegularFile,
            _ => bail!(ObjectError::ErrInvalidMode(mode)),
        };

        Ok(entry_mode)
    }
}
