use super::hash::{Hash, HashHex};
use anyhow::Result;

pub fn write_commit(
    tree_hash: HashHex,
    parents: Vec<HashHex>,
    message: String,
    author: String,
) -> Result<Hash> {
    // validate tree exists
    // validate parent commits exist
    // create content
    // compress
    // write object
    // return hash
    todo!()
}
