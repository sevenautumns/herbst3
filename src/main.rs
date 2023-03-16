use anyhow::Result;
use clap::Parser;

use crate::cli::Args;

mod cli;
mod herbstclient;
mod parser;
mod shift;

fn main() -> Result<()> {
    pretty_env_logger::init();

    match Args::parse() {
        Args::Shift(dir) => shift::shift(dir)?,
    }

    Ok(())
    // let index = get_focused_index()?;
    // println!("{index:#?}");
    // let stack = get_layout_stack(index)?;
    // println!("{stack:?}");
    // Ok(())
}
