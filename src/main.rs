use clap::Parser;
use log::error;

use crate::cli::Args;

mod cli;
mod helper;
mod herbstclient;
mod parser;
mod shift;

fn main() {
    pretty_env_logger::init();

    let res = match Args::parse() {
        Args::Shift(dir) => shift::shift(dir),
    };

    if let Err(e) = res {
        error!("{e:?}")
    }
}
