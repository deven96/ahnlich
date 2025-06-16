use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;

use crate::error::DslError;

pub(crate) fn to_non_linear(input: &str) -> Option<NonLinearAlgorithm> {
    match input.to_lowercase().trim() {
        "kdtree" => Some(NonLinearAlgorithm::KdTree),
        _ => None,
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
