use crate::db::Rule as DBRule;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DslError {
    #[error("Error parsing DB rule {0}")]
    DBRuleParse(#[from] Box<pest::error::Error<DBRule>>),
    #[error("Unexpected Statement found {0:?}")]
    UnexpectedSpan((usize, usize)),
}
