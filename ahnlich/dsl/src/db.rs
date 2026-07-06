use std::num::NonZeroUsize;

use crate::{
    algorithm::{non_linear_to_index, to_algorithm, to_non_linear},
    array::{parse_f32_array, parse_multi_f32_array},
    metadata::parse_store_keys_to_store_value,
    parser::{QueryParser, Rule},
    shared::{
        parse_create_non_linear_algorithm_index, parse_create_pred_index,
        parse_drop_non_linear_algorithm_index, parse_drop_pred_index, parse_drop_store,
        parse_schema_clause,
    },
};
use ahnlich_types::db::{
    pipeline::db_query::Query as DBQuery,
    query::{
        CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey,
        DropNonLinearAlgorithmIndex, DropPredIndex, DropSchema, DropStore, GetKey, GetPred,
        GetSimN, GetStore, InfoServer, ListClients, ListStores, Ping, Set, Upsert,
    },
};
use pest::Parser;

use crate::{error::DslError, predicate::parse_predicate_expression};

// Parse raw strings separated by ; into a Vec<DBQuery>. Examples include but are not restricted
// to
pub const COMMANDS: &[&str] = &[
    "ping",
    "listclients",
    "liststores",
    "dropschema",
    "getstore",
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
    "upsert", // ([4.0, 5.0, 6.0], {id: 456}) where (id = 123) in store merge
];

pub fn parse_db_query(input: &str) -> Result<Vec<DBQuery>, DslError> {
    let pairs = QueryParser::parse(Rule::db_query, input).map_err(Box::new)?;
    let statements = pairs.into_iter().collect::<Vec<_>>();
    let mut queries = Vec::with_capacity(statements.len());
    for statement in statements {
        let start_pos = statement.as_span().start_pos().pos();
        let end_pos = statement.as_span().end_pos().pos();
        let query = match statement.as_rule() {
            Rule::ping => DBQuery::Ping(Ping {}),
            Rule::list_clients => DBQuery::ListClients(ListClients {}),
            Rule::list_stores => {
                let schema = statement
                    .into_inner()
                    .next()
                    .map(parse_schema_clause)
                    .transpose()?;
                DBQuery::ListStores(ListStores { schema })
            }
            Rule::info_server => DBQuery::InfoServer(InfoServer {}),
            Rule::get_store => {
                let mut inner_pairs = statement.into_inner();
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                let schema = inner_pairs.next().map(parse_schema_clause).transpose()?;
                DBQuery::GetStore(GetStore { store, schema })
            }
            Rule::set_in_store => {
                let mut inner_pairs = statement.into_inner();
                let store_keys_to_store_values = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();

                DBQuery::Set(Set {
                    store,
                    inputs: parse_store_keys_to_store_value(store_keys_to_store_values)?,
                    schema: inner_pairs.next().map(parse_schema_clause).transpose()?,
                })
            }
            Rule::create_store => {
                let mut inner_pairs = statement.into_inner().peekable();
                let mut error_if_exists = true;
                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::if_not_exists
                {
                    inner_pairs.next(); // Consume rule
                    error_if_exists = false;
                };
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                let dimension = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .parse::<NonZeroUsize>()?
                    .get() as u32;
                let mut create_predicates = Vec::new();
                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::metadata_keys
                {
                    let index_name_pairs = inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?; // Consume rule
                    create_predicates = index_name_pairs
                        .into_inner()
                        .map(|index_pair| index_pair.as_str().to_string())
                        .collect();
                };
                let mut non_linear_indices = Vec::new();
                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::non_linear_algorithms
                {
                    let index_name_pairs = inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?; // Consume rule
                    non_linear_indices = index_name_pairs
                        .into_inner()
                        .flat_map(|index_pair| to_non_linear(index_pair.as_str()))
                        .map(non_linear_to_index)
                        .collect();
                };
                let schema = if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::schema_clause
                {
                    Some(parse_schema_clause(
                        inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?,
                    )?)
                } else {
                    None
                };
                DBQuery::CreateStore(CreateStore {
                    store,
                    dimension,
                    create_predicates,
                    non_linear_indices,
                    error_if_exists,
                    schema,
                })
            }
            Rule::get_sim_n => {
                let mut inner_pairs = statement.into_inner();
                let closest_n = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .parse::<NonZeroUsize>()?
                    .get() as u64;
                let f32_array = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let search_input = parse_f32_array(f32_array);
                let algorithm = to_algorithm(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .as_str(),
                )? as i32;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                let mut schema = None;
                let mut condition = None;
                for pair in inner_pairs {
                    match pair.as_rule() {
                        Rule::schema_clause => schema = Some(parse_schema_clause(pair)?),
                        _ => condition = Some(parse_predicate_expression(pair)?),
                    }
                }
                DBQuery::GetSimN(GetSimN {
                    store,
                    search_input: Some(search_input),
                    closest_n,
                    algorithm,
                    condition,
                    schema,
                })
            }
            Rule::get_pred => {
                let mut inner_pairs = statement.into_inner();
                let predicate_conditions = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                DBQuery::GetPred(GetPred {
                    store,
                    condition: Some(parse_predicate_expression(predicate_conditions)?),
                    schema: inner_pairs.next().map(parse_schema_clause).transpose()?,
                })
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
                    .as_str()
                    .to_string();
                DBQuery::GetKey(GetKey {
                    store,
                    keys,
                    schema: inner_pairs.next().map(parse_schema_clause).transpose()?,
                })
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
                    .as_str()
                    .to_string();
                DBQuery::DelKey(DelKey {
                    store,
                    keys,
                    schema: inner_pairs.next().map(parse_schema_clause).transpose()?,
                })
            }
            Rule::create_non_linear_algorithm_index => {
                let (store, non_linear_indices, schema) =
                    parse_create_non_linear_algorithm_index(statement)?;
                DBQuery::CreateNonLinearAlgorithmIndex(CreateNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices: non_linear_indices
                        .into_iter()
                        .map(non_linear_to_index)
                        .collect(),
                    schema,
                })
            }
            Rule::create_pred_index => {
                let (store, predicates, schema) = parse_create_pred_index(statement)?;
                DBQuery::CreatePredIndex(CreatePredIndex {
                    store,
                    predicates,
                    schema,
                })
            }
            Rule::drop_non_linear_algorithm_index => {
                let (store, error_if_not_exists, non_linear_indices, schema) =
                    parse_drop_non_linear_algorithm_index(statement)?;
                DBQuery::DropNonLinearAlgorithmIndex(DropNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices: non_linear_indices.into_iter().map(|a| a as i32).collect(),
                    error_if_not_exists,
                    schema,
                })
            }
            Rule::drop_pred_index => {
                let (store, predicates, error_if_not_exists, schema) =
                    parse_drop_pred_index(statement)?;
                DBQuery::DropPredIndex(DropPredIndex {
                    store,
                    predicates,
                    error_if_not_exists,
                    schema,
                })
            }
            Rule::drop_store => {
                let (store, error_if_not_exists, schema) = parse_drop_store(statement)?;
                DBQuery::DropStore(DropStore {
                    store,
                    error_if_not_exists,
                    schema,
                })
            }
            Rule::drop_schema => {
                let mut inner_pairs = statement.into_inner();
                let schema = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                DBQuery::DropSchema(DropSchema { schema })
            }
            Rule::upsert => {
                let mut inner_pairs = statement.into_inner().peekable();
                let key_value_pair = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;

                // Parse new_key and new_value from upsert_key_value rule
                let mut key_value_inner = key_value_pair.into_inner();
                let (new_key, new_value) = parse_upsert_key_value(&mut key_value_inner)?;

                // Parse predicate condition
                let condition_pair = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let condition = Some(parse_predicate_expression(condition_pair)?);

                // Parse store name
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();

                // Check for optional MERGE clause
                let mut merge_metadata = false;

                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::merge_clause
                {
                    inner_pairs.next(); // Consume merge_clause
                    merge_metadata = true;
                }

                // Parse optional schema
                let schema = inner_pairs.next().map(parse_schema_clause).transpose()?;

                DBQuery::Upsert(Upsert {
                    store,
                    condition,
                    new_key,
                    new_value,
                    merge_metadata,
                    schema,
                })
            }
            _ => return Err(DslError::UnexpectedSpan((start_pos, end_pos))),
        };
        queries.push(query);
    }
    Ok(queries)
}

use crate::metadata::parse_metadata_value;
use ahnlich_types::keyval::{StoreKey, StoreValue};
use ahnlich_types::metadata::MetadataValue;
use std::collections::HashMap;

fn parse_upsert_key_value(
    pairs: &mut pest::iterators::Pairs<Rule>,
) -> Result<(Option<StoreKey>, Option<StoreValue>), DslError> {
    // First element is either f32_array or "none"
    let first = pairs.next().ok_or(DslError::UnexpectedSpan((0, 0)))?;
    let first_start = first.as_span().start_pos().pos();
    let first_end = first.as_span().end_pos().pos();

    let new_key = match first.as_rule() {
        Rule::f32_array => Some(parse_f32_array(first)),
        _ if first.as_str().to_lowercase() == "none" => None,
        _ => return Err(DslError::UnexpectedSpan((first_start, first_end))),
    };

    // Second element is either store_value or "none"
    let second = pairs.next().ok_or(DslError::UnexpectedSpan((0, 0)))?;
    let second_start = second.as_span().start_pos().pos();
    let second_end = second.as_span().end_pos().pos();

    let new_value = match second.as_rule() {
        Rule::store_value => {
            let mut store_value_map = HashMap::new();
            for store_value_single in second.into_inner() {
                let start_pos = store_value_single.as_span().start_pos().pos();
                let end_pos = store_value_single.as_span().end_pos().pos();
                let mut v = store_value_single.into_inner();
                let key = v
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                let value = parse_metadata_value(
                    v.next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?,
                )?;
                store_value_map.insert(key, MetadataValue { value: Some(value) });
            }
            Some(StoreValue {
                value: store_value_map,
            })
        }
        _ if second.as_str().to_lowercase() == "none" => None,
        _ => return Err(DslError::UnexpectedSpan((second_start, second_end))),
    };

    // Validate at least one is provided
    if new_key.is_none() && new_value.is_none() {
        return Err(DslError::UnexpectedSpan((first_start, second_end)));
    }

    Ok((new_key, new_value))
}
