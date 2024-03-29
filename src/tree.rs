use crate::hash_object::{self, write_object_file};
use anyhow::{bail, Context, Ok, Result};
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::{Bytes, Write};
use std::ops::Deref;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::{
    fmt::Display,
    fs,
    io::{BufRead, BufReader, Read},
};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
enum EntryMode {
    RegularFile = 0o100644,
    ExecutableFile = 0o100755,
    SymbolicLink = 0o120000,
    Directory = 0o40000,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Entry {
    mode: EntryMode,
    name: String,
    hash: String,
}

impl Display for EntryMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EntryMode::Directory => write!(f, "040000 dir"),
            EntryMode::ExecutableFile => write!(f, "100755 blob"),
            EntryMode::RegularFile => write!(f, "100644 blob"),
            EntryMode::SymbolicLink => write!(f, "120000 blob"),
        }
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}\t{}", self.mode, self.hash, self.name)
    }
}

pub fn ls_tree(hash: &str, name_only: bool) -> Result<()> {
    let (dir, filename) = hash.split_at(2);
    let file = fs::File::open(format!(".git/objects/{}/{}", dir, filename))?;

    let mut entries = parse_tree_file_content(file)?;
    entries.sort_by(|a, b| a.name.cmp(&b.name));

    if name_only {
        for entry in entries {
            println!("{}", entry.name)
        }

        return Ok(());
    }

    println!("{:?}", entries);

    Ok(())
}

fn parse_tree_file_content<R: Read>(r: R) -> Result<Vec<Entry>> {
    let decompressed = flate2::read::ZlibDecoder::new(r);
    let mut buffer = BufReader::new(decompressed);
    let mut tree_buff = Vec::new();

    // read "blob " from decompressed data
    buffer.read_until(b' ', &mut tree_buff)?;

    if tree_buff.deref() != "tree ".as_bytes() {
        bail!("failed to read tree");
    }

    // read content length (until null byte is reached) from decompressed data
    let mut length_buff = Vec::new();
    buffer
        .read_until(b'\0', &mut length_buff)
        .context("failed to read tree content length")?;

    let length = (std::str::from_utf8(&length_buff[..length_buff.len() - 1])?).parse::<usize>()?;

    let mut read_bytes = 0;
    let mut entries = Vec::<Entry>::new();
    while read_bytes < length {
        let mut mode_buff = Vec::new();
        buffer.read_until(b' ', &mut mode_buff)?;
        read_bytes += mode_buff.len();
        mode_buff.pop();
        let mode = get_mode_from_bytes(&mode_buff)?;

        let mut name_buff = Vec::new();
        buffer.read_until(b'\0', &mut name_buff)?;
        read_bytes += name_buff.len();
        name_buff.pop();

        let mut hash_buff = [0; 20];
        buffer.read_exact(&mut hash_buff)?;
        read_bytes += 20;

        entries.push(Entry {
            hash: String::from_utf8(hash_buff.to_vec())?,
            mode,
            name: String::from_utf8(name_buff)?,
        });
    }

    Ok(entries)
}

fn get_mode_from_bytes(mode_buff: &[u8]) -> Result<EntryMode> {
    let mode = usize::from_str_radix(std::str::from_utf8(mode_buff)?, 8)?;
    let entry_mode = match mode {
        0o40000 => EntryMode::Directory,
        0o120000 => EntryMode::SymbolicLink,
        0o100755 => EntryMode::ExecutableFile,
        0o100644 => EntryMode::RegularFile,
        _ => bail!("invalid mode {}", mode),
    };

    Ok(entry_mode)
}

/// writes a tree and returns its hash(in hexadecimal form)
pub fn write_tree() -> Result<()> {
    let entries = fs::read_dir("./")?;

    todo!()
}

fn write_tree_rec(path: PathBuf) -> Result<Vec<u8>> {
    let paths = fs::read_dir(path)?;
    let mut content = String::new();
    for path in paths {
        let entry = path?;
        if entry.file_type()?.is_dir() {
            let hash = write_tree_rec(entry.path())?;
            content = format!("{}{} {:?}\0{}", content, 40000, entry.file_name(), hash)
        } else if entry.file_type()?.is_symlink() {
            let hash = hash_object::hash_object(entry.path(), true)?;
            content = format!("{}{} {:?}\0{}", content, 120000, entry.file_name(), hash)
        } else if entry.file_type()?.is_file() {
            if entry.metadata()?.permissions().mode() & 0111 != 0 {
                // executable
                let hash = hash_object::hash_object(entry.path(), true)?;
                content = format!("{}{} {:?}\0{}", content, 100755, entry.file_name(), hash)
            } else {
                let hash = hash_object::hash_object(entry.path(), true)?;
                content = format!("{}{} {:?}\0{}", content, 100644, entry.file_name(), hash)
            }
        }
    }
    let size = content.len();
    let content = format!("tree {}\0{}", size, content);

    // compress
    let mut writer = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
    writer.write_all(content.as_bytes())?;
    let compressed = writer.finish()?;

    // write to file
    // hash
    let mut hasher = Sha1::new();
    hasher.update(compressed);
    let result = hasher.finalize();

    let hash_hex = format!("{:x}", result);
    write_object_file(&hash_hex, compressed)?;

    Ok(result[..].to_vec())
}

#[cfg(test)]
mod test {
    use std::io::{BufReader, Cursor, Write};

    use flate2::{write::ZlibEncoder, Compression};

    use super::{parse_tree_file_content, Entry};

    #[test]
    fn parse_tree_file_content_test() {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(b"tree 97\0040000 dir1\0aaaaaaaaaaaaaaaaaaaa040000 dir2\0aaaaaaaaaaaaaaaaaaaa100644 file1\0bbbbbbbbbbbbbbbbbbbb").unwrap();
        let compressed = e.finish().unwrap();
        let reader = BufReader::new(Cursor::new(compressed));

        let entries = match parse_tree_file_content(reader) {
            Ok(entries) => entries,
            Err(error) => {
                println!("error: {}", error);
                return;
            }
        };

        let want = vec![
            Entry {
                hash: String::from("aaaaaaaaaaaaaaaaaaaa"),
                name: String::from("dir1"),
                mode: super::EntryMode::Directory,
            },
            Entry {
                hash: String::from("aaaaaaaaaaaaaaaaaaaa"),
                name: String::from("dir2"),
                mode: super::EntryMode::Directory,
            },
            Entry {
                hash: String::from("bbbbbbbbbbbbbbbbbbbb"),
                name: String::from("file1"),
                mode: super::EntryMode::RegularFile,
            },
        ];

        assert_eq!(entries, want)
    }
}
