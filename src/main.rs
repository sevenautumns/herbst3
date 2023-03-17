use clap::Parser;
use cli::SubCommand;
use log::error;

use crate::cli::Args;

mod cli;
mod helper;
mod herbstclient;
mod parser;
mod shift;

fn main() {
    pretty_env_logger::init();

    let args = Args::parse();
    let res = match args.cmd {
        SubCommand::Shift(dir) => shift::shift(dir, args.frame),
    };

    if let Err(e) = res {
        error!("{e:?}")
    }
}
