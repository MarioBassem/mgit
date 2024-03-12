use std::{io::Write, path::PathBuf};

use anyhow::{Ok, Result};
use flate2::Compression;

pub fn hash_object(path: PathBuf, write: bool) -> Result<()> {
    let original_file_content = std::fs::read_to_string(path)?;
    let blob_content = format!(
        "blob {}\0{}",
        original_file_content.len(),
        original_file_content
    );
    let hash = hash256(&blob_content)?;
    if write {
        let compressed_data = compress(&blob_content)?;
        write_object_file(&hash, compressed_data)?;
    }

    print!("{}", hash);

    Ok(())
}

fn write_object_file(hash: &str, compressed_data: Vec<u8>) -> Result<()> {
    let (dir_name, file_name) = hash.split_at(2);
    std::fs::create_dir_all(format!(".git/objects/{}", dir_name))?;

    std::fs::write(
        format!(".git/objects/{}/{}", dir_name, file_name),
        compressed_data,
    )?;

    Ok(())
}

fn compress(data: &str) -> Result<Vec<u8>> {
    let mut writer = flate2::write::ZlibEncoder::new(Vec::new(), Compression::default());
    writer.write_all(data.as_bytes())?;
    let compressed_data = writer.finish()?;
    return Ok(compressed_data);
}

fn hash256(data: &str) -> Result<String> {
    let digest = sha256::digest(data);
    return Ok(digest);
}

#[cfg(test)]
mod test {
    use crate::hash_object::hash256;

    #[test]
    fn hash256_test() {
        let data = "hello world";
        let hashed = hash256(data).unwrap();
        assert_eq!(
            hashed,
            String::from("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9")
        );
    }
}
