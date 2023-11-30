use std::collections::HashMap;
use std::fmt;
use std::fs::{create_dir_all, write, File};
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use minijinja::value::Value;
use minijinja::Environment;

use tracing::debug;

use crate::Snippets;
use crate::{Error, Result};

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

static OUTPUT_TEMPLATES: &[(&str, &str)] = &builtin_templates![("md.markdown", "markdown.md")];

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

    pub(crate) fn write_format(&self, output_path: &Path, snippets: &[Snippets]) -> Result<()> {
        // Create output filenames.
        let filenames = create_filenames(snippets);

        match self {
            Self::All => {
                let environment = more_templates_environment()?;
                Markdown::write_template(output_path, &filenames, snippets, &environment)?;
                Html::write_format(output_path, &filenames, snippets)?;
                Json::write_format(output_path, &filenames, snippets)
            }
            Self::Json => Json::write_format(output_path, &filenames, snippets),
            Self::Markdown => Markdown::write_template(
                output_path,
                &filenames,
                snippets,
                &single_template_environment(Markdown::TEMPLATE)?,
            ),
            Self::Html => Html::write_format(output_path, &filenames, snippets),
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

fn more_templates_environment() -> Result<Environment<'static>> {
    let mut environment = Environment::new();
    for (template_name, template_file) in OUTPUT_TEMPLATES {
        environment.add_template(template_name, template_file)?;
    }
    Ok(environment)
}

fn single_template_environment(
    template: (&'static str, &'static str),
) -> Result<Environment<'static>> {
    let mut environment = Environment::new();
    let (template_name, template_file) = template;
    environment.add_template(template_name, template_file)?;
    Ok(environment)
}

trait WriteFormat {
    const EXTENSION: &'static str;
    const DIR: &'static str;

    fn write_format(path: &Path, filenames: &[String], snippets: &[Snippets]) -> Result<()>;
}

trait WriteTemplateFormat {
    const EXTENSION: &'static str;
    const DIR: &'static str;
    const TEMPLATE: (&'static str, &'static str);

    fn write_template(
        path: &Path,
        filenames: &[String],
        snippets: &[Snippets],
        environment: &Environment,
    ) -> Result<()>;
}

struct Markdown;

impl WriteTemplateFormat for Markdown {
    const EXTENSION: &'static str = "md";
    const DIR: &'static str = "markdown";
    const TEMPLATE: (&'static str, &'static str) = ("md.markdown", "markdown.md");

    fn write_template(
        path: &Path,
        filenames: &[String],
        snippets: &[Snippets],
        environment: &Environment,
    ) -> Result<()> {
        let template = environment.get_template("md.markdown")?;

        let dir = create_dir(path, Self::DIR)?;

        for (filename, snippet) in filenames.iter().zip(snippets) {
            let mut context = HashMap::new();
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
        }

        Ok(())
    }
}

struct Html;

impl WriteFormat for Html {
    const EXTENSION: &'static str = "html";
    const DIR: &'static str = "html";

    fn write_format(path: &Path, filenames: &[String], snippets: &[Snippets]) -> Result<()> {
        let dir = create_dir(path, Self::DIR)?;

        let mut index_body = Vec::new();

        for (filename, snippet) in filenames.iter().zip(snippets) {
            let final_path = dir.join(filename).with_extension(Self::EXTENSION);
            debug!("Creating {:?}", final_path);

            let mut html_file = File::create(&final_path)?;

            index_body.push(format!(
                "<a href=\"{index_path}\" target=\"_blank\">{index_path}</a><br>",
                index_path = final_path
                    .file_name()
                    .ok_or_else(|| Error::FormatPath(format!(
                        "Error getting filename for {:?}",
                        final_path
                    )))?
                    .to_str()
                    .ok_or_else(|| Error::FormatPath(format!(
                        "Error converting {:?} path to str",
                        final_path
                    )))?
            ));

            let title = path
                .file_name()
                .map_or("Unknown file", |os| os.to_str().unwrap_or("Unknown file"));
            let body = snippet
                .snippets
                .iter()
                .map(|(complexity_name, all_snippets)| {
                    format!(
                        r#"<h1>{complexity_name}</h1>{snippet}"#,
                        snippet = all_snippets
                            .iter()
                            .map(|v| {
                                format!(
                                    r#"
<p>
    complexity: <b>{complexity}</b><br>
    start line: <b>{start_line}</b><br>
    end line: <b>{end_line}</b><br>
    <pre><code>{text}
    </code></pre>
</p>"#,
                                    complexity = v.complexity,
                                    start_line = v.start_line,
                                    end_line = v.end_line,
                                    text = html_escape::encode_text(&v.text),
                                )
                            })
                            .collect::<Vec<String>>()
                            .join("\n\n")
                    )
                })
                .collect::<Vec<String>>()
                .join("\n\n");
            writeln!(
                html_file,
                r#"<!DOCTYPE html>
<html>
<head>
    <title>{title}</title>
</head>
<body>
    {body}
</body>
</html>"#
            )?;
        }

        let mut index_file = File::create(dir.join("index.html"))?;
        writeln!(
            index_file,
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Index</title>
</head>
<body>
    {index_body}
</body>
</html>"#,
            index_body = index_body.join("\n")
        )?;
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
