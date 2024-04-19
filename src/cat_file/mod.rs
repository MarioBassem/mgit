use anyhow::Result;
use log::info;

use crate::objects::{
    blob::decode_blob, commit::decode_commit, tag::decode_tag, tree::decode_tree, Object,
    ObjectKind,
};

pub fn cat_file(hash: String) -> Result<()> {
    let object = Object::read_from_hash(hash)?;
    match object.kind {
        ObjectKind::Blob => {
            let blob = decode_blob(object.data);
            print!("{}", String::from_utf8(blob.data)?);
        }
        ObjectKind::Commit => {
            let commit = decode_commit(object.data)?;
            print!("{}", commit)
        }
        ObjectKind::Tag => {
            let tag = decode_tag(object.data)?;
            print!("{}", tag)
        }
        ObjectKind::Tree => {
            let tree = decode_tree(object.data)?;
            print!("{}", tree)
        }
    }

    Ok(())
}

/*
    TODO: let all objects implement display, expose a method that returns a type that implements display
*/
