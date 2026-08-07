use std::collections::HashMap;
use std::num::NonZeroUsize;

use crate::{
    algorithm::{non_linear_to_index, to_algorithm, to_non_linear},
    metadata::{parse_store_input, parse_store_inputs, parse_store_inputs_to_store_value},
    parser::{QueryParser, Rule},
    shared::{
        parse_create_non_linear_algorithm_index, parse_create_pred_index,
        parse_drop_non_linear_algorithm_index, parse_drop_pred_index, parse_drop_store,
        parse_schema_clause,
    },
};
use ahnlich_types::ai::{
    execution_provider::ExecutionProvider,
    models::AiModel,
    pipeline::ai_query::Query as AiQuery,
    preprocess::PreprocessAction,
    query::{
        CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey,
        DropNonLinearAlgorithmIndex, DropPredIndex, DropSchema, DropStore, GetKey, GetPred,
        GetSimN, GetStore, InfoServer, ListStores, Ping, PurgeStores, Set, Upsert,
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
        "rocm" => Ok(ExecutionProvider::Rocm),
        "migraphx" => Ok(ExecutionProvider::Migraphx),
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
        "clip-vit-b32-text" => Ok(AiModel::ClipVitB32Text),
        "clap-audio" => Ok(AiModel::ClapAudio),
        "clap-text" => Ok(AiModel::ClapText),
        "jina-embeddings-v2-base-code" => Ok(AiModel::JinaEmbeddingsV2BaseCode),
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
    "dropschema",
    "dropstore",                     // store_name if exists can be handled dynamically
    "createpredindex",               // (key_1, key_2) in store_name
    "droppredindex",                 // if exists (key1, key2) in store_name
    "createnonlinearalgorithmindex", // (kdtree) in store_name
    "dropnonlinearalgorithmindex",   // if exists (kdtree) in store_name
    "getstore",                      // store_name
    "getkey",                        // ([input 1 text], [input 2 text]) in my_store
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
            Rule::list_stores => {
                let schema = statement
                    .into_inner()
                    .next()
                    .map(parse_schema_clause)
                    .transpose()?;
                AiQuery::ListStores(ListStores { schema })
            }
            Rule::info_server => AiQuery::InfoServer(InfoServer {}),
            Rule::purge_stores => AiQuery::PurgeStores(PurgeStores {}),
            Rule::ai_set_in_store => {
                let mut inner_pairs = statement.into_inner().peekable();
                let store_keys_to_store_values = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();

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

                let preprocess_action = parse_to_preprocess_action(
                    inner_pairs
                        .next()
                        .map(|a| a.as_str())
                        .unwrap_or("nopreprocessing"),
                )? as i32;

                let mut execution_provider = None;
                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::execution_provider_optional
                {
                    let mut pair = inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .into_inner();
                    execution_provider = Some(parse_to_execution_provider(
                        pair.next().map(|a| a.as_str()).unwrap(),
                    )? as i32);
                };

                AiQuery::Set(Set {
                    store,
                    inputs: parse_store_inputs_to_store_value(store_keys_to_store_values)?,
                    preprocess_action,
                    execution_provider,
                    model_params: HashMap::new(),
                    schema,
                })
            }
            Rule::ai_create_store => {
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
                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::metadata_keys
                {
                    let index_name_pairs = inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?; // Consume rule
                    predicates = index_name_pairs
                        .into_inner()
                        .map(|index_pair| index_pair.as_str().to_string())
                        .collect();
                };
                let mut non_linear_indices = Vec::new();
                let mut store_original = false;
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
                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::store_original
                {
                    store_original = true;
                    inner_pairs.next();
                }
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
                AiQuery::CreateStore(CreateStore {
                    store,
                    query_model,
                    index_model,
                    predicates,
                    non_linear_indices,
                    error_if_exists,
                    store_original,
                    schema,
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
                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::preprocess_optional
                {
                    let mut pair = inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .into_inner();
                    preprocess_action = parse_to_preprocess_action(
                        pair.next().map(|a| a.as_str()).unwrap_or("nopreprocessing"),
                    )?;
                };
                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::execution_provider_optional
                {
                    let mut pair = inner_pairs
                        .next()
                        .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                        .into_inner();
                    execution_provider = Some(parse_to_execution_provider(
                        pair.next().map(|a| a.as_str()).unwrap(),
                    )? as i32);
                };
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
                AiQuery::GetSimN(GetSimN {
                    store,
                    search_input: Some(search_input),
                    closest_n,
                    algorithm,
                    condition,
                    preprocess_action: preprocess_action as i32,
                    execution_provider,
                    model_params: HashMap::new(),
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
                AiQuery::GetPred(GetPred {
                    store,
                    condition: Some(parse_predicate_expression(predicate_conditions)?),
                    schema: inner_pairs.next().map(parse_schema_clause).transpose()?,
                })
            }
            Rule::ai_get_key => {
                let mut inner_pairs = statement.into_inner();
                let key = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let keys = parse_store_inputs(key)?;

                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                AiQuery::GetKey(GetKey {
                    store,
                    keys,
                    schema: inner_pairs.next().map(parse_schema_clause).transpose()?,
                })
            }
            Rule::ai_del_key => {
                let mut inner_pairs = statement.into_inner();
                let key = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;
                let keys = parse_store_inputs(key)?;

                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                AiQuery::DelKey(DelKey {
                    store,
                    keys,
                    schema: inner_pairs.next().map(parse_schema_clause).transpose()?,
                })
            }
            Rule::create_non_linear_algorithm_index => {
                let (store, non_linear_indices, schema) =
                    parse_create_non_linear_algorithm_index(statement)?;
                AiQuery::CreateNonLinearAlgorithmIndex(CreateNonLinearAlgorithmIndex {
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
                AiQuery::CreatePredIndex(CreatePredIndex {
                    store,
                    predicates,
                    schema,
                })
            }
            Rule::drop_non_linear_algorithm_index => {
                let (store, error_if_not_exists, non_linear_indices, schema) =
                    parse_drop_non_linear_algorithm_index(statement)?;
                AiQuery::DropNonLinearAlgorithmIndex(DropNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices: non_linear_indices.into_iter().map(|a| a as i32).collect(),
                    error_if_not_exists,
                    schema,
                })
            }
            Rule::drop_pred_index => {
                let (store, predicates, error_if_not_exists, schema) =
                    parse_drop_pred_index(statement)?;
                AiQuery::DropPredIndex(DropPredIndex {
                    store,
                    predicates,
                    error_if_not_exists,
                    schema,
                })
            }
            Rule::drop_store => {
                let (store, error_if_not_exists, schema) = parse_drop_store(statement)?;
                AiQuery::DropStore(DropStore {
                    store,
                    error_if_not_exists,
                    schema,
                })
            }
            Rule::get_store => {
                let mut inner_pairs = statement.into_inner();
                let store = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                let schema = inner_pairs.next().map(parse_schema_clause).transpose()?;
                AiQuery::GetStore(GetStore { store, schema })
            }
            Rule::drop_schema => {
                let mut inner_pairs = statement.into_inner();
                let schema = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                    .as_str()
                    .to_string();
                AiQuery::DropSchema(DropSchema { schema })
            }
            Rule::ai_upsert => {
                let mut inner_pairs = statement.into_inner().peekable();
                let input_value_pair = inner_pairs
                    .next()
                    .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?;

                // Parse new_input and new_value from ai_upsert_input_value rule
                let mut input_value_inner = input_value_pair.into_inner();
                let (new_input, new_value) = parse_ai_upsert_input_value(&mut input_value_inner)?;

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

                // Parse preprocess_action
                let preprocess_action = parse_to_preprocess_action(
                    inner_pairs
                        .next()
                        .map(|a| a.as_str())
                        .unwrap_or("nopreprocessing"),
                )? as i32;

                // Parse optional execution_provider
                let mut execution_provider = None;
                if let Some(next_pair) = inner_pairs.peek()
                    && next_pair.as_rule() == Rule::execution_provider
                {
                    execution_provider = Some(parse_to_execution_provider(
                        inner_pairs
                            .next()
                            .ok_or(DslError::UnexpectedSpan((start_pos, end_pos)))?
                            .as_str(),
                    )? as i32);
                }

                // Parse optional schema
                let schema = inner_pairs.next().map(parse_schema_clause).transpose()?;

                AiQuery::Upsert(Upsert {
                    store,
                    condition,
                    new_input,
                    new_value,
                    preprocess_action,
                    execution_provider,
                    model_params: HashMap::new(), // Model params not supported in DSL yet
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
use ahnlich_types::keyval::{StoreInput, StoreValue};
use ahnlich_types::metadata::MetadataValue;

fn parse_ai_upsert_input_value(
    pairs: &mut pest::iterators::Pairs<Rule>,
) -> Result<(Option<StoreInput>, Option<StoreValue>), DslError> {
    // First element is either metadata_value (wrapped in []) or "none"
    let first = pairs.next().ok_or(DslError::UnexpectedSpan((0, 0)))?;
    let first_start = first.as_span().start_pos().pos();
    let first_end = first.as_span().end_pos().pos();

    let new_input = match first.as_rule() {
        Rule::metadata_value => Some(parse_store_input(first)?),
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
    if new_input.is_none() && new_value.is_none() {
        return Err(DslError::UnexpectedSpan((first_start, second_end)));
    }

    Ok((new_input, new_value))
}
