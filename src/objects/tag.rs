use bytes::Bytes;

use super::{commit::Author, hash::HashHex, Object, ObjectKind, ObjectTrait};
use anyhow::Result;

pub fn new_tag(
    object: HashHex,
    object_type: ObjectKind,
    tag_name: String,
    tagger: Author,
    commit_message: Option<String>,
    signature: Option<String>,
) -> Object {
    todo!()
}

pub fn parse_tag(data: Vec<u8>) -> Result<Object> {
    todo!()
}

/*
    format:
        tag size NUL object object_hex_hash LF
        type object_type LF
        tag tag_name LF
        tagger author LF LF
        commit_message LF
        signature

*/
