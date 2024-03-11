mod blob;
mod init;

use std::process::exit;

use blob::read_blob;
use clap::{Parser, Subcommand, ValueEnum};
use log::error;
use simple_logger::{set_up_color_terminal, SimpleLogger};

#[derive(Debug, Parser, Clone)]
#[command(name = "mgit", about = "A simple VSC")]
enum CLI {
    /// Initializes a new git repo
    #[command()]
    Init,

    #[command()]
    CatFile { object: String },
}

fn main() {
    set_up_color_terminal();
    let logger = SimpleLogger::new().without_timestamps();
    let max_level = logger.max_level();

    log::set_max_level(max_level);
    log::set_boxed_logger(Box::new(logger)).unwrap();

    let args = CLI::parse();

    let res = match args {
        CLI::Init => init::init(),
        CLI::CatFile { object: hash } => read_blob(hash),
    };

    if let Err(err) = res {
        error!("{}", err);
        exit(1)
    }
}
