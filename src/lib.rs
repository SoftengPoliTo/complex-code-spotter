#![deny(missing_docs, unsafe_code)]

//! The `complex-code-spotter` tool extracts snippets of code deemed complex
//! according to the following complexity metrics:
//!
//! - Cyclomatic
//! - Cognitive
//!
//! When the value associated to each of the metrics exceeds a preset threshold,
//! a snippet of code is automatically extracted.

mod concurrent;
mod error;
mod metrics;
mod non_utf8;
mod output;
mod snippets;

pub use metrics::Complexity;
pub use output::OutputFormat;
pub use snippets::Snippets;

use std::path::{Path, PathBuf};
use std::thread::available_parallelism;

use globset::{Glob, GlobSet, GlobSetBuilder};
use rust_code_analysis::{get_function_spaces, guess_language, read_file_with_eol};
use tracing::info;

use concurrent::{ConcurrentRunner, FilesData};
use error::{Error, Result};
use non_utf8::encode_to_utf8;
use snippets::get_code_snippets;

#[derive(Debug, Default)]
struct Parameters<'a> {
    output_format: OutputFormat,
    output_path: Option<&'a Path>,
    is_single_file: bool,
    include: Vec<String>,
    exclude: Vec<String>,
    complexities: Vec<(Complexity, usize)>,
}

/// Produce snippets of complex code for a source file.
///
/// If no parameters are set, the producer uses *cyclomatic* with a
/// threshold of 15 as default metric.
/// Write on files is disabled by default, but when enabled,
/// *markdown* is the output format.
#[derive(Debug)]
pub struct SnippetsProducer<'a>(Parameters<'a>);

impl<'a> Default for SnippetsProducer<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> SnippetsProducer<'a> {
    /// Creates a new `SnippetsProducer` instance.
    pub fn new() -> Self {
        Self(Parameters {
            complexities: vec![(Complexity::Cyclomatic, 15)],
            ..Default::default()
        })
    }

    /// Sets a glob to include only a certain kind of files.
    pub fn include(mut self, include: Vec<String>) -> Self {
        self.0.include = include;
        self
    }

    /// Sets a glob to exclude only a certain kind of files.
    pub fn exclude(mut self, exclude: Vec<String>) -> Self {
        self.0.exclude = exclude;
        self
    }

    /// Sets all complexities metric that will be computed.
    pub fn complexities(mut self, complexities: Vec<(Complexity, usize)>) -> Self {
        self.0.complexities = complexities;
        self
    }

    /// Sets output path.
    pub fn output(mut self, path: &'a Path) -> Self {
        self.0.output_path = Some(path);
        self
    }

    /// Whether the output file is a single file path.
    pub fn is_single_file(mut self, is_single_file: bool) -> Self {
        self.0.is_single_file = is_single_file;
        self
    }

    /// Sets output format.
    pub fn output_format(mut self, output_format: OutputFormat) -> Self {
        self.0.output_format = output_format;
        self
    }

    /// Runs the complex code snippets producer.
    pub fn run<P: AsRef<Path>>(self, source_path: P) -> Result<Option<Vec<Snippets>>> {
        // Check if output path is a directory.
        if self
            .0
            .output_path
            .map_or(false, |p| p.is_file() && !self.0.is_single_file)
        {
            return Err(Error::FormatPath("Output path MUST be a directory"));
        } else if self // Check if output path is a file.
            .0
            .output_path
            .map_or(false, |p| p.is_dir() && self.0.is_single_file)
        {
            return Err(Error::FormatPath("Output path MUST be a file"));
        }

        // Retrieve the number of available threads.
        let num_jobs = available_parallelism()?.get();

        // Define how to treat files.
        let files_data = FilesData {
            include: Self::mk_globset(self.0.include),
            exclude: Self::mk_globset(self.0.exclude),
            path: source_path.as_ref().to_path_buf(),
        };

        // Extracts and retrieves snippets concurrently.
        let snippets_context =
            ConcurrentRunner::new(extract_file_snippets, num_jobs, self.0.complexities)
                .run(files_data)?;

        // If there are no snippets, print a message informing that the code is
        // clean.
        if snippets_context.is_empty() {
            info!("Congratulations! Your code is clean, it does not have any complexity!");
            return Ok(None);
        }

        // Write files.
        if let Some(output_path) = self.0.output_path {
            self.0.output_format.write_output(
                output_path,
                self.0.is_single_file,
                &snippets_context,
            )?;
        }

        Ok(Some(snippets_context))
    }

    fn mk_globset(elems: Vec<String>) -> GlobSet {
        if elems.is_empty() {
            return GlobSet::empty();
        }
        let mut globset = GlobSetBuilder::new();
        elems.iter().filter(|e| !e.is_empty()).for_each(|e| {
            if let Ok(glob) = Glob::new(e) {
                globset.add(glob);
            }
        });
        globset.build().map_or(GlobSet::empty(), |globset| globset)
    }
}

fn extract_file_snippets(
    source_path: PathBuf,
    complexities: &[(Complexity, usize)],
) -> Option<Snippets> {
    // Read source file an return it as a sequence of bytes.
    let source_file_bytes = read_file_with_eol(&source_path).ok()??;

    // Convert source code bytes to an utf-8 string.
    // When the conversion is not possible for every bytes,
    // encode all bytes as utf-8.
    let source_file = match std::str::from_utf8(&source_file_bytes) {
        Ok(source_file) => source_file.to_owned(),
        Err(_) => encode_to_utf8(&source_file_bytes).ok()?,
    };

    // Guess which is the language associated to the source file.
    let language = guess_language(source_file.as_bytes(), &source_path).0?;

    // Get metrics values for each space which forms the source code.
    let spaces = get_function_spaces(
        &language,
        source_file.as_bytes().to_vec(),
        &source_path,
        None,
    )?;

    // Get code snippets for each metric and return them.
    get_code_snippets(
        spaces,
        language.into(),
        source_path,
        source_file.as_ref(),
        complexities,
    )
}
