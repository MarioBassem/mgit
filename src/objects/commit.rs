use core::time;
use std::io::Read;

use crate::objects::{compress::compress, write_object};

use super::{
    hash::{Hash, HashHex},
    Object,
};
use anyhow::{anyhow, Result};
use bytes::Bytes;

pub struct Commit {
    tree: HashHex,
    author: Author,
    committer: Author,
    parents: Vec<HashHex>,
    message: String,
    additional_data: Option<String>,
}

/*
    tree hash_hex LF
    parent hash_hex LF
    author Author LF
    committer Author LF
    gpgsig signature LF LF
    commit_message LF
*/

pub struct Author {
    name: String,
    email: String,
    time: u64,
    time_zone: String,
}

pub fn new_commit(
    tree: HashHex,
    parents: Vec<HashHex>,
    author: Author,
    committer: Option<Author>,
    signature: Option<String>,
    message: Option<String>,
) -> Object {
    todo!()
}

pub fn decode_commit(mut data: Vec<u8>) -> Result<Commit> {
    let tree = get_tree_hash_hex(&mut data)?;
    let parents = get_commit_parents(&mut data)?;
    let author = get_author(&mut data)?;
    let committer = get_committer(&mut data)?;
    let message = get_commit_message(&mut data)?;
    let additional_data = get_additional_data(&mut data);

    Ok(Commit {
        author,
        committer,
        message,
        parents,
        tree,
        additional_data,
    })
}

pub fn encode_commit(commit: Commit) -> Vec<u8> {
    todo!()
}

fn get_tree_hash_hex(data: &mut Vec<u8>) -> Result<HashHex> {
    /*
        tree SP hash_hex LF
    */
    let x = Bytes::new();

    let tree_str = data.split_off(5);
    if tree_str.as_slice() != "tree ".as_bytes() {
        return Err(anyhow!("missing tree hash"));
    }

    let hash_hex = data.split_off(40);
    let lf = data.split_off(1);
    if lf.as_slice() != "\n".as_bytes() {
        return Err(anyhow!("invalid tree hash line"));
    }

    Ok(HashHex(String::from_utf8(hash_hex)?))
}

fn get_commit_parents(data: &mut Vec<u8>) -> Result<Vec<HashHex>> {
    let mut parents = Vec::new();
    loop {
        match data.first_chunk::<7>() {
            None => break,
            Some(d) => {
                if d != "parent ".as_bytes() {
                    break;
                }
                data.split_off(7);
            }
        }

        let hash_hex = data.split_off(40);
        let lf = data.split_off(1);
        if lf.as_slice() != "\n".as_bytes() {
            return Err(anyhow!("invalid tree hash line"));
        }

        parents.push(HashHex(String::from_utf8(hash_hex)?))
    }

    Ok(parents)
}

fn get_author(data: &mut Vec<u8>) -> Result<Author> {
    let author_str = data.split_off(7);
    if author_str.as_slice() != "author ".as_bytes() {
        return Err(anyhow!("missing author"));
    }

    let mut author_data = Vec::new();
    for _ in 0..4 {
        for i in 0..data.len() {
            if (i < 3 && data[i] == b' ') || (i == 3 && data[i] == b'\n') {
                author_data.push(data.split_off(i + 1));
                break;
            }
        }
    }

    if author_data.len() != 4 {
        return Err(anyhow!("invalid author data"));
    }

    Ok(Author {
        name: String::from_utf8(author_data[0])?,
        email: String::from_utf8(author_data[1])?,
        time: u64::from_str_radix(String::from_utf8(author_data[2])?.as_str(), 10)?,
        time_zone: String::from_utf8(author_data[3])?,
    })
}

fn get_committer(data: &mut Vec<u8>) -> Result<Author> {
    let committer_str = data.split_off(7);
    if committer_str.as_slice() != "committer ".as_bytes() {
        return Err(anyhow!("missing committer"));
    }

    let mut committer_data = Vec::new();
    for _ in 0..4 {
        for i in 0..data.len() {
            if (i < 3 && data[i] == b' ') || (i == 3 && data[i] == b'\n') {
                committer_data.push(data.split_off(i));
                get_char(data, data[i])?;
                break;
            }
        }
    }

    if committer_data.len() != 4 {
        return Err(anyhow!("invalid committer data"));
    }

    Ok(Author {
        name: String::from_utf8(committer_data[0])?,
        email: String::from_utf8(committer_data[1])?,
        time: u64::from_str_radix(String::from_utf8(committer_data[2])?.as_str(), 10)?,
        time_zone: String::from_utf8(committer_data[3])?,
    })
}

fn get_signature(data: &mut Vec<u8>) -> Result<Option<String>> {
    match data.first_chunk::<7>() {
        None => return Ok(None),
        Some(d) => {
            if d != "gpgsig ".as_bytes() {
                return Ok(None);
            }
            data.split_off(7);
        }
    }

    let mut sig = String::new();
    for i in 0..data.len() {
        if data[i] == b'\n' {
            sig = String::from_utf8(data.split_off(i))?;
            get_char(data, b'\n')?;
            break;
        }
    }

    Ok(Some(sig))
}

fn get_char(data: &mut Vec<u8>, b: u8) -> Result<()> {
    if data.len() == 0 {
        return Err(anyhow!("failed to read SP"));
    }

    if data[0] != b {
        return Err(anyhow!("expected SP"));
    }

    data.split_off(1);

    Ok(())
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
