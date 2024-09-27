use crate::error::DslError;
use crate::metadata::{parse_metadata_value, parse_metadata_values};
use crate::parser::Rule;
use ahnlich_types::metadata::MetadataKey;
use ahnlich_types::predicate::Predicate;
use ahnlich_types::predicate::PredicateCondition;
use pest::iterators::Pair;

pub(crate) fn parse_predicate_expression(pair: Pair<Rule>) -> Result<PredicateCondition, DslError> {
    match pair.as_rule() {
        Rule::simple_expression => {
            // Parse the simple expression into a Predicate
            let predicate = parse_simple_expression(pair)?;
            Ok(PredicateCondition::Value(predicate))
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
                    Rule::and => {
                        PredicateCondition::And(Box::new(result), Box::new(next_condition))
                    }
                    Rule::or => PredicateCondition::Or(Box::new(result), Box::new(next_condition)),
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

    let metadata_key = MetadataKey::new(key.to_string());

    match operator.as_rule() {
        Rule::equals => {
            let metadata_value = parse_metadata_value(value)?;
            Ok(Predicate::Equals {
                key: metadata_key,
                value: metadata_value,
            })
        }
        Rule::in_op => {
            let value = parse_metadata_values(value)?;
            Ok(Predicate::In {
                key: metadata_key,
                value,
            })
        }
        Rule::not_equals => {
            let metadata_value = parse_metadata_value(value)?;
            Ok(Predicate::NotEquals {
                key: metadata_key,
                value: metadata_value,
            })
        }
        Rule::not_in => {
            let value = parse_metadata_values(value)?;
            Ok(Predicate::NotIn {
                key: metadata_key,
                value,
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
            Ok(PredicateCondition::Value(predicate))
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
                    Rule::and => {
                        PredicateCondition::And(Box::new(result), Box::new(next_condition))
                    }
                    Rule::or => PredicateCondition::Or(Box::new(result), Box::new(next_condition)),
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
