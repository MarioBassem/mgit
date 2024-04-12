use anyhow::{Ok, Result};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use std::io::{Read, Write};

/// compress content
pub fn compress(content: &Vec<u8>) -> Result<Vec<u8>> {
    let mut writer = ZlibEncoder::new(Vec::new(), Compression::default());
    writer.write_all(&content)?;
    let compressed_data = writer.finish()?;
    Ok(compressed_data)
}

/// decompress content
pub fn decompress<R: Read>(data: R) -> Result<String> {
    let mut decompressed = ZlibDecoder::new(data);
    let mut ret = String::new();
    decompressed.read_to_string(&mut ret)?;

    Ok(ret)
}
