use std::{collections::HashSet, num::NonZeroUsize};

use crate::{
    algorithm::{to_algorithm, to_non_linear},
    array::{parse_f32_array, parse_multi_f32_array},
    metadata::parse_store_keys_to_store_value,
    parser::{QueryParser, Rule},
    shared::{
        parse_create_non_linear_algorithm_index, parse_create_pred_index,
        parse_drop_non_linear_algorithm_index, parse_drop_pred_index, parse_drop_store,
    },
};
use ahnlich_types::{db::DBQuery, keyval::StoreName, metadata::MetadataKey};
use pest::Parser;

use crate::{error::DslError, predicate::parse_predicate_expression};

// Parse raw strings separated by ; into a Vec<DBQuery>. Examples include but are not restricted
// to
pub const COMMANDS: &[&str] = &[
    "ping",
    "listclients",
    "liststores",
    "infoserver",
    "dropstore",                     // store_name if exists can be handled dynamically
    "createpredindex",               // (key_1, key_2) in store_name
    "droppredindex",                 // if exists (key1, key2) in store_name
    "createnonlinearalgorithmindex", // (kdtree) in store_name
    "dropnonlinearalgorithmindex",   // if exists (kdtree) in store_name
    "getkey",                        // ([1.0, 2.0], [3.0, 4.0]) in my_store
    "delkey",                        // ([1.2, 3.0], [5.6, 7.8]) in my_store
    "getpred",                       // ((author = dickens) or (country != Nigeria)) in my_store
    "getsimn", // 4 with [0.65, 2.78] using cosinesimilarity in my_store where (author = dickens)
    "createstore", // if not exists my_store dimension 21 predicates (author, country) nonlinearalgorithmindex (kdtree)
    "set", // (([1.0, 2.1, 3.2], {name: Haks, category: dev}), ([3.1, 4.8, 5.0], {name: Deven, category: dev})) in store
];

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
            Rule::set_in_store => {
                let mut inner_pairs = statement.into_inner();
                let store_keys_to_store_values = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();

                DBQuery::Set {
                    store: StoreName(store.to_string()),
                    inputs: parse_store_keys_to_store_value(store_keys_to_store_values)?,
                }
            }
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
                let (store, non_linear_indices) =
                    parse_create_non_linear_algorithm_index(statement)?;
                DBQuery::CreateNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices,
                }
            }
            Rule::create_pred_index => {
                let (store, predicates) = parse_create_pred_index(statement)?;
                DBQuery::CreatePredIndex { store, predicates }
            }
            Rule::drop_non_linear_algorithm_index => {
                let (store, error_if_not_exists, non_linear_indices) =
                    parse_drop_non_linear_algorithm_index(statement)?;
                DBQuery::DropNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices,
                    error_if_not_exists,
                }
            }
            Rule::drop_pred_index => {
                let (store, predicates, error_if_not_exists) = parse_drop_pred_index(statement)?;
                DBQuery::DropPredIndex {
                    store,
                    predicates,
                    error_if_not_exists,
                }
            }
            Rule::drop_store => {
                let (store, error_if_not_exists) = parse_drop_store(statement)?;
                DBQuery::DropStore {
                    store,
                    error_if_not_exists,
                }
            }
            _ => return Err(DslError::UnexpectedSpan((start_pos, end_pos))),
        };
        queries.push(query);
    }
    Ok(queries)
}
