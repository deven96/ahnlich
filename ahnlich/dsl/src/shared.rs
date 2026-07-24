use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use pest::iterators::Pair;

use crate::{algorithm::to_non_linear, error::DslError, parser::Rule};

pub(crate) fn parse_schema_clause(pair: Pair<Rule>) -> Result<String, DslError> {
    match pair.as_rule() {
        Rule::schema_clause => {
            let start_pos = pair.as_span().start_pos().pos();
            let end_pos = pair.as_span().end_pos().pos();
            Ok(pair
                .into_inner()
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str()
                .to_string())
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}

pub(crate) fn parse_drop_store(
    statement: Pair<Rule>,
) -> Result<(String, bool, Option<String>), DslError> {
    match statement.as_rule() {
        Rule::drop_store => {
            let start_pos = statement.as_span().start_pos().pos();
            let end_pos = statement.as_span().end_pos().pos();
            let mut inner_pairs = statement.into_inner();
            let store = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str()
                .to_string();
            let mut if_exists = false;
            let mut schema = None;
            for pair in inner_pairs {
                match pair.as_rule() {
                    Rule::if_exists => if_exists = true,
                    Rule::schema_clause => schema = Some(parse_schema_clause(pair)?),
                    _ => {
                        let start_pos = pair.as_span().start_pos().pos();
                        let end_pos = pair.as_span().end_pos().pos();
                        return Err(DslError::UnexpectedSpan((start_pos, end_pos)));
                    }
                }
            }
            Ok((store, !if_exists, schema))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}

pub(crate) fn parse_drop_non_linear_algorithm_index(
    statement: Pair<Rule>,
) -> Result<(String, bool, Vec<NonLinearAlgorithm>, Option<String>), DslError> {
    match statement.as_rule() {
        Rule::drop_non_linear_algorithm_index => {
            let start_pos = statement.as_span().start_pos().pos();
            let end_pos = statement.as_span().end_pos().pos();
            let mut inner_pairs = statement.into_inner().peekable();
            let mut if_exists = false;
            if let Some(next_pair) = inner_pairs.peek()
                && next_pair.as_rule() == Rule::if_exists
            {
                inner_pairs.next(); // Consume rule
                if_exists = true;
            };
            let index_names_pair = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
            let store = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str()
                .to_string();
            let schema = inner_pairs.next().map(parse_schema_clause).transpose()?;
            let non_linear_indices = index_names_pair
                .into_inner()
                .flat_map(|index_pair| to_non_linear(index_pair.as_str()))
                .collect();
            Ok((store, !if_exists, non_linear_indices, schema))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}

pub(crate) fn parse_drop_pred_index(
    statement: Pair<Rule>,
) -> Result<(String, Vec<String>, bool, Option<String>), DslError> {
    match statement.as_rule() {
        Rule::drop_pred_index => {
            let start_pos = statement.as_span().start_pos().pos();
            let end_pos = statement.as_span().end_pos().pos();
            let mut inner_pairs = statement.into_inner().peekable();
            let mut if_exists = false;
            if let Some(next_pair) = inner_pairs.peek()
                && next_pair.as_rule() == Rule::if_exists
            {
                inner_pairs.next();
                if_exists = true;
            };
            let index_names_pair = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
            let store = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str()
                .to_string();
            let schema = inner_pairs.next().map(parse_schema_clause).transpose()?;
            let predicates = index_names_pair
                .into_inner()
                .map(|index_pair| index_pair.as_str().to_string())
                .collect();
            Ok((store, predicates, !if_exists, schema))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}

pub(crate) fn parse_create_non_linear_algorithm_index(
    statement: Pair<Rule>,
) -> Result<(String, Vec<NonLinearAlgorithm>, Option<String>), DslError> {
    match statement.as_rule() {
        Rule::create_non_linear_algorithm_index => {
            let start_pos = statement.as_span().start_pos().pos();
            let end_pos = statement.as_span().end_pos().pos();
            let mut inner_pairs = statement.into_inner();
            let index_name_pairs = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
            let non_linear_indices = index_name_pairs
                .into_inner()
                .flat_map(|index_pair| to_non_linear(index_pair.as_str()))
                .collect();
            let store = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str()
                .to_string();
            let schema = inner_pairs.next().map(parse_schema_clause).transpose()?;
            Ok((store, non_linear_indices, schema))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}

pub(crate) fn parse_create_pred_index(
    statement: Pair<Rule>,
) -> Result<(String, Vec<String>, Option<String>), DslError> {
    match statement.as_rule() {
        Rule::create_pred_index => {
            let start_pos = statement.as_span().start_pos().pos();
            let end_pos = statement.as_span().end_pos().pos();
            let mut inner_pairs = statement.into_inner();
            let index_name_pairs = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
            let predicates = index_name_pairs
                .into_inner()
                .map(|index_pair| index_pair.as_str().to_string())
                .collect();
            let store = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str()
                .to_string();
            let schema = inner_pairs.next().map(parse_schema_clause).transpose()?;
            Ok((store, predicates, schema))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}
