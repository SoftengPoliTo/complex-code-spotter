#[path = "../cli/mod.rs"]
mod cli;

use clap::Parser;

use cli::{run_complex_code_spotter, Args};

fn main() {
    let args = Args::parse();
    let source_path = args.source_path.clone();
    run_complex_code_spotter(args, source_path);
}
