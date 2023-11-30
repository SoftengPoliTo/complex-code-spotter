#[path = "../cli/mod.rs"]
mod cli;

use clap::Parser;

use cli::{run_complex_code_spotter, Args};

fn main() {
    let args = Args::parse();
    run_complex_code_spotter(args);
}
