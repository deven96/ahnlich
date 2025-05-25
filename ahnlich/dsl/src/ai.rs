use std::num::NonZeroUsize;

use crate::{
    algorithm::{to_algorithm, to_non_linear},
    metadata::{parse_store_input, parse_store_inputs, parse_store_inputs_to_store_value},
    parser::{QueryParser, Rule},
    shared::{
        parse_create_non_linear_algorithm_index, parse_create_pred_index,
        parse_drop_non_linear_algorithm_index, parse_drop_pred_index, parse_drop_store,
    },
};
use ahnlich_types::ai::{
    execution_provider::ExecutionProvider,
    models::AiModel,
    pipeline::ai_query::Query as AiQuery,
    preprocess::PreprocessAction,
    query::{
        CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey,
        DropNonLinearAlgorithmIndex, DropPredIndex, DropStore, GetPred, GetSimN, InfoServer,
        ListStores, Ping, PurgeStores, Set,
    },
};
use pest::Parser;

use crate::{error::DslError, predicate::parse_predicate_expression};

fn parse_to_preprocess_action(input: &str) -> Result<PreprocessAction, DslError> {
    match input.to_lowercase().trim() {
        "nopreprocessing" => Ok(PreprocessAction::NoPreprocessing),
        "modelpreprocessing" => Ok(PreprocessAction::ModelPreprocessing),
        a => Err(DslError::UnsupportedPreprocessingMode(a.to_string())),
    }
}

fn parse_to_execution_provider(input: &str) -> Result<ExecutionProvider, DslError> {
    match input.to_lowercase().trim() {
        "cuda" => Ok(ExecutionProvider::Cuda),
        "coreml" => Ok(ExecutionProvider::CoreMl),
        "directml" => Ok(ExecutionProvider::DirectMl),
        "tensorrt" => Ok(ExecutionProvider::TensorRt),
        a => Err(DslError::UnsupportedPreprocessingMode(a.to_string())),
    }
}

fn parse_to_ai_model(input: &str) -> Result<AiModel, DslError> {
    match input.to_lowercase().trim() {
        "all-minilm-l6-v2" => Ok(AiModel::AllMiniLmL6V2),
        "all-minilm-l12-v2" => Ok(AiModel::AllMiniLmL12V2),
        "bge-base-en-v1.5" => Ok(AiModel::BgeBaseEnV15),
        "bge-large-en-v1.5" => Ok(AiModel::BgeLargeEnV15),
        "resnet-50" => Ok(AiModel::Resnet50),
        "clip-vit-b32-image" => Ok(AiModel::ClipVitB32Image),
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
    "getsimn", // 4 with [random text inserted here] using cosinesimilarity preprocessaction nopreprocessing in my_store where (author = dickens)
    "createstore", // if not exists my_store querymodel resnet-50 indexmodel resnet-50 predicates (author, country) nonlinearalgorithmindex (kdtree)
    "set", // (([This is the life of Haks paragraphed], {name: Haks, category: dev}), ([This is the life of Deven paragraphed], {name: Deven, category: dev})) in store
];

pub fn parse_ai_query(input: &str) -> Result<Vec<AiQuery>, DslError> {
    let pairs = QueryParser::parse(Rule::ai_query, input).map_err(Box::new)?;
    let statements = pairs.into_iter().collect::<Vec<_>>();
    let mut queries = Vec::with_capacity(statements.len());
    for statement in statements {
        let start_pos = statement.as_span().start_pos().pos();
        let end_pos = statement.as_span().end_pos().pos();
        let query = match statement.as_rule() {
            Rule::ping => AiQuery::Ping(Ping {}),
            Rule::list_stores => AiQuery::ListStores(ListStores {}),
            Rule::info_server => AiQuery::InfoServer(InfoServer {}),
            Rule::purge_stores => AiQuery::PurgeStores(PurgeStores {}),
            Rule::ai_set_in_store => {
                let mut inner_pairs = statement.into_inner();
                let store_keys_to_store_values = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();

                let preprocess_action = parse_to_preprocess_action(
                    inner_pairs
                        .next()
                        .map(|a| a.as_str())
                        .unwrap_or("nopreprocessing"),
                )? as i32;

                let mut execution_provider = None;
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::execution_provider_optional {
                        let mut pair = inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                            .into_inner();
                        execution_provider = Some(parse_to_execution_provider(
                            pair.next().map(|a| a.as_str()).unwrap(),
                        )? as i32);
                    }
                };

                AiQuery::Set(Set {
                    store,
                    inputs: parse_store_inputs_to_store_value(store_keys_to_store_values)?,
                    preprocess_action,
                    execution_provider,
                })
            }
            Rule::ai_create_store => {
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
                    .as_str()
                    .to_string();
                let query_model = parse_to_ai_model(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .as_str(),
                )? as i32;
                let index_model = parse_to_ai_model(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .as_str(),
                )? as i32;
                let mut predicates = Vec::new();
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::metadata_keys {
                        let index_name_pairs = inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?; // Consume rule
                        predicates = index_name_pairs
                            .into_inner()
                            .map(|index_pair| index_pair.as_str().to_string())
                            .collect();
                    }
                };
                let mut non_linear_indices = Vec::new();
                let mut store_original = false;
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::non_linear_algorithms {
                        let index_name_pairs = inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?; // Consume rule
                        non_linear_indices = index_name_pairs
                            .into_inner()
                            .flat_map(|index_pair| to_non_linear(index_pair.as_str()))
                            .map(|a| a as i32)
                            .collect();
                    }
                };
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::store_original {
                        store_original = true;
                    }
                }
                AiQuery::CreateStore(CreateStore {
                    store,
                    query_model,
                    index_model,
                    predicates,
                    non_linear_indices,
                    error_if_exists,
                    store_original,
                })
            }
            Rule::ai_get_sim_n => {
                let mut inner_pairs = statement.into_inner();
                let closest_n = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .parse::<NonZeroUsize>()?
                    .get() as u64;
                let store_input = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let search_input = parse_store_input(store_input)?;
                let algorithm = to_algorithm(
                    inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .as_str(),
                )? as i32;
                let mut preprocess_action = PreprocessAction::NoPreprocessing;
                let mut execution_provider = None;
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::preprocess_optional {
                        let mut pair = inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                            .into_inner();
                        preprocess_action = parse_to_preprocess_action(
                            pair.next().map(|a| a.as_str()).unwrap_or("nopreprocessing"),
                        )?;
                    }
                };
                if let Some(next_pair) = inner_pairs.peek() {
                    if next_pair.as_rule() == Rule::execution_provider_optional {
                        let mut pair = inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                            .into_inner();
                        execution_provider = Some(parse_to_execution_provider(
                            pair.next().map(|a| a.as_str()).unwrap(),
                        )? as i32);
                    }
                };
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                let condition = if let Some(predicate_conditions) = inner_pairs.next() {
                    Some(parse_predicate_expression(predicate_conditions)?)
                } else {
                    None
                };
                AiQuery::GetSimN(GetSimN {
                    store,
                    search_input: Some(search_input),
                    closest_n,
                    algorithm,
                    condition,
                    preprocess_action: preprocess_action as i32,
                    execution_provider,
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
                AiQuery::GetPred(GetPred {
                    store,
                    condition: Some(parse_predicate_expression(predicate_conditions)?),
                })
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
                    .as_str()
                    .to_string();
                AiQuery::DelKey(DelKey {
                    store,
                    // TODO: Fix inconsistencies with protocol delkey, this should take in a
                    // Vec<StoreInput> and not a single store input
                    key: Some(key.remove(0)),
                })
            }
            Rule::create_non_linear_algorithm_index => {
                let (store, non_linear_indices) =
                    parse_create_non_linear_algorithm_index(statement)?;
                AiQuery::CreateNonLinearAlgorithmIndex(CreateNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices: non_linear_indices.into_iter().map(|a| a as i32).collect(),
                })
            }
            Rule::create_pred_index => {
                let (store, predicates) = parse_create_pred_index(statement)?;
                AiQuery::CreatePredIndex(CreatePredIndex { store, predicates })
            }
            Rule::drop_non_linear_algorithm_index => {
                let (store, error_if_not_exists, non_linear_indices) =
                    parse_drop_non_linear_algorithm_index(statement)?;
                AiQuery::DropNonLinearAlgorithmIndex(DropNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices: non_linear_indices.into_iter().map(|a| a as i32).collect(),
                    error_if_not_exists,
                })
            }
            Rule::drop_pred_index => {
                let (store, predicates, error_if_not_exists) = parse_drop_pred_index(statement)?;
                AiQuery::DropPredIndex(DropPredIndex {
                    store,
                    predicates,
                    error_if_not_exists,
                })
            }
            Rule::drop_store => {
                let (store, error_if_not_exists) = parse_drop_store(statement)?;
                AiQuery::DropStore(DropStore {
                    store,
                    error_if_not_exists,
                })
            }
            _ => return Err(DslError::UnexpectedSpan((start_pos, end_pos))),
        };
        queries.push(query);
    }
    Ok(queries)
}
