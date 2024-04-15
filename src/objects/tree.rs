use super::{hash::Hash, Object, ObjectError};
use anyhow::{anyhow, bail, Ok, Result};
use std::{
    fmt::Display,
    fs::{self, DirEntry},
    os::unix::fs::PermissionsExt,
    str::FromStr,
};

#[derive(Debug)]
pub struct Tree {
    entries: Vec<Entry>,
}

impl Display for Tree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut entries = String::new();
        for entry in &self.entries {
            entries = format!("{}{}\n", entries, entry)
        }

        write!(f, "{}", entries)
    }
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
        let tree_entry_info = String::from_utf8(data[..null_byte_index].to_vec())?;
        data.drain(0..null_byte_index + 1);
        let hash = data.drain(0..20).collect::<Vec<u8>>();

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
        data.append(&mut format!("{} {}\0", entry.mode, entry.name).into_bytes());
        data.append(&mut entry.hash.into())
    }

    data
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entry {
    mode: EntryMode,
    name: String,
    hash: Hash,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {:x}", self.mode, self.name, self.hash)
    }
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
            EntryMode::Directory => write!(f, "40000"),
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

#[cfg(test)]
mod test {
    use crate::objects::{
        hash::Hash,
        tree::{Entry, EntryMode},
    };

    use super::{decode_tree, encode_tree, Tree};

    #[test]
    fn test_decode_tree() {
        /*
            mode name\0hash(20-byte)
        */
        let hash1 = Hash::try_from((0..40).map(|_| 'a').collect::<String>().as_bytes()).unwrap();
        let hash2 = Hash::try_from((0..40).map(|_| 'b').collect::<String>().as_bytes()).unwrap();
        let hash3 = Hash::try_from((0..40).map(|_| 'c').collect::<String>().as_bytes()).unwrap();
        let mut data = Vec::new();
        data.append(&mut format!("40000 dir1\0").into_bytes());
        data.append(&mut hash1.clone().into());
        data.append(&mut format!("120000 symlink1\0").into_bytes());
        data.append(&mut hash2.clone().into());
        data.append(&mut format!("100644 regfile1\0").into_bytes());
        data.append(&mut hash3.clone().into());

        let tree = decode_tree(data).unwrap();
        assert_eq!(
            tree.entries,
            vec![
                Entry {
                    hash: hash1,
                    mode: EntryMode::Directory,
                    name: String::from("dir1")
                },
                Entry {
                    hash: hash2,
                    mode: EntryMode::SymbolicLink,
                    name: String::from("symlink1")
                },
                Entry {
                    hash: hash3,
                    mode: EntryMode::RegularFile,
                    name: String::from("regfile1")
                }
            ]
        );
    }

    #[test]
    fn test_encode_tree() {
        let hash1 = Hash::try_from((0..40).map(|_| 'a').collect::<String>().as_bytes()).unwrap();
        let hash2 = Hash::try_from((0..40).map(|_| 'b').collect::<String>().as_bytes()).unwrap();
        let hash3 = Hash::try_from((0..40).map(|_| 'c').collect::<String>().as_bytes()).unwrap();

        let tree = Tree {
            entries: vec![
                Entry {
                    hash: hash1.clone(),
                    mode: EntryMode::Directory,
                    name: String::from("dir1"),
                },
                Entry {
                    hash: hash2.clone(),
                    mode: EntryMode::SymbolicLink,
                    name: String::from("symlink1"),
                },
                Entry {
                    hash: hash3.clone(),
                    mode: EntryMode::RegularFile,
                    name: String::from("regfile1"),
                },
            ],
        };

        let mut want = Vec::new();
        want.append(&mut format!("40000 dir1\0").into_bytes());
        want.append(&mut hash1.clone().into());
        want.append(&mut format!("120000 symlink1\0").into_bytes());
        want.append(&mut hash2.clone().into());
        want.append(&mut format!("100644 regfile1\0").into_bytes());
        want.append(&mut hash3.clone().into());

        let data = encode_tree(tree);
        assert_eq!(data, want);
    }
}
