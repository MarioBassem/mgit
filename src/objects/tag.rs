use std::{fmt::Display, io::BufRead};

use super::{commit::Author, hash::Hash, Object, ObjectKind};
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Tag {
    object: Hash,
    object_type: ObjectKind,
    tag_name: String,
    tagger: Author,
    commit_message: String,
    additional_data: Option<String>,
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "object {:x}\nobject_type {}\ntag {}\ntagger {}\n\n{}",
            self.object, self.object_type, self.tag_name, self.tagger, self.commit_message
        )
    }
}

pub fn new_tag(
    object: Hash,
    object_type: ObjectKind,
    tag_name: String,
    tagger: Author,
    commit_message: Option<String>,
    additional_data: Option<String>,
) -> Object {
    todo!()
}

/*
    format:
        tag size NUL
        object object_hex_hash LF
        type object_type LF
        tag tag_name LF
        tagger author LF
        additional_data LF
        LF
        commit_message LF

*/

pub fn decode_tag(data: Vec<u8>) -> Result<Tag> {
    let mut object: Option<Hash> = None;
    let mut object_type: Option<ObjectKind> = None;
    let mut tag_name: Option<String> = None;
    let mut tagger: Option<Author> = None;
    let mut additional_data: Option<String> = None;
    let mut commit_message: String = String::new();

    let lines: Vec<String> = data.lines().collect::<Result<_, _>>()?;
    for (i, line) in lines.iter().enumerate() {
        if line.len() == 0 {
            // next is commit message
            if i != lines.len() - 2 {
                return Err(anyhow!("invalid tag data"));
            }

            commit_message = lines[i + 1].clone();
            break;
        }

        let (first_word, words) = line.split_once(' ').ok_or(anyhow!("invalid tag data"))?;
        match first_word {
            "object" => object = Some(Hash::try_from(words.as_bytes())?),
            "type" => object_type = Some(ObjectKind::try_from(words)?),
            "tag" => tag_name = Some(String::from(words)),
            "tagger" => tagger = Some(Author::try_from(words)?),
            _ => additional_data = Some(line.to_string()),
        }
    }

    Ok(Tag {
        object: object.ok_or(anyhow!("tag missing object information"))?,
        object_type: object_type.ok_or(anyhow!("tag missing object type information"))?,
        tag_name: tag_name.ok_or(anyhow!("tag missing tag name information"))?,
        tagger: tagger.ok_or(anyhow!("tag missing tagger information"))?,
        commit_message,
        additional_data,
    })
}

pub fn encode_tag(tag: Tag) -> Vec<u8> {
    let mut content = Vec::new();

    content.append(&mut format!("object {:x}\n", tag.object).into_bytes());

    content.append(&mut format!("type {}\n", tag.object_type).into_bytes());

    content.append(&mut format!("tag {}\n", tag.tag_name).into_bytes());

    content.append(
        &mut format!(
            "tagger {} <{}> {} {}\n",
            tag.tagger.name, tag.tagger.email, tag.tagger.time, tag.tagger.time_zone
        )
        .into_bytes(),
    );

    if let Some(additional_data) = tag.additional_data {
        content.append(&mut format!("{}\n", additional_data).into_bytes());
    }

    content.append(&mut "\n".as_bytes().to_vec());
    content.append(&mut format!("{}\n", tag.commit_message).into_bytes());

    content
}

#[cfg(test)]

mod test {
    use crate::objects::{commit::Author, hash::Hash, ObjectKind};

    use super::{decode_tag, encode_tag, Tag};

    #[test]
    fn test_decode_tag() {
        /*
            format:
                object object_hex_hash LF
                type object_type LF
                tag tag_name LF
                tagger author LF
                additional_data LF
                LF
                commit_message LF

        */
        let hash = (0..40).map(|_| 'a').collect::<String>();
        let obj_type = String::from("commit");
        let tag_name = String::from("v1.2");
        let tagger = Author {
            email: String::from("m@m.com"),
            name: String::from("name"),
            time: 1,
            time_zone: String::from("+0200"),
        };
        let data = format!(
            "object {}\ntype {}\ntag {}\ntagger {} <{}> {} {}\ngpgsig mysig\n\nmy message\n",
            hash, obj_type, tag_name, tagger.name, tagger.email, tagger.time, tagger.time_zone
        );

        let tag = decode_tag(data.into_bytes()).unwrap();

        assert_eq!(tag.additional_data, Some(String::from("gpgsig mysig")));
        assert_eq!(tag.commit_message, String::from("my message"));
        assert_eq!(tag.object, Hash::try_from(hash.as_bytes()).unwrap());
        assert_eq!(tag.object_type.to_string(), ObjectKind::Commit.to_string());

        assert_eq!(tag.tagger.name, tagger.name);
        assert_eq!(tag.tagger.email, tagger.email);
        assert_eq!(tag.tagger.time, tagger.time);
        assert_eq!(tag.tagger.time_zone, tagger.time_zone);

        assert_eq!(tag.tag_name, tag_name);
    }

    #[test]
    fn test_encode_tag() {
        let hash_hex = (0..40).map(|_| 'a').collect::<String>();
        let tag_name = String::from("abc");
        let tagger = Author {
            email: String::from("h@g.com"),
            name: String::from("abc"),
            time: 9,
            time_zone: String::from("-0200"),
        };
        let tag = Tag {
            additional_data: Some(String::from("add data")),
            commit_message: String::from("tag message"),
            object: Hash::try_from(hash_hex.as_bytes()).unwrap(),
            object_type: ObjectKind::Commit,
            tag_name: tag_name.clone(),
            tagger: tagger.clone(),
        };

        let data = encode_tag(tag);

        assert_eq!(
            data,
            format!(
                "object {}\ntype {}\ntag {}\ntagger {} <{}> {} {}\nadd data\n\ntag message\n",
                hash_hex,
                ObjectKind::Commit,
                tag_name,
                tagger.name,
                tagger.email,
                tagger.time,
                tagger.time_zone,
            )
            .into_bytes()
        );
    }
}
