use crate::metadata::MetadataKey;
use crate::metadata::MetadataValue;
use serde::Deserialize;
use serde::Serialize;

/// PredicateOp are the various operations that can be conducted against a predicate value
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredicateOp {
    Equals,
    NotEquals,
}

/// Representation of how one predicate value and ops looks
/// to specify a predicate of name != "David", one would use the format
/// PredicateOp { key: "name", value: "David", op: NotEquals }
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Predicate {
    pub key: MetadataKey,
    pub value: MetadataValue,
    pub op: PredicateOp,
}

/// All possible representations of a predicate condition
/// We can only have a simple And or Or and we can combine those in any fashion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredicateCondition {
    And(Box<PredicateCondition>, Box<PredicateCondition>),
    Or(Box<PredicateCondition>, Box<PredicateCondition>),
    Value(Predicate),
}

impl PredicateCondition {
    pub fn and(self, other: PredicateCondition) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: PredicateCondition) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }
}
