use anyhow::{bail, Context, Result};
use core::fmt;
use flate2;
use std::{
    fs,
    io::{BufRead, BufReader, Read},
    ops::Deref,
};

#[derive(Debug)]
enum ReadError {
    ErrNullByteNotFound,
    ErrInvalidBlobFormat(String),
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // return match self {
        //     ReadError::ErrNullByteNotFound => write!(f, "Null byte not found in blob"),
        // };
        write!(f, "{}", self)
    }
}

impl std::error::Error for ReadError {}

pub fn read_blob(hash: String) -> Result<String> {
    let (dir, filename) = hash.split_at(2);
    let file = fs::File::open(format!(".git/{}/{}", dir, filename))?;

    let mut decompressed = flate2::read::ZlibDecoder::new(file);
    let mut blob = [0; 5];
    decompressed
        .read_exact(&mut blob)
        .context("failed to read blob")?;

    let mut buffer = BufReader::new(decompressed);
    let mut read_buff = Vec::new();

    // read "blob " from decompressed data
    buffer.read_until(b' ', &mut read_buff)?;

    if read_buff.deref() != "blob ".as_bytes() {
        bail!(ReadError::ErrInvalidBlobFormat(
            "failed to read blob".to_string()
        ));
    }

    // read content length (until null byte is reached) from decompressed data
    buffer
        .read_until(b'\0', &mut read_buff)
        .context("failed to read blob length")?;
    let length =
        usize::from_str_radix(std::str::from_utf8(&read_buff[..read_buff.len() - 1])?, 10)?;

    buffer.read_to_end(&mut read_buff)?;

    if read_buff.len() != length {
        bail!(
            "incorrect blob content length: expected {}, found {}",
            length,
            read_buff.len()
        )
    }

    return Ok(String::from_utf8(read_buff)?);
}

#[cfg(test)]
mod test {}
