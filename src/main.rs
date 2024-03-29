mod blob;
mod hash_object;
mod init;
mod objects;
mod tree;

use std::{path::PathBuf, process::exit};

use blob::read_blob;
use clap::Parser;
use hash_object::hash_object;
use log::error;
use simple_logger::{set_up_color_terminal, SimpleLogger};

#[derive(Debug, Parser, Clone)]
#[command(name = "mgit", about = "A simple VSC")]
enum Cli {
    /// Initializes a new git repo
    #[command()]
    Init,

    #[command()]
    CatFile { object: String },

    #[command()]
    HashObject {
        #[clap(short = 'w')]
        write: bool,
        file_path: String,
    },
}

fn main() {
    set_up_color_terminal();
    let logger = SimpleLogger::new().without_timestamps();
    let max_level = logger.max_level();

    log::set_max_level(max_level);
    log::set_boxed_logger(Box::new(logger)).unwrap();

    let args = Cli::parse();

    let res = match args {
        Cli::Init => init::init(),
        Cli::CatFile { object: hash } => read_blob(hash),
        Cli::HashObject { write, file_path } => hash_object(PathBuf::from(file_path), write),
    };

    if let Err(err) = res {
        error!("{}", err);
        exit(1)
    }
}
