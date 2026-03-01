use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::{
    KdTreeConfig, NonLinearAlgorithm, NonLinearIndex, non_linear_index,
};

use crate::error::DslError;

pub(crate) fn to_non_linear(input: &str) -> Option<NonLinearAlgorithm> {
    match input.to_lowercase().trim() {
        "kdtree" => Some(NonLinearAlgorithm::KdTree),
        _ => None,
    }
}

// TODO: DSL currently only supports algorithm names (e.g. "kdtree") with default configs.
// We need to extend the grammar to allow users to pass NonLinearIndex configurations
// (e.g. HNSW { ef_construction: 100, max_connections: 16 }).
pub(crate) fn non_linear_to_index(algo: NonLinearAlgorithm) -> NonLinearIndex {
    match algo {
        NonLinearAlgorithm::KdTree => NonLinearIndex {
            index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
        },
        NonLinearAlgorithm::Hnsw => NonLinearIndex {
            index: Some(non_linear_index::Index::Hnsw(Default::default())),
        },
    }
}

pub(crate) fn to_algorithm(input: &str) -> Result<Algorithm, DslError> {
    match input.to_lowercase().trim() {
        "kdtree" => Ok(Algorithm::KdTree),
        "cosinesimilarity" => Ok(Algorithm::CosineSimilarity),
        "dotproductsimilarity" => Ok(Algorithm::DotProductSimilarity),
        "euclideandistance" => Ok(Algorithm::EuclideanDistance),
        e => Err(DslError::UnsupportedAlgorithm(e.to_string())),
    }
}
