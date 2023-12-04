use std::collections::HashMap;
use std::fmt;
use std::fs::{create_dir_all, write, File};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use minijinja::value::Value;
use minijinja::Environment;

use tracing::debug;

use crate::Result;
use crate::Snippets;

// Builtin template macro to retrieve a template
macro_rules! builtin_templates {
    ($(($name:expr, $template:expr)),+) => {
        [
        $(
            (
                $name,
                include_str!(concat!(env!("CARGO_MANIFEST_DIR"),"/templates/", $template)),
            )
        ),+
        ]
    }
}

static OUTPUT_TEMPLATES: &[(&str, &str)] = &builtin_templates![
    ("md.markdown", "markdown.md"),
    ("html.index", "index.html"),
    ("html.web", "web.html")
];

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
                let environment = produce_templates_environment(OUTPUT_TEMPLATES)?;
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
fn produce_templates_environment(
    templates: &'static [(&'static str, &'static str)],
) -> Result<Environment<'static>> {
    let mut environment = Environment::new();
    for (template_name, template_file) in templates {
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
    const TEMPLATE: &'static [(&'static str, &'static str)];

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
    const TEMPLATE: &'static [(&'static str, &'static str)] = &[("md.markdown", "markdown.md")];

    fn write_template(
        path: &Path,
        filenames: &[String],
        snippets: &[Snippets],
        environment: &Environment,
    ) -> Result<()> {
        let dir = create_dir(path, Self::DIR)?;

        let template = environment.get_template(Self::TEMPLATE[0].0)?;

        let mut context = HashMap::new();

        for (filename, snippet) in filenames.iter().zip(snippets) {
            context.insert(
                "language",
                Value::from_serializable(&snippet.language.name()),
            );
            context.insert("snippets", Value::from_serializable(&snippet.snippets));

            // Fill template
            let filled_template = template.render(&context)?;

            // Write filled template in a new file
            create_file(dir.join(filename), Self::EXTENSION, |path| {
                write(path, filled_template).map_err(|e| e.into())
            })?;

            // Clear context
            context.clear();
        }

        Ok(())
    }
}

struct Html;

fn index_template(
    filenames: &[String],
    dir: &Path,
    template_data: (&str, &str),
    extension: &str,
    environment: &Environment,
) -> Result<()> {
    let files = filenames
        .iter()
        .map(|filename| {
            if let Some(filename) = Path::new(filename).with_extension(extension).file_name() {
                filename.to_str().map(|p| p.to_string())
            } else {
                None
            }
        })
        .flatten()
        .collect::<Vec<String>>();

    let mut context = HashMap::new();
    context.insert("directory", Value::from_serializable(&dir));
    context.insert("files", Value::from_serializable(&files));

    println!("{:?}", context);

    println!("{}", template_data.0);

    let template = environment.get_template(template_data.0)?;

    println!("{:?}", template);

    // Fill template
    let filled_template = template.render(&context)?;

    println!("{:?}", filled_template);

    // Write filled template in a new file
    create_file(dir.join(template_data.1), extension, |path| {
        write(path, filled_template).map_err(|e| e.into())
    })?;

    Ok(())
}

impl WriteTemplate for Html {
    const EXTENSION: &'static str = "html";
    const DIR: &'static str = "html";
    const TEMPLATE: &'static [(&'static str, &'static str)] =
        &[("html.index", "index.html"), ("html.web", "web.html")];

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
            Self::TEMPLATE[0],
            Self::EXTENSION,
            environment,
        )?;

        /*for (filename, snippet) in filenames.iter().zip(snippets) {
            println!();
        }*/

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
