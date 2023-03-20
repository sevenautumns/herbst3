use std::env;
use std::io::Error;
use std::path::PathBuf;

use clap::CommandFactory;
use clap_complete::generate_to;
use clap_complete::shells::{Bash, Fish, Zsh};

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    cc::Build::new()
        .define("HERBSTLUFT_VERSION", "\"herbst3\"")
        .flag("-Wno-unused-parameter")
        // .flag("-Dmain=ipc_main")
        // .file("herbstluftwm/ipc-client/main.c")
        .file("herbstluftwm/ipc-client/ipc-client.c")
        .file("herbstluftwm/ipc-client/client-utils.c")
        .opt_level(3)
        .compile("herbstipc");
    println!("cargo:rustc-link-lib=X11");

    let bindings = bindgen::Builder::default()
        .header("herbstluftwm/ipc-client/ipc-client.c")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .allowlist_function("hc_send_command")
        .allowlist_function("hc_connect")
        .allowlist_function("hc_disconnect")
        .allowlist_function("hc_check_running")
        .layout_tests(false)
        .generate()
        .expect("Unable to generate bindings");
    bindings
        .write_to_file(PathBuf::from(&outdir).join("herbst_ipc.rs"))
        .expect("Error writing bindgen");

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
