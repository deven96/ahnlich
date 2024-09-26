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
}
