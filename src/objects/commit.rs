use super::Object;
use crate::objects::Hash;
use anyhow::{anyhow, Result};

pub struct Commit {
    tree: Hash,
    author: Author,
    committer: Author,
    parents: Vec<Hash>,
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
    pub name: String,
    pub email: String,
    pub time: u64,
    pub time_zone: String,
}

impl TryFrom<&str> for Author {
    type Error = anyhow::Error;
    fn try_from(value: &str) -> std::prelude::v1::Result<Self, Self::Error> {
        let mut words = value.split_whitespace();

        let name = words.next().ok_or(anyhow!("invalid author data"))?;
        let email = words
            .next()
            .ok_or(anyhow!("invalid author data"))?
            .trim_matches(|c| c == '<' || c == '>');
        let time = u64::from_str_radix(words.next().ok_or(anyhow!("invalid author data"))?, 10)?;
        let time_zone = words.next().ok_or(anyhow!("invalid author data"))?;

        Ok(Author {
            name: String::from(name),
            email: String::from(email),
            time,
            time_zone: String::from(time_zone),
        })
    }
}

pub fn new_commit(
    tree: Hash,
    parents: Vec<Hash>,
    author: Author,
    committer: Option<Author>,
    signature: Option<String>,
    message: Option<String>,
) -> Object {
    todo!()
}

pub fn decode_commit(mut data: Vec<u8>) -> Result<Commit> {
    let tree = get_tree_hash(&mut data)?;
    let parents = get_commit_parents(&mut data)?;
    let author = get_author(&mut data)?;
    let committer = get_committer(&mut data)?;
    let additional_data = get_additional_data(&mut data)?;
    let message = get_commit_message(&mut data)?;

    Ok(Commit {
        author,
        committer,
        message,
        parents,
        tree,
        additional_data,
    })
}

pub fn encode_commit(commit: Commit) -> Result<Vec<u8>> {
    let mut content = Vec::new();

    content.append(&mut format!("tree {:x}\n", commit.tree).into_bytes());

    for parent in commit.parents {
        content.append(&mut "parent ".as_bytes().to_vec());
        content.append(&mut parent.into());
        content.append(&mut "\n".as_bytes().to_vec());
    }

    content.append(
        &mut format!(
            "author {} <{}> {} {}\n",
            commit.author.name, commit.author.email, commit.author.time, commit.author.time_zone
        )
        .into_bytes(),
    );
    content.append(
        &mut format!(
            "committer {} <{}> {} {}\n",
            commit.committer.name,
            commit.committer.email,
            commit.committer.time,
            commit.committer.time_zone
        )
        .into_bytes(),
    );

    if let Some(additional_data) = commit.additional_data {
        content.append(&mut format!("{}\n", additional_data).into_bytes())
    }

    content.append(&mut "\n".as_bytes().to_vec());
    content.append(&mut commit.message.into_bytes());
    content.append(&mut "\n".as_bytes().to_vec());

    Ok(content)
}

fn get_tree_hash(data: &mut Vec<u8>) -> Result<Hash> {
    /*
        tree SP hash_hex LF
    */
    let tree_str = data.split_off(5);
    if tree_str.as_slice() != "tree ".as_bytes() {
        return Err(anyhow!("missing tree hash"));
    }

    let hash_hex = data.split_off(40);
    let lf = data.split_off(1);
    if lf.as_slice() != "\n".as_bytes() {
        return Err(anyhow!("invalid tree hash line"));
    }

    Ok(Hash::try_from(hash_hex.as_ref())?)
}

fn get_commit_parents(data: &mut Vec<u8>) -> Result<Vec<Hash>> {
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

        parents.push(Hash::try_from(hash_hex.as_ref())?)
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

fn get_additional_data(data: &mut Vec<u8>) -> Result<Option<String>> {
    /*
        read until LF LF is reached
    */

    let mut additional_data: Option<String> = None;
    for i in 0..data.len() {
        if i < data.len() - 1 && data[i] == b'\n' && data[i + 1] == b'\n' {
            additional_data = Some(String::from_utf8(data.split_off(i))?);
            data.split_off(2);
            break;
        }
    }

    Ok(additional_data)
}

fn get_commit_message(data: &mut Vec<u8>) -> Result<String> {
    // TODO: message data should be escaped
    let message = String::from_utf8(data.to_vec())?;
    Ok(message)
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
