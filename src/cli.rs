use clap::{Parser, Subcommand};
use strum::Display;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Shift focused window
    #[command(subcommand)]
    pub cmd: SubCommand,

    /// Only shift on "Frame" level
    #[arg(short, long, global(true))]
    pub frame: bool,
}

#[derive(Parser, Debug)]
pub enum SubCommand {
    #[command(subcommand)]
    Shift(ShiftDirection),
}

#[derive(Subcommand, Debug, Display, Clone, Copy, PartialEq)]
#[strum(serialize_all = "camelCase")]
pub enum ShiftDirection {
    /// Shift focused window right
    Right,
    /// Shift focused window left
    Left,
    /// Shift focused window up
    Up,
    /// Shift focused window down
    Down,
}
