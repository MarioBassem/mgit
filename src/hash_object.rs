use std::{io::Write, path::PathBuf};

use anyhow::{Ok, Result};
use flate2::Compression;
use sha1::{Digest, Sha1};

/// reads a file and generates its hash
pub fn hash_object(path: PathBuf, write: bool) -> Result<()> {
    let original_file_content = std::fs::read_to_string(path)?;
    let blob_content = format!(
        "blob {}\0{}",
        original_file_content.len(),
        original_file_content
    );

    let mut hasher = Sha1::new();
    hasher.update(blob_content);
    let hash = hasher.finalize();
    let hash_hex = format!("{:x}", hash);

    if write {
        let compressed_data = compress(&blob_content)?;
        write_object_file(&hash_hex, compressed_data)?;
    }

    print!("{}", hash_hex);

    Ok(())
}

/// writes compressed data to object file
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
    Ok(compressed_data)
}

fn hash(data: &str) -> Result<Vec<u8>> {
    let mut hash = Sha1::new();
    hash.update(data);
    let digest = hash.finalize();

    Ok(digest[..].to_vec())
}

#[cfg(test)]
mod test {
    use crate::hash_object::hash;
    use sha1::{Digest, Sha1};
    #[test]
    fn hash_test() {
        let data = "hello world";
        let mut hasher = Sha1::new();
        hasher.update(data);
        let hash = hasher.finalize();
        let hash_hex = format!("{:x}", hash);
        // let hashed = hash(data).unwrap();
        assert_eq!(
            hash_hex,
            String::from("b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9")
        );
    }
}
