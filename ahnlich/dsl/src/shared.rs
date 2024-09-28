use std::collections::HashSet;

use ahnlich_types::{keyval::StoreName, metadata::MetadataKey, similarity::NonLinearAlgorithm};
use pest::iterators::Pair;

use crate::{algorithm::to_non_linear, error::DslError, parser::Rule};

pub(crate) fn parse_drop_store(statement: Pair<Rule>) -> Result<(StoreName, bool), DslError> {
    match statement.as_rule() {
        Rule::drop_store => {
            let start_pos = statement.as_span().start_pos().pos();
            let end_pos = statement.as_span().end_pos().pos();
            let mut inner_pairs = statement.into_inner();
            let store = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str();
            let if_exists = match inner_pairs.next() {
                None => false,
                Some(p) => {
                    if p.as_str().trim().to_lowercase() != "if exists" {
                        let start_pos = p.as_span().start_pos().pos();
                        let end_pos = p.as_span().end_pos().pos();
                        return Err(DslError::UnexpectedSpan((start_pos, end_pos)));
                    }
                    true
                }
            };
            Ok((StoreName(store.to_string()), !if_exists))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}

pub(crate) fn parse_drop_non_linear_algorithm_index(
    statement: Pair<Rule>,
) -> Result<(StoreName, bool, HashSet<NonLinearAlgorithm>), DslError> {
    match statement.as_rule() {
        Rule::drop_non_linear_algorithm_index => {
            let start_pos = statement.as_span().start_pos().pos();
            let end_pos = statement.as_span().end_pos().pos();
            let mut inner_pairs = statement.into_inner().peekable();
            let mut if_exists = false;
            if let Some(next_pair) = inner_pairs.peek() {
                if next_pair.as_rule() == Rule::if_exists {
                    inner_pairs.next(); // Consume rule
                    if_exists = true;
                }
            };
            let index_names_pair = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
            let store = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str();
            let non_linear_indices = index_names_pair
                .into_inner()
                .flat_map(|index_pair| to_non_linear(index_pair.as_str()))
                .collect();
            Ok((StoreName(store.to_string()), !if_exists, non_linear_indices))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}

pub(crate) fn parse_drop_pred_index(
    statement: Pair<Rule>,
) -> Result<(StoreName, HashSet<MetadataKey>, bool), DslError> {
    match statement.as_rule() {
        Rule::drop_pred_index => {
            let start_pos = statement.as_span().start_pos().pos();
            let end_pos = statement.as_span().end_pos().pos();
            let mut inner_pairs = statement.into_inner().peekable();
            let mut if_exists = false;
            if let Some(next_pair) = inner_pairs.peek() {
                if next_pair.as_rule() == Rule::if_exists {
                    inner_pairs.next();
                    if_exists = true;
                }
            };
            let index_names_pair = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
            let store = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str();
            let predicates = index_names_pair
                .into_inner()
                .map(|index_pair| MetadataKey::new(index_pair.as_str().to_string()))
                .collect();
            Ok((StoreName(store.to_string()), predicates, !if_exists))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}

pub(crate) fn parse_create_non_linear_algorithm_index(
    statement: Pair<Rule>,
) -> Result<(StoreName, HashSet<NonLinearAlgorithm>), DslError> {
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
                .as_str();
            Ok((StoreName(store.to_string()), non_linear_indices))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}

pub(crate) fn parse_create_pred_index(
    statement: Pair<Rule>,
) -> Result<(StoreName, HashSet<MetadataKey>), DslError> {
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
                .map(|index_pair| MetadataKey::new(index_pair.as_str().to_string()))
                .collect();
            let store = inner_pairs
                .next()
                .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                .as_str();
            Ok((StoreName(store.to_string()), predicates))
        }
        e => Err(DslError::UnsupportedRule(e)),
    }
}
