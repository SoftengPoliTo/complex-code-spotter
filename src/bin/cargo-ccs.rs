#[path = "../cli/mod.rs"]
mod cli;

use clap::{Parser, Subcommand};

use cli::{run_complex_code_spotter, CargoArgs};

#[derive(Subcommand)]
enum Cmd {
    /// Complex Code Spotter cargo subcommand
    #[clap(name = "ccs")]
    Ccs(CargoArgs),
}

/// Complex Code Spotter cargo applet
#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    cargo_args: Cmd,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Cli {
        cargo_args: Cmd::Ccs(mut cargo_args),
    } = Cli::parse();

    let mut cmd = cargo_metadata::MetadataCommand::new();
    if let Some(ref manifest_path) = cargo_args.manifest_path {
        cmd.manifest_path(manifest_path);
    }

    let metadata = cmd.exec()?;
    cargo_args.args.source_path = metadata.workspace_packages()[0]
        .manifest_path
        .parent()
        .unwrap()
        .join("src")
        .into_std_path_buf();

    run_complex_code_spotter(cargo_args.args);

    Ok(())
}
