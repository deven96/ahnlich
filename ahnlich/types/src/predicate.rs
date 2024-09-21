use crate::metadata::MetadataKey;
use crate::metadata::MetadataValue;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

/// Representation of how one predicate value and ops looks
/// to specify a predicate of name != "David", one would use the format
/// PredicateOp { key: "name", value: "David", op: NotEquals }
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Predicate {
    Equals {
        key: MetadataKey,
        value: MetadataValue,
    },
    NotEquals {
        key: MetadataKey,
        value: MetadataValue,
    },
    In {
        key: MetadataKey,
        value: HashSet<MetadataValue>,
    },
    NotIn {
        key: MetadataKey,
        value: HashSet<MetadataValue>,
    },
}

impl Predicate {
    pub fn get_key(&self) -> &MetadataKey {
        match self {
            Predicate::Equals { key, .. } => key,
            Predicate::NotEquals { key, .. } => key,
            Predicate::In { key, .. } => key,
            Predicate::NotIn { key, .. } => key,
        }
    }
}
impl std::fmt::Debug for Predicate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Predicate::In { key, .. } => f.write_fmt(format_args!("In {key}")),
            Predicate::NotIn { key, .. } => f.write_fmt(format_args!("NotIn {key}")),
            Predicate::Equals { key, .. } => f.write_fmt(format_args!("Equals {key}")),
            Predicate::NotEquals { key, .. } => f.write_fmt(format_args!("NotEquals {key}")),
        }
    }
}

/// All possible representations of a predicate condition
/// We can only have a simple And or Or and we can combine those in any fashion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PredicateCondition {
    Value(Predicate),
    And(Box<PredicateCondition>, Box<PredicateCondition>),
    Or(Box<PredicateCondition>, Box<PredicateCondition>),
}

impl PredicateCondition {
    pub fn and(self, other: PredicateCondition) -> Self {
        Self::And(Box::new(self), Box::new(other))
    }

    pub fn or(self, other: PredicateCondition) -> Self {
        Self::Or(Box::new(self), Box::new(other))
    }
}
