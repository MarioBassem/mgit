use super::Object;
use crate::objects::Hash;
use anyhow::{anyhow, Result};
use std::io::BufRead;

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
    additional_data LF
    LF
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
    let mut commit: Commit;
    let lines: Vec<String> = data.lines().collect::<Result<_, _>>()?;
    for (i, line) in lines.iter().enumerate() {
        if line.len() == 0 {
            // next is commit message
            if i != lines.len() - 2 {
                return Err(anyhow!("invalid commit data"));
            }

            commit.message = lines[i + 1];
            break;
        }

        let (first_word, words) = line.split_once(' ').ok_or(anyhow!("invalid tag data"))?;
        match first_word {
            "tree" => commit.tree = Hash::try_from(words.as_bytes())?,
            "parent" => commit.parents.push(Hash::try_from(words.as_bytes())?),
            "author" => commit.author = Author::try_from(words)?,
            "committer" => commit.committer = Author::try_from(words)?,
            _ => commit.additional_data = Some(line.to_string()),
        }
    }

    Ok(commit)
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
