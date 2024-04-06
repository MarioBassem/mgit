use bytes::Bytes;

use super::{commit::Author, Object, ObjectTrait};
use anyhow::Result;

pub struct AnnotatedTag {
    data: Vec<u8>,
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

impl ObjectTrait for AnnotatedTag {
    fn write(&self) -> Result<()> {
        todo!()
    }

    fn hash(&self) -> Result<Vec<u8>> {
        todo!()
    }

    fn hash_hex(&self) -> Result<String> {
        todo!()
    }

    fn compress(&self) -> Result<Vec<u8>> {
        todo!()
    }

    fn data(&self) -> &Vec<u8> {
        todo!()
    }
}
