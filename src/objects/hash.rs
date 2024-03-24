use std::fmt::LowerHex;

use anyhow::{Ok, Result};
use sha1::{Digest, Sha1};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Hash(Vec<u8>);

/// hash data using sha1
pub fn hash(data: &[u8]) -> Result<Hash> {
    let mut hash = Sha1::new();
    hash.update(data);
    let digest = hash.finalize();

    Ok(Hash(digest[..].to_vec()))
}

impl LowerHex for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:02x?}", self.0)
    }
}

impl Into<Vec<u8>> for Hash {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl From<&str> for Hash {
    fn from(value: &str) -> Self {
        Hash(value.as_bytes().to_vec())
    }
}

pub struct HashHex(String);

impl From<Hash> for HashHex {
    fn from(value: Hash) -> Self {
        HashHex(format!("{:x}", value))
    }
}

impl HashHex {
    pub fn get_object_path(&self) -> (&str, &str) {
        self.0.split_at(2)
    }
}
