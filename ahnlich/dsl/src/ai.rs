use std::{collections::HashSet, num::NonZeroUsize};

use crate::{
    algorithm::{to_algorithm, to_non_linear},
    metadata::{parse_store_input, parse_store_inputs, parse_store_inputs_to_store_value},
    parser::{QueryParser, Rule},
    shared::{
        parse_create_non_linear_algorithm_index, parse_create_pred_index,
        parse_drop_non_linear_algorithm_index, parse_drop_pred_index, parse_drop_store,
    },
};
use ahnlich_types::{
    ai::{AIModel, AIQuery, ImageAction, PreprocessAction, StringAction},
    keyval::StoreName,
    metadata::MetadataKey,
};
use pest::Parser;

use crate::{error::DslError, predicate::parse_predicate_expression};

fn parse_to_preprocess_action(input: &str) -> PreprocessAction {
    match input.to_lowercase().trim() {
        "erroriftokensexceed" => PreprocessAction::RawString(StringAction::ErrorIfTokensExceed),
        "truncateiftokensexceed" => {
            PreprocessAction::RawString(StringAction::TruncateIfTokensExceed)
        }
        "resizeimage" => PreprocessAction::Image(ImageAction::ResizeImage),
        "errorifdimensionsmismatch" => {
            PreprocessAction::Image(ImageAction::ErrorIfDimensionsMismatch)
        }
        _ => panic!("Unexpected preprocess action"),
    }
}

fn parse_to_ai_model(input: &str) -> Result<AIModel, DslError> {
    match input.to_lowercase().trim() {
        "all-minilm-l6-v2" => Ok(AIModel::AllMiniLML6V2),
        "all-minilm-l12-v2" => Ok(AIModel::AllMiniLML12V2),
        "bge-base-en-v1.5" => Ok(AIModel::BGEBaseEnV15),
        "bge-large-en-v1.5" => Ok(AIModel::BGELargeEnV15),
        "resnet-50" => Ok(AIModel::Resnet50),
        "clip-vit-b32" => Ok(AIModel::ClipVitB32),
        e => Err(DslError::UnsupportedAIModel(e.to_string())),
    }
}

// Parse raw strings separated by ; into a Vec<AIQuery>. Examples include but are not restricted
// to
pub const COMMANDS: &[&str] = &[
    "ping",
    "listclients",
    "liststores",
    "infoserver",
    "purgestores",
    "dropstore",                     // store_name if exists can be handled dynamically
    "createpredindex",               // (key_1, key_2) in store_name
    "droppredindex",                 // if exists (key1, key2) in store_name
    "createnonlinearalgorithmindex", // (kdtree) in store_name
    "dropnonlinearalgorithmindex",   // if exists (kdtree) in store_name
    "delkey",                        // ([input 1 text], [input 2 text]) in my_store
    "getpred",                       // ((author = dickens) or (country != Nigeria)) in my_store
    "getsimn", // 4 with [random text inserted here] using cosinesimilarity in my_store where (author = dickens)
    "createstore", // if not exists my_store querymodel resnet-50 indexmodel resnet-50 predicates (author, country) nonlinearalgorithmindex (kdtree)
    "set", // (([This is the life of Haks paragraphed], {name: Haks, category: dev}), ([This is the life of Deven paragraphed], {name: Deven, category: dev})) in store
];

pub fn parse_ai_query(input: &str) -> Result<Vec<AIQuery>, DslError> {
    let pairs = QueryParser::parse(Rule::ai_query, input).map_err(Box::new)?;
    let statements = pairs.into_iter().collect::<Vec<_>>();
    let mut queries = Vec::with_capacity(statements.len());
    for statement in statements {
        let start_pos = statement.as_span().start_pos().pos();
        let end_pos = statement.as_span().end_pos().pos();
        let query = match statement.as_rule() {
            Rule::ping => AIQuery::Ping,
            Rule::list_stores => AIQuery::ListStores,
            Rule::info_server => AIQuery::InfoServer,
            Rule::purge_stores => AIQuery::PurgeStores,
            Rule::ai_set_in_store => {
                let mut inner_pairs = statement.into_inner();
                let store_keys_to_store_values = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();

                let preprocess_action = parse_to_preprocess_action(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .as_str(),
                );

                AIQuery::Set {
                    store: StoreName(store.to_string()),
                    inputs: parse_store_inputs_to_store_value(store_keys_to_store_values)?,
                    preprocess_action,
                }
            }
            Rule::ai_create_store => {
                let mut inner_pairs = statement.into_inner().peekable();
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                let query_model = parse_to_ai_model(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .as_str(),
                )?;
                let index_model = parse_to_ai_model(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .as_str(),
                )?;
                let mut predicates = HashSet::new();
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::metadata_keys {
                        let index_name_pairs = inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?; // Consume rule
                        predicates = index_name_pairs
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
                AIQuery::CreateStore {
                    store: StoreName(store.to_string()),
                    query_model,
                    index_model,
                    predicates,
                    non_linear_indices,
                }
            }
            Rule::ai_get_sim_n => {
                let mut inner_pairs = statement.into_inner();
                let closest_n = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .parse::<NonZeroUsize>()?;
                let store_input = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let search_input = parse_store_input(store_input)?;
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
                AIQuery::GetSimN {
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
                AIQuery::GetPred {
                    store: StoreName(store.to_string()),
                    condition: parse_predicate_expression(predicate_conditions)?,
                }
            }
            Rule::ai_del_key => {
                let mut inner_pairs = statement.into_inner();
                let key = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let mut key = parse_store_inputs(key)?;

                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str();
                AIQuery::DelKey {
                    store: StoreName(store.to_string()),
                    // TODO: Fix inconsistencies with protocol delkey, this should take in a
                    // Vec<StoreInput> and not a single store input
                    key: key.remove(0),
                }
            }
            // TODO: Introduce AIQuery::GetKey & AIQuery::ListClients
            Rule::create_non_linear_algorithm_index => {
                let (store, non_linear_indices) =
                    parse_create_non_linear_algorithm_index(statement)?;
                AIQuery::CreateNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices,
                }
            }
            Rule::create_pred_index => {
                let (store, predicates) = parse_create_pred_index(statement)?;
                AIQuery::CreatePredIndex { store, predicates }
            }
            Rule::drop_non_linear_algorithm_index => {
                let (store, error_if_not_exists, non_linear_indices) =
                    parse_drop_non_linear_algorithm_index(statement)?;
                AIQuery::DropNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices,
                    error_if_not_exists,
                }
            }
            Rule::drop_pred_index => {
                let (store, predicates, error_if_not_exists) = parse_drop_pred_index(statement)?;
                AIQuery::DropPredIndex {
                    store,
                    predicates,
                    error_if_not_exists,
                }
            }
            Rule::drop_store => {
                let (store, error_if_not_exists) = parse_drop_store(statement)?;
                AIQuery::DropStore {
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
