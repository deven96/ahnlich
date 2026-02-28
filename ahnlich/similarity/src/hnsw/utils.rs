use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::error::Error;
use crate::{DistanceFn, NonLinearAlgorithmWithIndexImpl, hnsw::index::HNSW};
use crate::{EmbeddingKey, LinearAlgorithm};

impl NonLinearAlgorithmWithIndexImpl for HNSW<LinearAlgorithm> {
    #[tracing::instrument(skip_all)]
    fn insert(&self, new: &[EmbeddingKey]) -> Result<(), Error> {
        todo!()
    }

    #[tracing::instrument(skip_all)]
    fn delete(&self, new: &[EmbeddingKey]) -> Result<usize, Error> {
        todo!()
    }

    #[tracing::instrument(skip_all)]
    fn n_nearest(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        accept_list: Option<HashSet<EmbeddingKey>>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        todo!()
    }
    // size of index structure
    fn size(&self) -> usize {
        todo!()
    }
}

impl<D: DistanceFn> serde::Serialize for HNSW<D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer + Sized,
    {
        todo!()
    }
}

impl<'de, Distance: DistanceFn> serde::Deserialize<'de> for HNSW<Distance> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}
