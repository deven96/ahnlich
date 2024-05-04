/// PredicateOp are the various operations that can be conducted against a predicate value
#[derive(Debug, Copy, Clone)]
pub enum PredicateOp {
    Equals,
    NotEquals,
}

/// New types for Predicate key and values
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PredicateKey(String);
impl PredicateKey {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct PredicateValue(String);
impl PredicateValue {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}

/// Representation of how one predicate value and ops looks
/// to specify a predicate of name != "David", one would use the format
/// PredicateOp { key: "name", value: "David", op: NotEquals }
#[derive(Debug, Clone)]
pub struct Predicate {
    pub key: PredicateKey,
    pub value: PredicateValue,
    pub op: PredicateOp,
}

/// All possible representations of a predicate condition
/// We can only have a simple And or Or and we can combine those in any fashion
#[derive(Debug, Clone)]
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
