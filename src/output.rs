use std::fmt;
use std::fs::{create_dir_all, write, File};
use std::ops::RangeInclusive;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use minijinja::{context, Environment};

use tracing::debug;

use crate::Result;
use crate::Snippets;

// Builtin template macro to retrieve a template
macro_rules! builtin_templates {
    ($($name:expr),+) => {
        [
        $(
            (
                $name,
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/templates/", $name)),
            )
        ),+
        ]
    }
}

static OUTPUT_TEMPLATES: &[(&str, &str)] =
    &builtin_templates!["markdown.md", "index.html", "web.html"];

/// Supported output formats.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum OutputFormat {
    /// Json format.
    #[default]
    Json,
    /// Markdown format.
    Markdown,
    /// Html format.
    Html,
    /// Enables all supported output formats.
    All,
}

impl FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "json" => Ok(Self::Json),
            "markdown" => Ok(Self::Markdown),
            "html" => Ok(Self::Html),
            "all" => Ok(Self::All),
            _ => Err(format!("Unknown output format: {s}")),
        }
    }
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Json => "json",
            Self::Markdown => "markdown",
            Self::Html => "html",
            Self::All => "all",
        };
        s.fmt(f)
    }
}

impl OutputFormat {
    /// Default output format.
    pub const fn all() -> &'static [&'static str] {
        &["json", "markdown", "html", "all"]
    }

    pub(crate) fn write_output<P: AsRef<Path>>(
        &self,
        output_path: P,
        snippets: &[Snippets],
    ) -> Result<()> {
        // Get path
        let output_path = output_path.as_ref();

        // Create output filenames.
        let filenames = create_filenames(snippets);

        match self {
            Self::All => {
                let environment =
                    produce_templates_environment(RangeInclusive::new(0, OUTPUT_TEMPLATES.len()))?;
                Markdown::write_template(output_path, &filenames, snippets, &environment)?;
                Html::write_template(output_path, &filenames, snippets, &environment)?;
                Json::write_format(output_path, &filenames, snippets)
            }
            Self::Json => Json::write_format(output_path, &filenames, snippets),
            Self::Markdown => Markdown::write_template(
                output_path,
                &filenames,
                snippets,
                &produce_templates_environment(Markdown::TEMPLATE)?,
            ),
            Self::Html => Html::write_template(
                output_path,
                &filenames,
                snippets,
                &produce_templates_environment(Html::TEMPLATE)?,
            ),
        }
    }
}

fn create_filenames(snippets: &[Snippets]) -> Vec<String> {
    snippets
        .iter()
        .map(|s| {
            s.source_path
                .iter()
                .filter_map(|c| {
                    c.to_str()
                        .map(|s| (![".", "..", ":", "/", "\\"].contains(&s)).then_some(s))
                })
                .flatten()
                .collect::<Vec<&str>>()
                .join("_")
        })
        .collect()
}

#[inline(always)]
fn create_dir(path: &Path, dir: &str) -> Result<PathBuf> {
    let dir = path.join(dir);
    debug!("Creating {:?}", dir);
    create_dir_all(&dir)?;
    Ok(dir)
}

#[inline(always)]
fn create_file<W>(path: PathBuf, extension: &str, writer: W) -> Result<()>
where
    W: FnOnce(PathBuf) -> Result<()>,
{
    let final_path = path.with_extension(extension);
    debug!("Creating {:?}", final_path);
    writer(final_path)
}

#[inline(always)]
fn produce_templates_environment(range: RangeInclusive<usize>) -> Result<Environment<'static>> {
    let mut environment = Environment::new();
    for (template_name, template_file) in OUTPUT_TEMPLATES[range].iter() {
        environment.add_template(template_name, template_file)?;
    }
    Ok(environment)
}

trait WriteFormat {
    const EXTENSION: &'static str;
    const DIR: &'static str;

    fn write_format(path: &Path, filenames: &[String], snippets: &[Snippets]) -> Result<()>;
}

trait WriteTemplate {
    const EXTENSION: &'static str;
    const DIR: &'static str;
    const TEMPLATE: RangeInclusive<usize>;

    fn write_template(
        path: &Path,
        filenames: &[String],
        snippets: &[Snippets],
        environment: &Environment,
    ) -> Result<()>;
}

struct Markdown;

impl WriteTemplate for Markdown {
    const EXTENSION: &'static str = "md";
    const DIR: &'static str = "markdown";
    const TEMPLATE: RangeInclusive<usize> = RangeInclusive::new(0, 0);

    fn write_template(
        path: &Path,
        filenames: &[String],
        snippets: &[Snippets],
        environment: &Environment,
    ) -> Result<()> {
        let dir = create_dir(path, Self::DIR)?;

        let template = environment.get_template(OUTPUT_TEMPLATES[*Self::TEMPLATE.start()].0)?;

        for (filename, snippet) in filenames.iter().zip(snippets) {
            // Fill template
            let filled_template = template.render(context! {
                language => snippet.language.name(),
                snippets => snippet.snippets
            })?;

            // Write filled template in a new file
            create_file(dir.join(filename), Self::EXTENSION, |path| {
                write(path, filled_template).map_err(|e| e.into())
            })?;
        }

        Ok(())
    }
}

struct Html;

fn index_template(
    filenames: &[String],
    dir: &Path,
    template_name: &str,
    extension: &str,
    environment: &Environment,
) -> Result<()> {
    let mut files = filenames
        .iter()
        .filter_map(|filename| {
            if let Some(filename) = Path::new(filename).with_extension(extension).file_name() {
                filename.to_str().map(|p| p.to_string())
            } else {
                None
            }
        })
        .collect::<Vec<String>>();

    // Sort filenames
    files.sort_unstable();

    let template = environment.get_template(template_name)?;

    // Fill template
    let filled_template = template.render(context! {
        directory => dir,
        files => files
    })?;

    // Write filled template in a new file
    create_file(dir.join(template_name), extension, |path| {
        write(path, filled_template).map_err(|e| e.into())
    })?;

    Ok(())
}

impl WriteTemplate for Html {
    const EXTENSION: &'static str = "html";
    const DIR: &'static str = "html";
    const TEMPLATE: RangeInclusive<usize> = RangeInclusive::new(1, 2);

    fn write_template(
        path: &Path,
        filenames: &[String],
        snippets: &[Snippets],
        environment: &Environment,
    ) -> Result<()> {
        // Create directory
        let dir = create_dir(path, Self::DIR)?;

        // Create index
        index_template(
            filenames,
            &dir,
            OUTPUT_TEMPLATES[*Self::TEMPLATE.start()].0,
            Self::EXTENSION,
            environment,
        )?;

        // Retrieve template
        let template = environment.get_template(OUTPUT_TEMPLATES[*Self::TEMPLATE.end()].0)?;

        for (filename, snippet) in filenames.iter().zip(snippets) {
            let mut all_snippets = snippet
                .snippets
                .iter()
                .collect::<Vec<(&crate::Complexity, &Vec<crate::snippets::SnippetData>)>>();

            // Sort by complexities
            all_snippets.sort_by(|a, b| a.0.cmp(b.0));

            // Fill template
            let filled_template = template.render(context! { snippets => all_snippets })?;

            // Write filled template in a new file
            create_file(dir.join(filename), Self::EXTENSION, |path| {
                write(path, filled_template).map_err(|e| e.into())
            })?;
        }

        Ok(())
    }
}

struct Json;

impl WriteFormat for Json {
    const EXTENSION: &'static str = "json";
    const DIR: &'static str = "json";

    fn write_format(path: &Path, filenames: &[String], snippets: &[Snippets]) -> Result<()> {
        let dir = create_dir(path, Self::DIR)?;

        for (filename, snippet) in filenames.iter().zip(snippets) {
            create_file(dir.join(filename), Self::EXTENSION, |path| {
                let json_file = File::create(path)?;
                serde_json::to_writer_pretty(json_file, snippet).map_err(|e| e.into())
            })?;
        }
        Ok(())
    }
}
