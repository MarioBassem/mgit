use std::ops::Deref;
use std::str::FromStr;
use std::{
    fmt::{write, Display},
    fs,
    io::{BufRead, BufReader, Read},
};

use anyhow::{bail, Context, Ok, Result};

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

    print_mode: EntryPrintMode,
}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
enum EntryPrintMode {
    NameOnly,
    Full,
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
        match self.print_mode {
            EntryPrintMode::Full => {
                write!(f, "{} {}\t{}", self.mode, self.hash, self.name)
            }
            EntryPrintMode::NameOnly => {
                write!(f, "{}", self.name)
            }
        }
    }
}

pub fn ls_tree(hash: &str, name_only: bool) -> Result<()> {
    let (dir, filename) = hash.split_at(2);
    let file = fs::File::open(format!(".git/objects/{}/{}", dir, filename))?;

    let mut entries = parse_tree_file_content(file)?;
    entries.sort_by(|a, b| a.name.cmp(&b.name));

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

    let length = usize::from_str_radix(
        std::str::from_utf8(&length_buff[..length_buff.len() - 1])?,
        10,
    )?;

    let mut read_bytes = 0;
    let mut entries = Vec::<Entry>::new();
    while read_bytes < length {
        let mut mode_buff = Vec::new();
        buffer.read_until(b' ', &mut mode_buff)?;
        let mode = get_mode_from_bytes(&mode_buff)?;
        read_bytes += mode_buff.len();
        mode_buff.pop();

        let mut name_buff = Vec::new();
        buffer.read_until(b'\0', &mut name_buff)?;
        read_bytes += name_buff.len();
        name_buff.pop();

        let mut hash_buff = Vec::with_capacity(20);
        buffer.read_exact(&mut hash_buff)?;
        read_bytes += 20;

        entries.push(Entry {
            hash: String::from_utf8(hash_buff)?,
            mode: mode,
            name: String::from_utf8(name_buff)?,
            print_mode: EntryPrintMode::NameOnly,
        });
    }

    return Ok(entries);
}

fn get_mode_from_bytes(mode_buff: &Vec<u8>) -> Result<EntryMode> {
    let mode = usize::from_str_radix(std::str::from_utf8(&mode_buff)?, 8)?;
    let entry_mode = match mode {
        0o40000 => EntryMode::Directory,
        0o120000 => EntryMode::SymbolicLink,
        0o100755 => EntryMode::ExecutableFile,
        0o100644 => EntryMode::RegularFile,
        _ => bail!("invalid mode {}", mode),
    };

    Ok(entry_mode)
}
