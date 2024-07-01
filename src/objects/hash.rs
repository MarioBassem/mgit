use std::fmt::{Debug, LowerHex};

use anyhow::{anyhow, Ok};
use hex;
use sha1::{Digest, Sha1};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd, Clone, Hash)]
pub(crate) struct Hash(pub Vec<u8>);

/// hash data using sha1
pub fn hash(data: &[u8]) -> Hash {
    let mut hash = Sha1::new();
    hash.update(data);
    let digest = hash.finalize();

    Hash(digest[..].to_vec())
}

impl LowerHex for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl Into<Vec<u8>> for Hash {
    fn into(self) -> Vec<u8> {
        self.0
    }
}

impl TryFrom<&[u8]> for Hash {
    type Error = anyhow::Error;
    fn try_from(value: &[u8]) -> std::prelude::v1::Result<Self, Self::Error> {
        let hash = match value.len() {
            20 => Hash(value.to_vec()),
            40 => {
                let hash = hex::decode(value)?;
                Hash(hash)
            }
            _ => return Err(anyhow!("invalid hash length {}", value.len())),
        };
        Ok(hash)
    }
}

impl Hash {
    pub fn to_hex(&self) -> String {
        format!("{:x}", self)
    }

    pub fn get_object_path(&self) -> (String, String) {
        let hash_hex = format!("{:x}", self);
        let (l, r) = hash_hex.split_at(2);
        (l.to_string(), r.to_string())
    }
}
