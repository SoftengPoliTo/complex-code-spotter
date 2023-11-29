#[path = "../cli/mod.rs"]
mod cli;

use clap::Parser;

use tracing_subscriber::EnvFilter;

use complex_code_spotter::SnippetsProducer;

use cli::Args;

fn main() {
    let args = Args::parse();

    let complexity = args.complexities.iter().map(|v| v.0).collect();
    let thresholds = args.complexities.iter().map(|v| v.1).collect();

    // Enable filter to log the information contained in the lib.
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| {
            if args.verbose {
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
        .output_format(args.output_format)
        .include(args.include)
        .exclude(args.exclude)
        .run(args.source_path, args.output_path)
        .unwrap();
}
