use crate::objects::{compress::compress, write_object};

use super::hash::{Hash, HashHex};
use anyhow::Result;

pub struct Author {
    name: String,
    email: String,
    time: u64,
    time_zone: String,
}

pub fn write_commit(
    tree_hash_hex: HashHex,
    parents: Vec<HashHex>,
    author: Author,
    message: String,
) -> Result<Hash> {
    // TODO: validate tree exists
    // TODO: validate parent commits exist

    // create content
    let mut content = Vec::new();
    let tree_hash = tree_hash_hex.get_hash()?;
    content.append(&mut "tree ".as_bytes().to_vec());
    content.append(&mut tree_hash.into());

    for parent in parents {
        let parent_hash = parent.get_hash()?;
        content.append(&mut "parent ".as_bytes().to_vec());
        content.append(&mut parent_hash.into());
    }

    content.append(
        &mut format!(
            "author {} <{}> {} {}",
            author.name, author.email, author.time, author.time_zone
        )
        .as_bytes()
        .to_vec(),
    );
    content.append(
        &mut format!(
            "committer {} <{}> {} {}",
            author.name, author.email, author.time, author.time_zone
        )
        .as_bytes()
        .to_vec(),
    );

    content.append(&mut "\n".as_bytes().to_vec());
    content.append(&mut message.as_bytes().to_vec());

    let mut commit_object_content = format!("commit {}\0", content.len()).as_bytes().to_vec();
    commit_object_content.append(&mut content);

    // compress
    let compressed_object = compress(commit_object_content)?;

    // write object
    let hash = write_object(compressed_object)?;

    // return hash
    Ok(hash)
}
