use std::num::ParseIntError;

use crate::parser::Rule;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DslError {
    #[error("Error parsing rule {0}")]
    RuleParse(#[from] Box<pest::error::Error<Rule>>),
    #[error("Unexpected Statement found {0:?}")]
    UnexpectedSpan((usize, usize)),
    #[error("Could not parse Hex string into image {0:?}")]
    UnexpectedHex(String),
    #[error("Could not parse string into nonzerousize {0:?}")]
    NonZeroUsizeParse(#[from] ParseIntError),
    #[error("Found unsupported algorithm {0}")]
    UnsupportedAlgorithm(String),
    #[error("Found unsupported ai model {0}")]
    UnsupportedAIModel(String),
    #[error("Unsupported rule used in parse fn {0:?}")]
    UnsupportedRule(Rule),
}
