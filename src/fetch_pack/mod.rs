use anyhow::{anyhow, bail, Context, Result};

pub fn fetch_pack(url: &str) -> Result<()>{
    /*
        doc: https://git-scm.com/docs/git-fetch-pack

        Invokes git-upload-pack on a possibly remote repository and asks it 
        to send objects missing from this repository, to update the named heads. 
        The list of commits available locally is found out by scanning the 
        local refs/ hierarchy and sent to git-upload-pack running on the other end.

        This command degenerates to download everything to complete the asked refs from
        the remote side when the local side does not have a common ancestor commit.
    */
    todo!()
}