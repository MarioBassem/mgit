use crate::objects::hash::Hash;
use crate::pack_protocol::pack_file::PackFile;
use anyhow::{anyhow, bail, Context, Result};
use reqwest::blocking;
use std::{collections::HashMap, io::Read, io::Write, path::PathBuf};
use url::Url;

struct Ref {}
struct DiscoveredRefs {
    refs: HashMap<String, String>,
    capabilities: Vec<String>,
}

/// clones a git repo to the specified path
pub fn clone(url: String, path: Option<PathBuf>) -> Result<()> {
    // validate url
    // validate or assign path

    // create repo dir
    // init repo
    // discover refs
    // fetch refs
    // checkout commit
    todo!()
}

fn discover_refs(url: &str) -> Result<DiscoveredRefs> {
    let client = reqwest::blocking::Client::new();

    let base_url = Url::parse(format!("{}/", url).as_str())?;
    let mut refs_url = base_url.join("info/refs")?;
    refs_url
        .query_pairs_mut()
        .append_pair("service", "git-upload-pack");

    let response = client.get(refs_url).send()?;
    if response.status() != reqwest::StatusCode::OK {
        bail!(anyhow!("failed to fetch refs: {}", response.status()));
    }

    let response_headers = response.headers();
    let content_type = response_headers
        .get("Content-Type")
        .ok_or_else(|| anyhow::anyhow!("missing Content-Type"))?;

    anyhow::ensure!(
        content_type == "application/x-git-upload-pack-advertisement",
        "invalid Content-Type: '{}'",
        content_type.to_str()?
    );

    let mut response_bytes = response.bytes()?;

    let header = read_pkt_line(&mut response_bytes)
        .context("error while reading header")?
        .ok_or(anyhow!("missing header"))?;

    anyhow::ensure!(
        header == "# service=git-upload-pack\n",
        "invalid header: {}",
        header
    );

    let start_sym = read_pkt_line(&mut response_bytes).context("while reading start_sym")?;
    anyhow::ensure!(
        start_sym.is_none(),
        "expected start_sym, but got {:?}",
        start_sym
    );

    let mut capabilities = Vec::new();
    let mut refs = HashMap::new();

    while let Some(line) =
        read_pkt_line(&mut response_bytes).context("error while reading ref line")?
    {
        let line = line.trim();
        if line.contains('\0') {
            let (line, cap) = line
                .split_once(' ')
                .ok_or_else(|| anyhow!("invalid ref line: {}", line))?;
            capabilities.extend(cap.split_ascii_whitespace().map(|c| c.to_string()));

            let (hash, name) = line
                .split_once(' ')
                .ok_or_else(|| anyhow!("invalid ref line: {}", line))?;

            refs.insert(name.to_string(), hash.to_string());
        } else {
            let (hash, name) = line
                .split_once(' ')
                .ok_or_else(|| anyhow!("invalid ref line: {}", line))?;
            refs.insert(name.to_string(), hash.to_string());
        }
    }

    Ok(DiscoveredRefs { refs, capabilities })
}

fn fetch_refs(url: &str, refs: &[&str]) -> Result<()> {
    let client = reqwest::blocking::Client::new();
    let url = Url::parse(format!("{}/git-upload-pack", url).as_str())?;

    let mut body = Vec::new();
    for r in refs {
        // TODO: check if we don't have ref
        writeln!(body, "0032want {}", r)?;
    }

    writeln!(body, "00000009done")?;

    let response = client
        .post(url)
        .body(body)
        .header("Content-type", "application/x-git-upload-pack-request")
        .send()?;

    if response.status() != reqwest::StatusCode::OK {
        bail!(anyhow!("failed to fetch refs: {}", response.status()))
    }

    let mut bytes = response.bytes()?;

    let header = read_pkt_line(&mut bytes)
        .context("error while reading header")?
        .ok_or(anyhow!("missing header"))?;
    let header = header.trim();
    anyhow::ensure!(header == "NAK", "only NAK response is supported");

    let mut pack_file = PackFile::new(bytes.clone())?;
    let pack_objects = pack_file.read_objects()?;
    println!("pack objects: {:?}", pack_objects);
    for pack_obj in pack_objects {
        // let (hash, obj) = pack_obj.prepare()?;
    }

    Ok(())
}

fn checkout_commit(commit: Hash) -> Result<()> {
    todo!()
}

fn read_pkt_line(bytes: &[u8]) -> Result<Option<String>> {
    todo!()
}
