use std::{error::Error, fmt::Display};

use anyhow::{anyhow, bail, Result};

pub struct PktLine(String);

pub struct Ref(String);

pub struct Shallow(String);

type Capability = String;

#[derive(Debug)]
pub enum PktLineError {
    /// version error
    ErrVersion(String),

    ErrInvalidNoRefs(String),
}

impl Error for PktLineError {}

impl Display for PktLineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PktLineError::ErrVersion(err) => write!(f, "pkt-line error: {}", err),
        }
    }
}

pub fn read_pkt_line() -> Result<Vec<PktLine>> {
    // read length in first 4 bytes
    // read next (length) bytes
    //
    todo!()
}

fn parse_advertised_refs(data: &mut str) -> Result<(Vec<Ref>, Vec<Capability>)> {
    /*
        *1("version 1")
        (no-refs / list-of-refs)
        *shallow
        flush-pkt
    */
    // parse version
    let version = parse_version(data)?;
    // try parse no refs. if failed, try parse refs
    // try parse shallow
    // parse shallow pkt
    todo!()
}

pub enum Version {
    One,
    Two,
    Three,
}

fn parse_version(data: &mut str) -> Result<u8> {
    let (version_str, data) = data.split_once(' ').ok_or(PktLineError::ErrVersion(
        "failed to read version".to_string(),
    ))?;

    let (version_number, data) = data.split_at(1);
    let version = u8::from_str_radix(version_number, 10)
        .map_err(|err| PktLineError::ErrVersion(format!("invalid version: {}", err)))?;

    if data.starts_with('\n') {
        let (_, data) = data.split_at(1);
    }

    Ok(version)
}

fn try_parse_no_refs(data: &mut str) -> Result<Vec<Capability>> {
    /*
        no-refs          =  PKT-LINE(zero-id SP "capabilities^{}"
                            NUL capability-list)
    */

    let _ = get_zero_id(data)?;
    let (sp, mut data) = data.split_at(1);
    if sp != "" {
        bail!(anyhow!(PktLineError::ErrInvalidNoRefs(
            "expected SP".to_string()
        )))
    }

    let cap_want = "capabilities^{}";
    let (capabilities_str, mut data) = data.split_at(cap_want.len());
    if capabilities_str != cap_want {
        bail!(anyhow!(PktLineError::ErrInvalidNoRefs(
            "invalid capabilities string".to_string()
        )))
    }

    let capabilities = parse_capability_list(&mut data)?;
    Ok(capabilities)
}

/// ensures the next 40 bytes from zero-id
fn get_zero_id(data: &mut str) -> Result<()> {
    let (zero_id, data) = data.split_at(40);

    if zero_id != (0..40).map(|_| "0").collect::<String>().as_str() {
        bail!(anyhow!(PktLineError::ErrInvalidNoRefs(
            "invalid zero id".to_string()
        )))
    }

    Ok(())
}

/// tries to parse list of refs
fn try_parse_list_of_refs(data: &mut str) -> Result<(Vec<Ref>, Vec<Capability>)> {
    /*
        first-ref *other-ref
    */
    // TODO: check if ref is no-ref first
    let (first_ref, capabilities) = parse_first_ref(data)?;
    let mut other_refs = try_parse_other_ref(data)?;
    let mut refs = Vec::new();
    refs.push(first_ref);
    refs.append(&mut other_refs);

    Ok((refs, capabilities))
}

fn parse_first_ref(data: &mut str) -> Result<(Ref, Vec<Capability>)> {
    todo!()
}

fn try_parse_other_ref(data: &str) -> Result<Vec<Ref>> {
    todo!()
}

fn parse_other_tip(data: &str) -> Result<Ref> {
    todo!()
}

fn parse_other_peeled(data: &str) -> Result<Ref> {
    todo!()
}

fn try_parse_shallow(data: &str) -> Result<Shallow> {
    todo!()
}

fn parse_capability_list(data: &mut str) -> Result<Vec<Capability>> {
    todo!()
}

fn parse_capability(data: &str) -> Result<Capability> {
    todo!()
}
