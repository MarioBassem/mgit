use anyhow::{bail, Context, Result};
use core::fmt;
use flate2;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
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

fn extract_blob_content<R: Read>(r: R) -> Result<String> {
    let decompressed = flate2::read::ZlibDecoder::new(r);
    let mut buffer = BufReader::new(decompressed);
    let mut blob_buff = Vec::new();

    // read "blob " from decompressed data
    buffer.read_until(b' ', &mut blob_buff)?;

    if blob_buff.deref() != "blob ".as_bytes() {
        bail!(ReadError::ErrInvalidBlobFormat(
            "failed to read blob".to_string()
        ));
    }

    // read content length (until null byte is reached) from decompressed data
    let mut length_buff = Vec::new();
    buffer
        .read_until(b'\0', &mut length_buff)
        .context("failed to read blob length")?;

    let length = usize::from_str_radix(
        std::str::from_utf8(&length_buff[..length_buff.len() - 1])?,
        10,
    )?;

    let mut content_buff = Vec::new();
    buffer.read_to_end(&mut content_buff)?;

    if content_buff.len() != length {
        bail!(
            "incorrect blob content length: expected {}, found {}",
            length,
            content_buff.len()
        )
    }

    return Ok(String::from_utf8(content_buff)?);
}

/// read_blob reads object content from file, decompresses it, then prints it to standard output
pub fn read_blob(hash: String) -> Result<()> {
    let (dir, filename) = hash.split_at(2);
    let file = fs::File::open(format!(".git/{}/{}", dir, filename))?;
    let content = extract_blob_content(file)?;
    print!("{}", content);

    Ok(())
}

#[cfg(test)]
mod test {
    use std::io::{BufReader, Cursor, Write};

    use flate2::{write::ZlibEncoder, Compression};

    use super::extract_blob_content;

    #[test]
    fn extract_blob_test() {
        let mut e = ZlibEncoder::new(Vec::new(), Compression::default());
        e.write_all(b"blob 11\0hello world").unwrap();
        let compressed = e.finish().unwrap();
        let reader = BufReader::new(Cursor::new(compressed));
        let res = extract_blob_content(reader).unwrap();

        assert_eq!(res, String::from("hello world"))
    }
}
