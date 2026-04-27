use clap::{CommandFactory, ValueEnum};
use clap_complete::generate_to;
use clap_complete_nushell::Nushell;
use std::{env, io::Error};

include!("src/cli.rs"); // Import your CLI struct

fn main() -> Result<(), Error> {
    let outdir = match env::var_os("OUT_DIR") {
        None => return Ok(()),
        Some(outdir) => outdir,
    };

    let mut cmd = Cli::command(); // Your Clap Command

    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, "dot", &outdir)?;
    }

    generate_to(Nushell, &mut cmd, "dot", &outdir)?;

    Ok(())
}
