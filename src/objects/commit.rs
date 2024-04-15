use super::Object;
use crate::objects::Hash;
use anyhow::{anyhow, Result};
use std::{fmt::Display, io::BufRead};

#[derive(Debug)]
pub struct Commit {
    tree: Hash,
    author: Author,
    committer: Author,
    parents: Vec<Hash>,
    message: String,
    additional_data: Option<String>,
}

impl Display for Commit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parents = String::new();
        for parent in &self.parents {
            parents = format!("{}parent {:x}\n", parents, parent);
        }

        write!(
            f,
            "tree {:x}\nauthor {}\ncommitter {}\n{}message {}",
            self.tree, self.author, self.committer, parents, self.message
        )
    }
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

#[derive(Debug, Clone)]
pub struct Author {
    pub name: String,
    pub email: String,
    pub time: u64,
    pub time_zone: String,
}

impl Display for Author {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} <{}> {} {}",
            self.name, self.email, self.time, self.time_zone
        )
    }
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

pub fn decode_commit(data: Vec<u8>) -> Result<Commit> {
    let mut tree: Option<Hash> = None;
    let mut parents = Vec::new();
    let mut author: Option<Author> = None;
    let mut committer: Option<Author> = None;
    let mut additional_data: Option<String> = None;
    let mut message = String::new();
    let lines: Vec<String> = data.lines().collect::<Result<_, _>>()?;
    for (i, line) in lines.iter().enumerate() {
        if line.len() == 0 {
            // next is commit message
            if i != lines.len() - 2 {
                return Err(anyhow!("invalid commit data"));
            }

            message = lines[i + 1].clone();
            break;
        }

        let (first_word, words) = line.split_once(' ').ok_or(anyhow!("invalid tag data"))?;
        match first_word {
            "tree" => tree = Some(Hash::try_from(words.as_bytes())?),
            "parent" => parents.push(Hash::try_from(words.as_bytes())?),
            "author" => author = Some(Author::try_from(words)?),
            "committer" => committer = Some(Author::try_from(words)?),
            _ => additional_data = Some(line.to_string()),
        }
    }

    Ok(Commit {
        tree: tree.ok_or(anyhow!("commit missing tree information"))?,
        author: author.ok_or(anyhow!("commit missing author information"))?,
        committer: committer.ok_or(anyhow!("commit missing committer information"))?,
        parents,
        message,
        additional_data,
    })
}

pub fn encode_commit(commit: Commit) -> Result<Vec<u8>> {
    let mut content = Vec::new();

    content.append(&mut format!("tree {:x}\n", commit.tree).into_bytes());

    for parent in commit.parents {
        content.append(&mut format!("parent {:x}\n", parent).into_bytes());
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

#[cfg(test)]
mod test {
    use crate::objects::hash::Hash;

    use super::{decode_commit, encode_commit, Author, Commit};

    #[test]
    fn test_decode_commit() {
        let hash1 = (0..40).map(|_| 'a').collect::<String>();
        let hash2 = (0..40).map(|_| 'b').collect::<String>();
        let author = Author {
            email: String::from("m@mail.com"),
            name: String::from("name"),
            time: 1,
            time_zone: String::from("+0200"),
        };

        let data = format!("tree {}\nparent {}\nparent {}\nauthor {} <{}> {} {}\ncommitter {} <{}> {} {}\ngpgsig my_signature\n\ncommit message\n", hash1, hash1, hash2, author.name, author.email, author.time, author.time_zone, author.name, author.email, author.time, author.time_zone);
        let commit = decode_commit(data.into_bytes()).unwrap();
        assert_eq!(commit.tree, Hash::try_from(hash1.as_bytes()).unwrap());

        assert_eq!(
            commit.parents,
            vec![
                Hash::try_from(hash1.as_bytes()).unwrap(),
                Hash::try_from(hash2.as_bytes()).unwrap()
            ]
        );

        assert_eq!(commit.author.name, author.name);
        assert_eq!(commit.author.email, author.email);
        assert_eq!(commit.author.time, author.time);
        assert_eq!(commit.author.time_zone, author.time_zone);

        assert_eq!(commit.committer.name, author.name);
        assert_eq!(commit.committer.email, author.email);
        assert_eq!(commit.committer.time, author.time);
        assert_eq!(commit.committer.time_zone, author.time_zone);

        assert_eq!(
            commit.additional_data,
            Some(String::from("gpgsig my_signature"))
        );

        assert_eq!(commit.message, String::from("commit message"));
    }

    #[test]
    fn test_encode_commit() {
        let hash1 = (0..40).map(|_| 'a').collect::<String>();
        let hash2 = (0..40).map(|_| 'b').collect::<String>();
        let author = Author {
            email: String::from("m@mail.com"),
            name: String::from("name"),
            time: 1,
            time_zone: String::from("+0200"),
        };

        let commit = Commit {
            additional_data: Some(String::from("gpgsig my_signature")),
            author: author.clone(),
            committer: author.clone(),
            message: String::from("commit message"),
            parents: vec![
                Hash::try_from(hash1.as_bytes()).unwrap(),
                Hash::try_from(hash2.as_bytes()).unwrap(),
            ],
            tree: Hash::try_from(hash1.as_bytes()).unwrap(),
        };

        let data = encode_commit(commit).unwrap();
        assert_eq!(data, format!("tree {}\nparent {}\nparent {}\nauthor {} <{}> {} {}\ncommitter {} <{}> {} {}\ngpgsig my_signature\n\ncommit message\n", hash1, hash1, hash2, author.name, author.email, author.time, author.time_zone, author.name, author.email, author.time, author.time_zone).into_bytes());
    }
}
