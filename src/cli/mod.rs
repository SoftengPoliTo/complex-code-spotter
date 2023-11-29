use std::error::Error;
use std::path::PathBuf;

use clap::Parser;

use complex_code_spotter::{Complexity, OutputFormat};

const fn thresholds_long_help() -> &'static str {
    "A threshold of 0 is the minimum allowed value, thus no threshold at all.\n\
     A threshold of 100 is the maximum allowed value, thus each complexity value is not accepted.\n\n\
   Thresholds values of 0 and 100 are not recommended, better intermediate values"
}

/// Parse a single key-value pair
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
    pub(crate) output_path: PathBuf,
    /// Output the generated paths as they are produced
    #[arg(short, long)]
    pub(crate) verbose: bool,
    /// Glob to include files
    #[arg(long, short = 'I')]
    pub(crate) include: Vec<String>,
    /// Glob to exclude files
    #[arg(long, short = 'X')]
    pub(crate) exclude: Vec<String>,
    /// Output format
    #[arg(long, short = 'O')]
    pub(crate) output_format: OutputFormat,
    /// List of complexities metrics and thresholds considered for snippets
    #[arg(short, value_parser = parse_key_val::<Complexity, usize>, long_help = thresholds_long_help())]
    pub(crate) complexities: Vec<(Complexity, usize)>,
}
