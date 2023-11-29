use std::error::Error;
use std::path::PathBuf;

use clap::builder::TypedValueParser as _;
use clap::Parser;

use tracing_subscriber::EnvFilter;

use complex_code_spotter::{Complexity, OutputFormat, SnippetsProducer};

const fn thresholds_long_help() -> &'static str {
    "A threshold of 0 is the minimum allowed value, thus no threshold at all.\n\
     A threshold of 100 is the maximum allowed value, thus each complexity value is not accepted.\n\n\
   Thresholds values of 0 and 100 are not recommended, better intermediate values"
}

// Parse a single key-value pair
fn parse_key_val<T, U>(s: &str) -> Result<(T, U), Box<dyn Error + Send + Sync + 'static>>
where
    T: std::str::FromStr,
    T::Err: Error + Send + Sync + 'static,
    U: std::str::FromStr,
    U::Err: Error + Send + Sync + 'static,
{
    let pos = s
        .find('=')
        .ok_or_else(|| format!("invalid KEY=value: no `=` found in `{s}`"))?;
    Ok((s[..pos].parse()?, s[pos + 1..].parse()?))
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct CargoArgs {
    /// Path to a Cargo.toml
    #[clap(long)]
    pub(crate) manifest_path: Option<PathBuf>,
    #[clap(flatten)]
    pub(crate) args: Args,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub(crate) struct Args {
    /// Path to the source files to be analyzed
    pub(crate) source_path: PathBuf,
    /// Output path containing the snippets of complex code for each file
    output_path: PathBuf,
    /// Whether to output every snippet of complex code within a single file
    is_single_file: bool,
    /// Output the generated paths as they are produced
    #[arg(short, long)]
    verbose: bool,
    /// Glob to include files
    #[arg(long, short = 'I')]
    include: Vec<String>,
    /// Glob to exclude files
    #[arg(long, short = 'X')]
    exclude: Vec<String>,
    /// Output format
    #[arg(long, short = 'O', default_value_t = OutputFormat::Json,
        value_parser = clap::builder::PossibleValuesParser::new(OutputFormat::all())
            .map(|s| s.parse::<OutputFormat>().unwrap()),)]
    output_format: OutputFormat,
    /// List of complexities metrics and thresholds considered for snippets
    #[arg(short, value_parser = parse_key_val::<Complexity, usize>, long_help = thresholds_long_help())]
    complexities: Vec<(Complexity, usize)>,
}

pub(crate) fn run_complex_code_spotter(args: Args) {
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
        .complexities(args.complexities)
        .output_format(args.output_format)
        .output(&args.output_path)
        .is_single_file(args.is_single_file)
        .include(args.include)
        .exclude(args.exclude)
        .run(args.source_path)
        .unwrap();
}
