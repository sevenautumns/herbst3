use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub enum Args {
    /// Shift focused window
    #[command(subcommand)]
    Shift(ShiftDirection),
}

#[derive(Subcommand, Debug)]
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
