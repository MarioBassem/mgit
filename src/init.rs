use std::fs;

pub fn init() -> Result<(), std::io::Error> {
    log::info!("initializing git repo...");

    fs::create_dir(".git")?;
    fs::create_dir(".git/objects")?;
    fs::create_dir(".git/refs")?;
    fs::write(".git/HEAD", "ref: refs/head/main\n")?;
    Ok(())
}
