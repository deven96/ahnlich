use std::collections::HashMap;
use std::collections::HashSet as StdHashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;

use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use ahnlich_types::algorithm::nonlinear::non_linear_index;
use ahnlich_types::db::query;
use ahnlich_types::db::server;
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreName, StoreValue};
use ahnlich_types::schema::Schema;
use ahnlich_types::shared::info::StoreUpsert;
use itertools::Itertools;
#[cfg(not(target_arch = "wasm32"))]
use rayon::prelude::*;

use crate::engine::store::StoreHandler;
use crate::errors::ServerError;

fn resolve_schema(schema: &Option<String>) -> Result<Schema, ServerError> {
    match schema {
        Some(s) => {
            Schema::try_new(s.clone()).map_err(|e| ServerError::InvalidArgument(e.to_owned()))
        }
        None => Ok(Schema::default()),
    }
}

pub fn create_store(
    store_handler: &StoreHandler,
    params: query::CreateStore,
) -> Result<(), ServerError> {
    let dimensions = NonZeroUsize::new(params.dimension as usize).ok_or_else(|| {
        ServerError::InvalidArgument("dimension must be greater than 0".to_owned())
    })?;

    let non_linear_indices: StdHashSet<non_linear_index::Index> = params
        .non_linear_indices
        .into_iter()
        .filter_map(|index| index.index)
        .collect();

    let schema = resolve_schema(&params.schema)?;

    store_handler.create_store(
        StoreName {
            value: params.store,
        },
        &schema,
        dimensions,
        params.create_predicates,
        non_linear_indices,
        params.error_if_exists,
    )
}

pub fn create_pred_index(
    store_handler: &StoreHandler,
    params: query::CreatePredIndex,
) -> Result<usize, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    store_handler.create_pred_index(
        &StoreName {
            value: params.store,
        },
        &schema,
        params.predicates,
    )
}

pub fn create_non_linear_algorithm_index(
    store_handler: &StoreHandler,
    params: query::CreateNonLinearAlgorithmIndex,
) -> Result<usize, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    let non_linear_indices: StdHashSet<non_linear_index::Index> = params
        .non_linear_indices
        .into_iter()
        .filter_map(|val| val.index)
        .collect();

    store_handler.create_non_linear_algorithm_index(
        &StoreName {
            value: params.store,
        },
        &schema,
        non_linear_indices,
    )
}

pub fn drop_pred_index(
    store_handler: &StoreHandler,
    params: query::DropPredIndex,
) -> Result<usize, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    store_handler.drop_pred_index_in_store(
        &StoreName {
            value: params.store,
        },
        &schema,
        params.predicates,
        params.error_if_not_exists,
    )
}

pub fn drop_non_linear_algorithm_index(
    store_handler: &StoreHandler,
    params: query::DropNonLinearAlgorithmIndex,
) -> Result<usize, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    let non_linear_indices: StdHashSet<NonLinearAlgorithm> = params
        .non_linear_indices
        .into_iter()
        .filter_map(|val| NonLinearAlgorithm::try_from(val).ok())
        .collect();

    store_handler.drop_non_linear_algorithm_index(
        &StoreName {
            value: params.store,
        },
        &schema,
        non_linear_indices,
        params.error_if_not_exists,
    )
}

pub fn del_key(store_handler: &StoreHandler, params: query::DelKey) -> Result<usize, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    let keys = params
        .keys
        .into_iter()
        .map(|key| StoreKey { key: key.key })
        .collect();

    store_handler.del_key_in_store(
        &StoreName {
            value: params.store,
        },
        &schema,
        keys,
    )
}

pub fn del_pred(
    store_handler: &StoreHandler,
    params: query::DelPred,
) -> Result<usize, ServerError> {
    let condition = params.condition.ok_or_else(|| {
        ServerError::InvalidArgument("Predicate Condition is required".to_owned())
    })?;
    let schema = resolve_schema(&params.schema)?;

    store_handler.del_pred_in_store(
        &StoreName {
            value: params.store,
        },
        &schema,
        &condition,
    )
}

pub fn drop_store(
    store_handler: &StoreHandler,
    params: query::DropStore,
) -> Result<usize, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    store_handler.drop_store(
        StoreName {
            value: params.store,
        },
        &schema,
        params.error_if_not_exists,
    )
}

pub fn set(store_handler: &StoreHandler, params: query::Set) -> Result<StoreUpsert, ServerError> {
    let schema = resolve_schema(&params.schema)?;

    #[cfg(not(target_arch = "wasm32"))]
    let inputs = params
        .inputs
        .into_par_iter()
        .filter_map(|entry| match (entry.key, entry.value) {
            (Some(key), Some(value)) => Some((key, value)),
            (Some(key), None) => Some((
                key,
                StoreValue {
                    value: HashMap::new(),
                },
            )),
            _ => None,
        })
        .collect();

    #[cfg(target_arch = "wasm32")]
    let inputs = params
        .inputs
        .into_iter()
        .filter_map(|entry| match (entry.key, entry.value) {
            (Some(key), Some(value)) => Some((key, value)),
            (Some(key), None) => Some((
                key,
                StoreValue {
                    value: HashMap::new(),
                },
            )),
            _ => None,
        })
        .collect();

    store_handler.set_in_store(
        &StoreName {
            value: params.store,
        },
        &schema,
        inputs,
    )
}

pub fn upsert(
    store_handler: &StoreHandler,
    params: query::Upsert,
) -> Result<StoreUpsert, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    let condition = params.condition.ok_or_else(|| {
        ServerError::InvalidArgument("Condition is required for UPSERT".to_owned())
    })?;

    store_handler.upsert(
        &StoreName {
            value: params.store,
        },
        &schema,
        params.new_key,
        params.new_value,
        &condition,
        params.merge_metadata,
    )
}

pub fn list_stores(store_handler: &StoreHandler, params: query::ListStores) -> server::StoreList {
    let schema = params
        .schema
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|s| Schema::new(s.clone()));
    let stores = store_handler
        .list_stores(schema.as_ref())
        .into_iter()
        .sorted()
        .collect();
    server::StoreList { stores }
}

pub fn drop_schema(
    store_handler: &StoreHandler,
    params: query::DropSchema,
) -> Result<u64, ServerError> {
    let schema =
        Schema::try_new(params.schema).map_err(|e| ServerError::InvalidArgument(e.to_owned()))?;
    let dropped = store_handler.drop_schema(&schema)?;
    Ok(dropped as u64)
}

pub fn get_key(
    store_handler: &StoreHandler,
    params: query::GetKey,
) -> Result<server::Get, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    let keys = params
        .keys
        .into_iter()
        .map(|key| StoreKey { key: key.key })
        .collect();

    let entries = store_handler
        .get_key_in_store(
            &StoreName {
                value: params.store,
            },
            &schema,
            keys,
        )?
        .into_iter()
        .map(|(embedding_key, store_value)| DbStoreEntry {
            key: Some(StoreKey {
                key: embedding_key.as_slice().to_vec(),
            }),
            value: Some(Arc::unwrap_or_clone(store_value)),
        })
        .collect();

    Ok(server::Get { entries })
}

pub fn get_pred(
    store_handler: &StoreHandler,
    params: query::GetPred,
) -> Result<server::Get, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    let condition = params
        .condition
        .ok_or_else(|| ServerError::InvalidArgument("Condition is required".to_owned()))?;

    let entries = store_handler
        .get_pred_in_store(
            &StoreName {
                value: params.store,
            },
            &schema,
            &condition,
        )?
        .into_iter()
        .map(|(embedding_key, store_value)| DbStoreEntry {
            key: Some(StoreKey {
                key: embedding_key.as_slice().to_vec(),
            }),
            value: Some(Arc::unwrap_or_clone(store_value)),
        })
        .collect();

    Ok(server::Get { entries })
}

pub fn get_sim_n(
    store_handler: &StoreHandler,
    params: query::GetSimN,
) -> Result<server::GetSimN, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    let search_input = params
        .search_input
        .ok_or_else(|| ServerError::InvalidArgument("Search input is required".to_owned()))?;
    let closest_n = NonZeroUsize::new(params.closest_n as usize)
        .ok_or_else(|| ServerError::InvalidArgument("closest_n must be > 0".to_owned()))?;
    let algorithm = Algorithm::try_from(params.algorithm)
        .map_err(|_| ServerError::InvalidArgument("Invalid algorithm".to_owned()))?;

    let results = store_handler.get_sim_in_store(
        &StoreName {
            value: params.store,
        },
        &schema,
        search_input,
        closest_n,
        algorithm,
        params.condition,
    )?;

    let entries = results
        .into_iter()
        .map(|(key, value, similarity)| server::GetSimNEntry {
            key: Some(StoreKey {
                key: key.as_slice().to_vec(),
            }),
            value: Some(Arc::unwrap_or_clone(value)),
            similarity: Some(similarity),
        })
        .collect();

    Ok(server::GetSimN { entries })
}

pub fn get_store(
    store_handler: &StoreHandler,
    params: query::GetStore,
) -> Result<server::StoreInfo, ServerError> {
    let schema = resolve_schema(&params.schema)?;
    store_handler.get_store(
        &StoreName {
            value: params.store,
        },
        &schema,
    )
}
