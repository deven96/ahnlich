use std::collections::HashMap;
use std::collections::HashSet as StdHashSet;
use std::num::NonZeroUsize;

use ahnlich_types::algorithm::nonlinear::non_linear_index;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use ahnlich_types::db::query;
use ahnlich_types::db::server;
use ahnlich_types::keyval::{StoreKey, StoreName, StoreValue};
use ahnlich_types::schema::Schema;
use ahnlich_types::shared::info::StoreUpsert;
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

use crate::engine::store::StoreHandler;
use crate::errors::ServerError;

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

    store_handler.create_store(
        StoreName {
            value: params.store,
        },
        Schema::default(),
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
    store_handler.create_pred_index(
        &StoreName {
            value: params.store,
        },
        &Schema::default(),
        params.predicates,
    )
}

pub fn create_non_linear_algorithm_index(
    store_handler: &StoreHandler,
    params: query::CreateNonLinearAlgorithmIndex,
) -> Result<usize, ServerError> {
    let non_linear_indices: StdHashSet<non_linear_index::Index> = params
        .non_linear_indices
        .into_iter()
        .filter_map(|val| val.index)
        .collect();

    store_handler.create_non_linear_algorithm_index(
        &StoreName {
            value: params.store,
        },
        &Schema::default(),
        non_linear_indices,
    )
}

pub fn drop_pred_index(
    store_handler: &StoreHandler,
    params: query::DropPredIndex,
) -> Result<usize, ServerError> {
    store_handler.drop_pred_index_in_store(
        &StoreName {
            value: params.store,
        },
        &Schema::default(),
        params.predicates,
        params.error_if_not_exists,
    )
}

pub fn drop_non_linear_algorithm_index(
    store_handler: &StoreHandler,
    params: query::DropNonLinearAlgorithmIndex,
) -> Result<usize, ServerError> {
    let non_linear_indices: StdHashSet<NonLinearAlgorithm> = params
        .non_linear_indices
        .into_iter()
        .filter_map(|val| NonLinearAlgorithm::try_from(val).ok())
        .collect();

    store_handler.drop_non_linear_algorithm_index(
        &StoreName {
            value: params.store,
        },
        &Schema::default(),
        non_linear_indices,
        params.error_if_not_exists,
    )
}

pub fn del_key(store_handler: &StoreHandler, params: query::DelKey) -> Result<usize, ServerError> {
    let keys = params
        .keys
        .into_iter()
        .map(|key| StoreKey { key: key.key })
        .collect();

    store_handler.del_key_in_store(
        &StoreName {
            value: params.store,
        },
        &Schema::default(),
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

    store_handler.del_pred_in_store(
        &StoreName {
            value: params.store,
        },
        &Schema::default(),
        &condition,
    )
}

pub fn drop_store(
    store_handler: &StoreHandler,
    params: query::DropStore,
) -> Result<usize, ServerError> {
    store_handler.drop_store(
        StoreName {
            value: params.store,
        },
        &Schema::default(),
        params.error_if_not_exists,
    )
}

pub fn set(store_handler: &StoreHandler, params: query::Set) -> Result<StoreUpsert, ServerError> {
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

    store_handler.set_in_store(
        &StoreName {
            value: params.store,
        },
        &Schema::default(),
        inputs,
    )
}

pub fn list_stores(store_handler: &StoreHandler) -> server::StoreList {
    let stores = store_handler.list_stores().into_iter().sorted().collect();
    server::StoreList { stores }
}
