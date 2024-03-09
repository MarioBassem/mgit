mod blob;
mod init;

use std::process::exit;

use clap::{Parser, Subcommand, ValueEnum};
use log::error;
use simple_logger::{set_up_color_terminal, SimpleLogger};

#[derive(Debug, Parser)]
#[command(name = "mgit", about = "A simple VSC")]
struct CLI {
    commands: Commands,
}

impl std::fmt::Display for CLI {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.commands)
    }
}

#[derive(Debug, Subcommand, Clone, ValueEnum)]
enum Commands {
    /// Initializes a new git repo
    #[command()]
    Init,
}

fn main() {
    set_up_color_terminal();
    let logger = SimpleLogger::new().without_timestamps();
    let max_level = logger.max_level();

    log::set_max_level(max_level);
    log::set_boxed_logger(Box::new(logger)).unwrap();

    let args = CLI::parse();

    let res = match args.commands {
        Commands::Init => init::init(),
    };

    if let Err(err) = res {
        error!("{}", err);
        exit(1)
    }
}
