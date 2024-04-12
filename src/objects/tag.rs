use std::io::BufRead;

use super::{commit::Author, hash::Hash, Object, ObjectKind};
use anyhow::{anyhow, Result};

pub struct Tag {
    object: Hash,
    object_type: ObjectKind,
    tag_name: String,
    tagger: Author,
    commit_message: Option<String>,
    additional_data: Option<String>,
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
    let mut tag: Tag;
    let lines: Vec<String> = data.lines().collect::<Result<_, _>>()?;
    for (i, line) in lines.iter().enumerate() {
        if line.len() == 0 {
            // next is commit message
            if i != lines.len() - 2 {
                return Err(anyhow!("invalid tag data"));
            }

            tag.commit_message = Some(lines[i + 1]);
            break;
        }

        let (first_word, words) = line.split_once(' ').ok_or(anyhow!("invalid tag data"))?;
        match first_word {
            "object" => tag.object = Hash::try_from(words.as_bytes())?,
            "type" => tag.object_type = ObjectKind::try_from(words)?,
            "tag" => tag.tag_name = String::from(words),
            "tagger" => tag.tagger = Author::try_from(words)?,
            _ => tag.additional_data = Some(line.to_string()),
        }
    }

    Ok(tag)
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
    if let Some(message) = tag.commit_message {
        content.append(&mut format!("{}\n", message).into_bytes())
    }

    content
}
