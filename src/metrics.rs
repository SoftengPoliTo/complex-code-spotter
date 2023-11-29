use std::fmt;
use std::io::{Error, ErrorKind};
use std::str::FromStr;

use rust_code_analysis::FuncSpace;
use serde::{Deserialize, Serialize};

trait ComplexityChecker {
    fn value(space: &FuncSpace, threshold: usize) -> Option<usize>;

    #[inline(always)]
    fn compute(metric_value: usize, metric_max_value: usize, threshold: usize) -> Option<usize> {
        (metric_value > threshold || metric_max_value > threshold).then_some(metric_value)
    }
}

struct Cyclomatic;

impl ComplexityChecker for Cyclomatic {
    #[inline(always)]
    fn value(space: &FuncSpace, threshold: usize) -> Option<usize> {
        Self::compute(
            space.metrics.cyclomatic.cyclomatic() as usize,
            space.metrics.cyclomatic.cyclomatic_max() as usize,
            threshold,
        )
    }
}

struct Cognitive;

impl ComplexityChecker for Cognitive {
    #[inline(always)]
    fn value(space: &FuncSpace, threshold: usize) -> Option<usize> {
        Self::compute(
            space.metrics.cognitive.cognitive() as usize,
            space.metrics.cognitive.cognitive_max() as usize,
            threshold,
        )
    }
}

/// Supported complexities metrics.
#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Complexity {
    /// Cyclomatic metric.
    #[default]
    Cyclomatic,
    /// Cognitive metric.
    Cognitive,
}

impl FromStr for Complexity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cyclomatic" => Ok(Self::Cyclomatic),
            "cognitive" => Ok(Self::Cognitive),
            _ => Err(Error::new(
                ErrorKind::Other,
                format!("Unknown complexity metric: {s}"),
            )),
        }
    }
}

impl fmt::Display for Complexity {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Cyclomatic => "cyclomatic",
            Self::Cognitive => "cognitive",
        };
        s.fmt(f)
    }
}

impl Complexity {
    /// Default threshold for a metric.
    pub const fn default_threshold(&self) -> usize {
        match self {
            Self::Cyclomatic => 15,
            Self::Cognitive => 15,
        }
    }
    /// All complexity metrics.
    pub const fn all() -> &'static [Complexity] {
        &[Self::Cyclomatic, Self::Cognitive]
    }

    pub(crate) fn value(&self, space: &FuncSpace, threshold: usize) -> Option<usize> {
        match self {
            Self::Cyclomatic => Cyclomatic::value(space, threshold),
            Self::Cognitive => Cognitive::value(space, threshold),
        }
    }
}
