use crate::error::DslError;
use crate::metadata::{parse_metadata_value, parse_metadata_values};
use crate::parser::Rule;
use grpc_types::metadata::MetadataValue;
use grpc_types::predicates::predicate::Kind as PredicateKind;
use grpc_types::predicates::predicate_condition::Kind;
use grpc_types::predicates::{
    AndCondition, Equals, In, NotEquals, NotIn, OrCondition, Predicate, PredicateCondition,
};
use pest::iterators::Pair;

pub(crate) fn parse_predicate_expression(pair: Pair<Rule>) -> Result<PredicateCondition, DslError> {
    match pair.as_rule() {
        Rule::simple_expression => {
            // Parse the simple expression into a Predicate
            let predicate = parse_simple_expression(pair)?;
            Ok(PredicateCondition {
                kind: Some(Kind::Value(predicate)),
            })
        }
        Rule::compound_expression => {
            let start_pos = pair.as_span().start_pos().pos();
            let end_pos = pair.as_span().end_pos().pos();
            let mut inner_pairs = pair.into_inner();
            let first_condition = parse_predicate_condition(
                inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?,
            )?;

            let mut result = first_condition;

            while let Some(op) = inner_pairs.next() {
                let operator = op.as_rule();
                let next_condition = parse_predicate_condition(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?,
                )?;

                result = match operator {
                    Rule::and => PredicateCondition {
                        kind: Some(Kind::And(Box::new(AndCondition {
                            left: Some(Box::new(result)),
                            right: Some(Box::new(next_condition)),
                        }))),
                    },
                    Rule::or => PredicateCondition {
                        kind: Some(Kind::Or(Box::new(OrCondition {
                            left: Some(Box::new(result)),
                            right: Some(Box::new(next_condition)),
                        }))),
                    },
                    _ => return Err(DslError::UnexpectedSpan((start_pos, end_pos))),
                };
            }

            Ok(result)
        }
        Rule::predicate_condition => {
            let start_pos = pair.as_span().start_pos().pos();
            let end_pos = pair.as_span().end_pos().pos();
            let mut inner_pairs = pair.into_inner();
            let simple_or_compound = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
            parse_predicate_expression(simple_or_compound)
        }
        _ => Err(DslError::UnexpectedSpan((
            pair.as_span().start_pos().pos(),
            pair.as_span().end_pos().pos(),
        ))),
    }
}

fn parse_simple_expression(pair: Pair<Rule>) -> Result<Predicate, DslError> {
    let start_pos = pair.as_span().start_pos().pos();
    let end_pos = pair.as_span().end_pos().pos();

    let mut inner_pairs = pair.into_inner();

    let key = inner_pairs
        .next()
        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
        .as_str();

    let operator = inner_pairs
        .next()
        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;

    let value = inner_pairs
        .next()
        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;

    let metadata_key = key.to_string();

    match operator.as_rule() {
        Rule::equals => {
            let metadata_value = parse_metadata_value(value)?;
            Ok(Predicate {
                kind: Some(PredicateKind::Equals(Equals {
                    key: metadata_key,
                    value: Some(MetadataValue {
                        value: Some(metadata_value),
                    }),
                })),
            })
        }
        Rule::in_op => {
            let values = parse_metadata_values(value)?;
            Ok(Predicate {
                kind: Some(PredicateKind::In(In {
                    key: metadata_key,
                    values,
                })),
            })
        }
        Rule::not_equals => {
            let metadata_value = parse_metadata_value(value)?;
            Ok(Predicate {
                kind: Some(PredicateKind::NotEquals(NotEquals {
                    key: metadata_key,
                    value: Some(MetadataValue {
                        value: Some(metadata_value),
                    }),
                })),
            })
        }
        Rule::not_in => {
            let value = parse_metadata_values(value)?;
            Ok(Predicate {
                kind: Some(PredicateKind::NotIn(NotIn {
                    key: metadata_key,
                    values: value.into_iter().collect(),
                })),
            })
        }
        _ => Err(DslError::UnexpectedSpan((start_pos, end_pos))),
    }
}

fn parse_predicate_condition(pair: Pair<Rule>) -> Result<PredicateCondition, DslError> {
    match pair.as_rule() {
        Rule::simple_expression => {
            // Parse the simple expression into a Predicate
            let predicate = parse_simple_expression(pair)?;
            Ok(PredicateCondition {
                kind: Some(Kind::Value(predicate)),
            })
        }
        Rule::compound_expression => {
            let start_pos = pair.as_span().start_pos().pos();
            let end_pos = pair.as_span().end_pos().pos();
            let mut inner_pairs = pair.into_inner();

            let first_condition = parse_predicate_condition(
                inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?,
            )?;

            let mut result = first_condition;

            while let Some(op) = inner_pairs.next() {
                let operator = op.as_rule();
                let next_condition = parse_predicate_condition(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?,
                )?;

                result = match operator {
                    Rule::and => PredicateCondition {
                        kind: Some(Kind::And(Box::new(AndCondition {
                            left: Some(Box::new(result)),
                            right: Some(Box::new(next_condition)),
                        }))),
                    },
                    Rule::or => PredicateCondition {
                        kind: Some(Kind::Or(Box::new(OrCondition {
                            left: Some(Box::new(result)),
                            right: Some(Box::new(next_condition)),
                        }))),
                    },
                    _ => return Err(DslError::UnexpectedSpan((start_pos, end_pos))),
                };
            }

            Ok(result)
        }
        _ => Err(DslError::UnexpectedSpan((
            pair.as_span().start_pos().pos(),
            pair.as_span().end_pos().pos(),
        ))),
    }
}
