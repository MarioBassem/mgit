use std::fmt::{Debug, LowerHex};

use anyhow::{anyhow, Ok, Result};
use sha1::{Digest, Sha1};

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) struct Hash(Vec<u8>);

/// hash data using sha1
pub fn hash(data: &[u8]) -> Hash {
    let mut hash = Sha1::new();
    hash.update(data);
    let digest = hash.finalize();

    Hash(digest[..].to_vec())
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

impl TryFrom<&[u8]> for Hash {
    type Error = anyhow::Error;
    fn try_from(value: &[u8]) -> std::prelude::v1::Result<Self, Self::Error> {
        let hash = match value.len() {
            20 => Hash(value.to_vec()),
            40 => {
                let hash = String::from_utf8(value.to_vec())?;
                let x: Result<Vec<u8>> = (0..hash.len())
                    .step_by(2)
                    .map(|i| -> Result<u8> { Ok(u8::from_str_radix(&hash[i..i + 2], 16)?) })
                    .collect();

                Hash(x?)
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

// pub struct HashHex(String);

// impl From<&Hash> for HashHex {
//     fn from(value: &Hash) -> Self {
//         HashHex(format!("{:x}", value))
//     }
// }

// impl HashHex {
// pub fn get_object_path(&self) -> (&str, &str) {
//     self.0.split_at(2)
// }

//     pub fn get_hash(&self) -> Result<Hash> {
// let x: Result<Vec<u8>> = (0..self.0.len())
//     .step_by(2)
//     .map(|i| -> Result<u8> { Ok(u8::from_str_radix(&self.0[i..i + 2], 16)?) })
//     .collect();

// Ok(Hash(x?))
//     }
// }
