use super::pkt_line::Capability;
use super::pkt_line::PktLine;
use super::pkt_line::Ref;
use super::pkt_line::Shallow;
use anyhow::anyhow;
use anyhow::Result;
use std::io::Write;

pub enum DepthRequest {
    Deepen(u32),
    DeepenSince(u64),
    DeepenNot(String),
}

pub type FilterRequest = String;

pub fn upload_pack_request(
    want_list: Vec<String>,
    // shallow_commits: Vec<String>,
    // depth_requests: Vec<DepthRequest>,
    // filter_requests: Vec<String>,
    // caps: Vec<Capability>,
) -> Result<Vec<u8>> {
    let mut request = Vec::new();

    if want_list.len() < 0 {
        return Err(anyhow!("want list must have at least 1 want"));
    }

    // if caps.len() < 0 {
    //     return Err(anyhow!("capabilities list must have at least 1 capability"));
    // }

    // let first_want = want_list[0];
    // let cap_list = caps.join(" ");
    // request.push_str(PktLine::new(format!("want {} {}", first_want, cap_list)).as_str());

    for want in want_list {
        write!(
            request,
            "{}",
            PktLine::new(format!("want {}", want)).as_str()
        );
    }

    // for shallow in shallow_commits {
    //     request.push_str(PktLine::new(format!("shallow {}", shallow)).as_str())
    // }

    // for r in depth_requests {
    //     match r {
    //         DepthRequest::Deepen(depth) => {
    //             request.push_str(PktLine::new(format!("deepen {}", depth)).as_str())
    //         }
    //         DepthRequest::DeepenNot(refname) => {
    //             request.push_str(PktLine::new(format!("deepen-not {}", refname)).as_str())
    //         }
    //         DepthRequest::DeepenSince(timestamp) => {
    //             request.push_str(PktLine::new(format!("deepen-since {}", timestamp)).as_str())
    //         }
    //     }
    // }

    // for r in filter_requests {
    //     request.push_str(PktLine::new(format!("filter {}", r)).as_str())
    // }

    write!(request, "{}", PktLine::new_flush().as_str());
    write!(request, "{}", PktLine::new_end().as_str());

    Ok(request)
}

// pub fn parse_upload_request_server_response()

/*
    negotiation phase:
        - after reference discovery a client can terminate the connection if it does not need to fetch anything.
        - otherwise, the client enters negotiation phase to determine the min pack file needed
        - The client MUST write all obj-ids which it only has shallow copies of
            (meaning that it does not have the parents of a commit) as shallow lines
            so that the server is aware of the limitations of the client’s history.

*/
