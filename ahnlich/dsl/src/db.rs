use std::{collections::HashSet, num::NonZeroUsize};

use crate::{
    algorithm::{to_algorithm, to_non_linear},
    parser::{QueryParser, Rule},
};
use ahnlich_types::{
    db::DBQuery,
    keyval::{StoreKey, StoreName},
    metadata::MetadataKey,
};
use ndarray::Array1;
use pest::iterators::Pair;
use pest::Parser;

use crate::{error::DslError, predicate::parse_predicate_expression};

fn parse_multi_f32_array(f32_arrays_pair: Pair<Rule>) -> Vec<StoreKey> {
    f32_arrays_pair.into_inner().map(parse_f32_array).collect()
}

fn parse_f32_array(pair: Pair<Rule>) -> StoreKey {
    StoreKey(Array1::from_iter(pair.into_inner().map(|f32_pair| {
        f32_pair
            .as_str()
            .parse::<f32>()
            .expect("Cannot parse single f32 num")
    })))
}

// Parse raw strings separated by ; into a Vec<DBQuery>. Examples include but are not restricted
// to
//
// PING
// LISTCLIENTS
// LISTSTORES
// INFOSERVER
// DROPSTORE store_name IF EXISTS
// CREATEPREDINDEX (key_1, key_2) in store_name
// DROPPREDINDEX IF EXISTS (key1, key2) in store_name
// CREATENONLINEARALGORITHMINDEX (kdtree) in store_name
// DROPNONLINEARALGORITHMINDEX IF EXISTS (kdtree) in store_name
// GETKEY ([1.0, 2.0], [3.0, 4.0]) IN my_store
// DELKEY ([1.2, 3.0], [5.6, 7.8]) IN my_store
// GETPRED ((author = dickens) OR (country != Nigeria)) IN my_store
// GETSIMN 4 WITH [0.65, 2.78] USING cosinesimilarity IN my_store WHERE (author = dickens)
// CREATESTORE IF NOT EXISTS my_store DIMENSION 21 PREDICATES (author, country) NONLINEARALGORITHMINDEX (kdtree)
//
// #TODO
// SET
pub fn parse_db_query(input: &str) -> Result<Vec<DBQuery>, DslError> {
    let pairs = QueryParser::parse(Rule::db_query, input).map_err(Box::new)?;
    let statements = pairs.into_iter().collect::<Vec<_>>();
    let mut queries = Vec::with_capacity(statements.len());
    for statement in statements {
        let start_pos = statement.as_span().start_pos().pos();
        let end_pos = statement.as_span().end_pos().pos();
        let query = match statement.as_rule() {
            Rule::ping => DBQuery::Ping,
            Rule::list_clients => DBQuery::ListClients,
            Rule::list_stores => DBQuery::ListStores,
            Rule::info_server => DBQuery::InfoServer,
            Rule::create_store => {
                let mut inner_pairs = statement.into_inner().peekable();
                let mut error_if_exists = true;
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::if_not_exists {
                        inner_pairs.next(); // Consume rule
                        error_if_exists = false;
                    }
                };
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                let dimension = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .parse::<NonZeroUsize>()?;
                let mut create_predicates = HashSet::new();
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::metadata_keys {
                        let index_name_pairs = inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?; // Consume rule
                        create_predicates = index_name_pairs
                            .into_inner()
                            .map(|index_pair| MetadataKey::new(index_pair.as_str().to_string()))
                            .collect();
                    }
                };
                let mut non_linear_indices = HashSet::new();
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::non_linear_algorithms {
                        let index_name_pairs = inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?; // Consume rule
                        non_linear_indices = index_name_pairs
                            .into_inner()
                            .flat_map(|index_pair| to_non_linear(index_pair.as_str()))
                            .collect();
                    }
                };
                DBQuery::CreateStore {
                    store: StoreName(store.to_string()),
                    dimension,
                    create_predicates,
                    non_linear_indices,
                    error_if_exists,
                }
            }
            Rule::get_sim_n => {
                let mut inner_pairs = statement.into_inner();
                let closest_n = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .parse::<NonZeroUsize>()?;
                let f32_array = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let search_input = parse_f32_array(f32_array);
                let algorithm = to_algorithm(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .as_str(),
                )?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                let condition = if let Some(predicate_conditions) = inner_pairs.next() {
                    Some(parse_predicate_expression(predicate_conditions)?)
                } else {
                    None
                };
                DBQuery::GetSimN {
                    store: StoreName(store.to_string()),
                    search_input,
                    closest_n,
                    algorithm,
                    condition,
                }
            }
            Rule::get_pred => {
                let mut inner_pairs = statement.into_inner();
                let predicate_conditions = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                DBQuery::GetPred {
                    store: StoreName(store.to_string()),
                    condition: parse_predicate_expression(predicate_conditions)?,
                }
            }
            Rule::get_key => {
                let mut inner_pairs = statement.into_inner();
                let f32_arrays_pair = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let keys = parse_multi_f32_array(f32_arrays_pair);

                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                DBQuery::GetKey {
                    store: StoreName(store.to_string()),
                    keys,
                }
            }
            Rule::del_key => {
                let mut inner_pairs = statement.into_inner();
                let f32_arrays_pair = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let keys = parse_multi_f32_array(f32_arrays_pair);

                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                DBQuery::DelKey {
                    store: StoreName(store.to_string()),
                    keys,
                }
            }
            Rule::create_non_linear_algorithm_index => {
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
                DBQuery::CreateNonLinearAlgorithmIndex {
                    store: StoreName(store.to_string()),
                    non_linear_indices,
                }
            }
            Rule::create_pred_index => {
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
                DBQuery::CreatePredIndex {
                    store: StoreName(store.to_string()),
                    predicates,
                }
            }
            Rule::drop_non_linear_algorithm_index => {
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
                DBQuery::DropNonLinearAlgorithmIndex {
                    store: StoreName(store.to_string()),
                    non_linear_indices,
                    error_if_not_exists: !if_exists,
                }
            }
            Rule::drop_pred_index => {
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
                DBQuery::DropPredIndex {
                    store: StoreName(store.to_string()),
                    predicates,
                    error_if_not_exists: !if_exists,
                }
            }
            Rule::drop_store => {
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
                DBQuery::DropStore {
                    store: StoreName(store.to_string()),
                    error_if_not_exists: !if_exists,
                }
            }
            _ => return Err(DslError::UnexpectedSpan((start_pos, end_pos))),
        };
        queries.push(query);
    }
    Ok(queries)
}
