use std::{collections::HashSet, num::NonZeroUsize};

use serde::{Deserialize, Serialize};

pub mod distance;
pub mod embedding_key;
pub mod error;
pub mod heap;
pub mod hnsw;
pub mod kdtree;

#[cfg(test)]
pub mod tests;

pub use embedding_key::EmbeddingKey;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum LinearAlgorithm {
    EuclideanDistance,
    CosineSimilarity,
    DotProductSimilarity,
}

pub trait NonLinearAlgorithmWithIndexImpl<'a>: Serialize + Deserialize<'a> {
    // insert a batch of new inputs
    fn insert(&self, new: Vec<EmbeddingKey>) -> Result<(), error::Error>;
    // delete a batch of new inputs
    fn delete(&self, new: &[EmbeddingKey]) -> Result<usize, error::Error>;
    // find the N-nearest points to the reference point, if accept_list is Some(_), only select
    // points from within the accept_list
    //
    // NOTE: that accept_list is triggered by sending predicate
    // conditions and can significantly slow down non-linear predicates as it might trigger an
    // almost linear search to find points within the accept list
    fn n_nearest(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        accept_list: Option<HashSet<EmbeddingKey>>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, error::Error>;
    // size of index structure
    fn size(&self) -> usize;
}

pub trait DistanceFn: Send + Sync + Copy {
    fn distance(&self, a: &[f32], b: &[f32]) -> f32;
}
