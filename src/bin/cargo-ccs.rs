#[path = "../cli/mod.rs"]
mod cli;

use clap::{Parser, Subcommand};

use tracing_subscriber::EnvFilter;

use complex_code_spotter::SnippetsProducer;

use cli::CargoArgs;

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
        cargo_args: Cmd::Ccs(cargo_args),
    } = Cli::parse();

    let mut cmd = cargo_metadata::MetadataCommand::new();
    if let Some(ref manifest_path) = cargo_args.manifest_path {
        cmd.manifest_path(manifest_path);
    }

    let metadata = cmd.exec()?;
    let source_path = metadata.workspace_packages()[0]
        .manifest_path
        .parent()
        .unwrap()
        .join("src")
        .into_std_path_buf();

    let complexity = cargo_args.args.complexities.iter().map(|v| v.0).collect();
    let thresholds = cargo_args.args.complexities.iter().map(|v| v.1).collect();

    // Enable filter to log the information contained in the lib.
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| {
            if cargo_args.args.verbose {
                EnvFilter::try_new("debug")
            } else {
                EnvFilter::try_new("info")
            }
        })
        .unwrap();

    // Run tracer.
    tracing_subscriber::fmt()
        .without_time()
        .with_env_filter(filter_layer)
        .with_writer(std::io::stderr)
        .init();

    SnippetsProducer::new()
        .complexities(complexity)
        .thresholds(thresholds)
        .enable_write()
        .output_format(cargo_args.args.output_format)
        .include(cargo_args.args.include)
        .exclude(cargo_args.args.exclude)
        .run(source_path, cargo_args.args.output_path)?;

    Ok(())
}
