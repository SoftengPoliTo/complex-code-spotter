use std::borrow::Cow;

use thiserror::Error;

/// Error types.
#[derive(Debug, Error)]
pub enum Error {
    /// Wrong file content.
    #[error("Wrong content")]
    WrongContent,
    /// Impossible to guess a programming language.
    #[error("Impossible to guess the programming language")]
    UnknownLanguage,
    /// Impossible to retrieve function spaces.
    #[error("Impossible to retrieve function spaces")]
    NoSpaces,
    /// A general utf-8 conversion error.
    #[error("Utf-8 error")]
    Utf8(#[from] std::str::Utf8Error),
    /// Impossible to complete a non-utf8 conversion.
    #[error("Impossible to complete a non-utf8 conversion")]
    NonUtf8Conversion,
    /// Path format.
    #[error("{0}")]
    FormatPath(&'static str),
    /// Concurrent failures.
    #[error("Concurrent failure: {0}")]
    Concurrent(Cow<'static, str>),
    /// Mutability access failures.
    #[error("Mutability failure: {0}")]
    Mutability(Cow<'static, str>),
    /// Less thresholds than complexity metrics.
    #[error("Each complexity metric MUST have a threshold.")]
    Thresholds,
    /// A more generic I/O error.
    #[error("I/O error")]
    Io(#[from] std::io::Error),
    #[error("Template error")]
    /// A template error.
    Template(#[from] minijinja::Error),
    #[error("Json error")]
    /// A Json output error.
    JsonOutput(#[from] serde_json::Error),
}

impl From<Box<dyn std::any::Any + Send>> for Error {
    fn from(_e: Box<dyn std::any::Any + Send>) -> Self {
        Error::Concurrent("Producer: child thread panicked".into())
    }
}

impl<T> From<std::sync::PoisonError<T>> for Error {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        Self::Mutability(Cow::from(e.to_string()))
    }
}

/// A specialized `Result` type.
pub type Result<T> = ::std::result::Result<T, Error>;
