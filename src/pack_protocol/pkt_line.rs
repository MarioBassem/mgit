use std::{error::Error, fmt::Display};

use anyhow::{anyhow, bail, Result};

pub struct PktLine(String);

impl PktLine {
    pub fn new(content: String) -> PktLine {
        let length = content.len() + 5; // 4 length bytes + LF byte
        let length_str = format!("{:04x}", length);

        PktLine(format!("{}{}\n", length_str, content))
    }

    pub fn new_flush() -> PktLine {
        PktLine("0000".to_string())
    }

    pub fn new_end() -> PktLine {
        PktLine("0009done\n".to_string())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub enum Ref {
    Tip { name: String, object_id: String },
    Peeled { name: String, object_id: String },
    Shallow { object_id: String },
}

pub type Shallow = String;

pub type Capability = String;

#[derive(Debug)]
pub enum PktLineError {
    /// indicates an error with pkt line length bytes
    ErrLineLengthBytes(String),
    /// version error
    ErrVersion(String),
    /// indicates an error with no-refs line
    ErrInvalidNoRefs(String),
    /// indicates an invalid capability
    ErrInvalidCapability(String),
    /// indicates an invalid ref
    ErrInvalidRef(String),
    /// indicates an invalid shallow
    ErrInvalidShallow(String),
    /// indicates an invalid flush-pkt
    ErrInvalidFlushPkt,
}

impl Error for PktLineError {}

impl Display for PktLineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ErrLineLengthBytes(err) => write!(f, "invalid pkt-line length: {}", err),
            Self::ErrVersion(err) => write!(f, "invalid version: {}", err),
            Self::ErrInvalidNoRefs(err) => write!(f, "invalid no-refs: {}", err),
            Self::ErrInvalidCapability(err) => write!(f, "invalid capability: {}", err),
            Self::ErrInvalidRef(err) => write!(f, "invalid ref: {}", err),
            Self::ErrInvalidShallow(err) => write!(f, "invalid shallow ref: {}", err),
            Self::ErrInvalidFlushPkt => write!(f, "invalid or missing flush-pkt"),
        }
    }
}

pub struct AdvertisedRefsParser {
    data: String,
    version: u8,
}

impl AdvertisedRefsParser {
    fn peeker<T>(&mut self, f: impl Fn(&str) -> Result<T>) -> Result<T> {
        let line = self.peek_pkt_line()?;
        let line_length = line.len() + 4; // including 4 length bytes

        let res = f(line.trim())?;
        self.data = self.data.split_off(line_length);

        Ok(res)
    }

    /// returns next line without modifying the actual buffer
    fn peek_pkt_line(&mut self) -> Result<&str> {
        let (length_str, rest) = self.data.split_at(4);
        if length_str.len() != 4 {
            bail!(anyhow!(PktLineError::ErrLineLengthBytes(format!(
                "length bytes should be 4 but {} were found",
                length_str.len()
            ))))
        }

        let length = usize::from_str_radix(length_str, 16)?;
        if length > 65520 {
            bail!(anyhow!(PktLineError::ErrLineLengthBytes(format!(
                "line length must not exceed 65520, {} was found",
                length
            ))))
        }

        let (line, _) = rest.split_at(length - 4);
        if line.len() != length - 4 {
            bail!(anyhow!(PktLineError::ErrLineLengthBytes(format!(
                "line length {} was expected, but {} was found",
                length,
                line.len() + 4
            ))))
        }

        Ok(line)
    }

    pub fn parse_advertised_refs(&mut self) -> Result<(Vec<Ref>, Vec<Capability>, Option<Ref>)> {
        /*
            *1("version 1")
            (no-refs / list-of-refs)
            *shallow
            flush-pkt
        */
        let mut refs = Vec::new();
        let capabilities: Vec<Capability>;
        let mut shallow: Option<Ref> = None;
        // let (mut refs, mut capabilities, mut shallow) = (Vec::new(), Vec::new(), None);
        // parse version
        let version = self.peeker(Self::parse_version_line)?;
        if version != self.version {
            return Err(PktLineError::ErrVersion(format!(
                "only supported version is {}",
                self.version
            ))
            .into());
        }
        // try parse no refs. if failed, try parse refs
        if let Ok(caps) = self.peeker(Self::parse_no_refs_line) {
            capabilities = caps
        } else {
            (refs, capabilities) = self.parse_list_of_refs()?;
        }

        // try parse shallow
        if let Ok(shall) = self.peeker(Self::parse_shallow_line) {
            shallow = Some(shall)
        }

        self.peeker(Self::validate_flush_pkt)?;

        Ok((refs, capabilities, shallow))
    }

    fn parse_version_line(line: &str) -> Result<u8> {
        let (version_str, version_number) = line
            .split_once(' ')
            .ok_or(PktLineError::ErrVersion("missing SP".to_string()))?;

        if version_str != "version" {
            return Err(PktLineError::ErrVersion("missing version string".to_string()).into());
        }

        let version = u8::from_str_radix(version_number, 10)
            .map_err(|err| PktLineError::ErrVersion(format!("invalid number: {}", err)))?;

        Ok(version)
    }

    fn parse_no_refs_line(line: &str) -> Result<Vec<Capability>> {
        /*
            no-refs          =  PKT-LINE(zero-id SP "capabilities^{}"
                                NUL capability-list)
        */
        let (zero_id, line) = line
            .split_once(' ')
            .ok_or(PktLineError::ErrInvalidNoRefs("expected SP".to_string()))?;

        Self::validate_zero_id(zero_id)?;

        let cap_want = "capabilities^{}";
        let (capabilities_str, capabilities_list_str) = line
            .split_once('\0')
            .ok_or(PktLineError::ErrInvalidNoRefs("expected NUL".to_string()))?;

        if capabilities_str != cap_want {
            return Err(PktLineError::ErrInvalidNoRefs(format!(
                "invalid capabilities string: {}",
                capabilities_str
            ))
            .into());
        }

        let capabilities = Self::parse_capability_list(capabilities_list_str)?;
        Ok(capabilities)
    }

    /// ensures the provided string forms a zero-id
    fn validate_zero_id(zero_id_str: &str) -> Result<()> {
        if zero_id_str != (0..40).map(|_| "0").collect::<String>().as_str() {
            return Err(PktLineError::ErrInvalidNoRefs("invalid zero id".to_string()).into());
        }

        Ok(())
    }

    /// tries to parse list of refs
    fn parse_list_of_refs(&mut self) -> Result<(Vec<Ref>, Vec<Capability>)> {
        /*
            list-of-refs     =  first-ref *other-ref

            next is shallow or flush-pkt
                shallow          =  PKT-LINE("shallow" SP obj-id)
                flush-pkt    = "0000"

        */

        let (first_ref, capabilities) = self.peeker(Self::parse_first_ref)?;
        let mut other_refs = Vec::new();
        loop {
            let line = self.peek_pkt_line()?;
            if line.starts_with("0000") || line.starts_with("shallow") {
                break;
            }

            let other_ref = self.peeker(Self::parse_other_ref)?;
            other_refs.push(other_ref);
        }

        let mut refs = Vec::new();
        refs.push(first_ref);
        refs.append(&mut other_refs);

        Ok((refs, capabilities))
    }

    fn parse_first_ref(line: &str) -> Result<(Ref, Vec<Capability>)> {
        /*
            first-ref        =  PKT-LINE(obj-id SP refname
                                NUL capability-list)
        */

        let (object_id, line) = line
            .split_once(' ')
            .ok_or(PktLineError::ErrInvalidRef("expected SP".to_string()))?;

        if object_id.len() != 40 {
            return Err(PktLineError::ErrInvalidRef(format!(
                "invalid object id length '{}'",
                object_id.len()
            ))
            .into());
        }

        let (refname, cap_list_str) = line
            .split_once('\0')
            .ok_or(PktLineError::ErrInvalidRef("expected NUL".to_string()))?;

        let capabilities = Self::parse_capability_list(cap_list_str)?;

        Ok((
            Ref::Tip {
                name: refname.to_string(),
                object_id: object_id.to_string(),
            },
            capabilities,
        ))
    }

    fn parse_other_ref(line: &str) -> Result<Ref> {
        /*
            other-ref        =  PKT-LINE(other-tip / other-peeled)
            other-tip        =  obj-id SP refname
            other-peeled     =  obj-id SP refname "^{}"
        */

        let (object_id, refname) = line
            .split_once(' ')
            .ok_or(PktLineError::ErrInvalidRef("expected SP".to_string()))?;

        if object_id.len() != 40 {
            return Err(PktLineError::ErrInvalidRef(format!(
                "invalid object id length '{}'",
                object_id.len()
            ))
            .into());
        }

        let ret: Ref;
        if refname.ends_with("^{}") {
            ret = Ref::Peeled {
                name: refname.trim_end_matches("^{}").to_string(),
                object_id: object_id.to_string(),
            }
        } else {
            ret = Ref::Tip {
                name: refname.to_string(),
                object_id: object_id.to_string(),
            }
        }

        Ok(ret)
    }

    fn parse_shallow_line(line: &str) -> Result<Ref> {
        /*
            shallow          =  PKT-LINE("shallow" SP obj-id)
        */

        let (shallow_str, object_id) = line
            .split_once(' ')
            .ok_or(PktLineError::ErrInvalidShallow("expected SP".to_string()))?;

        if shallow_str != "shallow" {
            return Err(PktLineError::ErrInvalidShallow("expected 'shallow'".to_string()).into());
        }

        Ok(Ref::Shallow {
            object_id: object_id.to_string(),
        })
    }

    fn parse_capability_list(list: &str) -> Result<Vec<Capability>> {
        let capabilities = list
            .split_terminator(' ')
            .map(|s| s.to_string())
            .collect::<Vec<Capability>>();

        Self::validate_capabilities(&capabilities);

        Ok(capabilities)
    }

    fn validate_capabilities(caps: &Vec<Capability>) -> Result<()> {
        caps.iter().map(|cap| -> Result<()> {
            if cap.len() == 0 {
                return Err(PktLineError::ErrInvalidCapability(
                    "invalid empty capability".to_string(),
                )
                .into());
            }

            cap.as_bytes().iter().map(|c| -> Result<()> {
                if c.is_ascii_lowercase() || c.is_ascii_digit() || *c == b'-' || *c == b'_' {
                    return Ok(());
                }

                return Err(PktLineError::ErrInvalidCapability(format!(
                    "invalid capability character: {}",
                    c
                ))
                .into());
            });

            Ok(())
        });

        Ok(())
    }

    fn validate_flush_pkt(line: &str) -> Result<()> {
        if line != "0000" {
            return Err(PktLineError::ErrInvalidFlushPkt.into());
        }

        Ok(())
    }
}
