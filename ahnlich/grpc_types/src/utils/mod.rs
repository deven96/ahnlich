use std::collections::HashSet;

use crate::{algorithm::nonlinear::NonLinearAlgorithm, db::query::CreateStore};
use ahnlich_types::metadata::MetadataKey;
use ahnlich_types::query_builders::db as db_query_builders;
use ahnlich_types::similarity::NonLinearAlgorithm as InternalNonLinearAlgorithm;

impl From<NonLinearAlgorithm> for InternalNonLinearAlgorithm {
    fn from(value: NonLinearAlgorithm) -> Self {
        match value {
            NonLinearAlgorithm::KdTree => Self::KDTree,
        }
    }
}

//FIXME: Validation for Nonzerosize so it doesn't panic and also have this function return a GRPC
//error ``InvalidArgument``?

pub fn db_create_store(params: CreateStore) -> db_query_builders::CreateStoreParams {
    // convert

    db_query_builders::CreateStoreParams::builder()
        .store(params.store)
        .create_predicates(HashSet::from_iter(
            params.create_predicates.into_iter().map(MetadataKey::new),
        ))
        .dimension(params.dimension as usize)
        .non_linear_indices(HashSet::from_iter(
            params
                .non_linear_indices
                .into_iter()
                .filter_map(|index| NonLinearAlgorithm::try_from(index).ok())
                .map(|algo| algo.into()),
        ))
        .error_if_exists(params.error_if_exists)
        .build()
}
