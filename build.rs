use std::env;
use std::io::Error;

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::{Bash, Fish, Zsh};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Args::command();
    let bin = "herbst3";

    let path = generate_to(Bash, &mut cmd, bin, &outdir)?;
    println!("cargo:warning=completion file is generated: {:?}", path);
    let path = generate_to(Fish, &mut cmd, bin, &outdir)?;
    println!("cargo:warning=completion file is generated: {:?}", path);
    let path = generate_to(Zsh, &mut cmd, bin, &outdir)?;
    println!("cargo:warning=completion file is generated: {:?}", path);

    Ok(())
}
