use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::error::Error;
use crate::hnsw::index::HNSW;
use crate::{EmbeddingKey, LinearAlgorithm, NonLinearAlgorithmWithIndexImpl};

impl NonLinearAlgorithmWithIndexImpl for HNSW<LinearAlgorithm> {
    #[tracing::instrument(skip_all)]
    fn insert(&self, new: &[EmbeddingKey]) -> Result<(), Error> {
        self.insert(new)
    }

    #[tracing::instrument(skip_all)]
    fn delete(&self, items: &[EmbeddingKey]) -> Result<usize, Error> {
        self.delete(items)
    }

    #[tracing::instrument(skip_all)]
    fn n_nearest(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        accept_list: Option<HashSet<EmbeddingKey>>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        self.n_nearest(reference_point, n, accept_list)
    }

    fn size(&self) -> usize {
        self.size()
    }
}
