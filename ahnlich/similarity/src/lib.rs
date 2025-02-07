use std::{collections::HashSet, num::NonZeroUsize};

use serde::{Deserialize, Serialize};
use utils::VecF32Ordered;

pub mod error;
pub mod hnsw;
pub mod kdtree;
pub mod utils;

pub trait NonLinearAlgorithmWithIndexImpl<'a>: Serialize + Deserialize<'a> {
    // insert a batch of new inputs
    fn insert(&self, new: Vec<Vec<f32>>) -> Result<(), error::Error>;
    // delete a batch of new inputs
    fn delete(&self, new: &[Vec<f32>]) -> Result<usize, error::Error>;
    // find the N-nearest points to the reference point, if accept_list is Some(_), only select
    // points from within the accept_list
    //
    // NOTE: that accept_list is triggered by sending predicate
    // conditions and can significantly slow down non-linear predicates as it might trigger an
    // almost linear search to find points within the accept list
    fn n_nearest(
        &self,
        reference_point: &Vec<f32>,
        n: NonZeroUsize,
        accept_list: Option<HashSet<VecF32Ordered>>,
    ) -> Result<Vec<(Vec<f32>, f32)>, error::Error>;
    // size of index structure
    fn size(&self) -> usize;
}
