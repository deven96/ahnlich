use super::super::errors::ServerError;
use super::FindSimilarN;
use crate::engine::store::StoreKeyId;
use ahnlich_similarity::EmbeddingKey;
use ahnlich_similarity::NonLinearAlgorithmWithIndexImpl;
use ahnlich_similarity::kdtree::KDTree;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use papaya::HashMap as ConcurrentHashMap;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::mem::size_of_val;
use std::num::NonZeroUsize;

// Maintain an enum even though we have a trait as we want to know size
#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum NonLinearAlgorithmWithIndex {
    KDTree(KDTree),
}

impl NonLinearAlgorithmWithIndex {
    fn get_inner(&self) -> &impl NonLinearAlgorithmWithIndexImpl<'_> {
        match self {
            Self::KDTree(kdtree) => kdtree,
        }
    }

    #[tracing::instrument]
    pub(crate) fn create(algorithm: NonLinearAlgorithm, dimension: NonZeroUsize) -> Self {
        match algorithm {
            NonLinearAlgorithm::KdTree => NonLinearAlgorithmWithIndex::KDTree(
                KDTree::new(dimension, dimension)
                    .expect("Impossible dimension happened during initalization of kdtree"),
            ),
        }
    }

    #[tracing::instrument(skip_all)]
    fn insert(&self, new: &[EmbeddingKey]) {
        if let Err(err) = self.get_inner().insert(new.to_vec()) {
            tracing::error!("Error inserting into index: {:?}", err)
        }
    }

    #[tracing::instrument(skip_all)]
    fn delete(&self, del: &[EmbeddingKey]) {
        if let Err(err) = self.get_inner().delete(del) {
            tracing::error!("Error deleting from index: {:?}", err)
        }
    }

    #[tracing::instrument(skip_all)]
    fn size(&self) -> usize {
        self.get_inner().size()
    }
}

impl FindSimilarN for NonLinearAlgorithmWithIndex {
    #[tracing::instrument(skip_all)]
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &EmbeddingKey,
        search_list: impl ParallelIterator<Item = &'a EmbeddingKey>,
        used_all: bool,
        n: NonZeroUsize,
    ) -> Vec<(StoreKeyId, f32)> {
        let accept_list = if used_all {
            None
        } else {
            Some(search_list.map(|key| key.clone()).collect())
        };
        self.get_inner()
            .n_nearest(search_vector.as_slice(), n, accept_list)
            .expect("Index does not have the same size as reference_point")
            .into_par_iter()
            .map(|(arr, sim)| (StoreKeyId::from(&arr), sim))
            .collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonLinearAlgorithmIndices {
    pub algorithm_to_index: ConcurrentHashMap<NonLinearAlgorithm, NonLinearAlgorithmWithIndex>,
}

impl NonLinearAlgorithmIndices {
    #[tracing::instrument]
    pub fn is_empty(&self) -> bool {
        let pinned = self.algorithm_to_index.pin();
        pinned.is_empty()
    }

    #[tracing::instrument]
    pub fn create(input: HashSet<NonLinearAlgorithm>, dimension: NonZeroUsize) -> Self {
        let algorithm_to_index = ConcurrentHashMap::new();
        for algo in input {
            let with_index = NonLinearAlgorithmWithIndex::create(algo, dimension);
            algorithm_to_index.insert(algo, with_index, &algorithm_to_index.guard());
        }
        Self { algorithm_to_index }
    }

    #[tracing::instrument(skip(self))]
    pub fn current_keys(&self) -> HashSet<NonLinearAlgorithm> {
        let pinned = self.algorithm_to_index.pin();
        pinned.keys().copied().collect()
    }

    #[tracing::instrument(skip(self, values))]
    pub fn insert_indices(
        &self,
        indices: HashSet<NonLinearAlgorithm>,
        values: &[EmbeddingKey],
        dimension: NonZeroUsize,
    ) {
        let pinned = self.algorithm_to_index.pin();
        for algo in indices {
            let with_index = NonLinearAlgorithmWithIndex::create(algo, dimension);
            with_index.insert(values);
            pinned.insert(algo, with_index);
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn remove_indices(
        &self,
        indices: HashSet<NonLinearAlgorithm>,
        error_if_not_exists: bool,
    ) -> Result<usize, ServerError> {
        let pinned = self.algorithm_to_index.pin();
        if let (true, Some(non_existing_index)) = (
            error_if_not_exists,
            indices.iter().find(|a| !pinned.contains_key(*a)),
        ) {
            return Err(ServerError::NonLinearIndexNotFound(*non_existing_index));
        }
        let mut deleted = 0;
        for index in indices {
            if pinned.remove_entry(&index).is_some() {
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    /// insert new entries into the non linear algorithm indices
    #[tracing::instrument(skip_all)]
    pub(crate) fn insert(&self, new: Vec<EmbeddingKey>) {
        let pinned = self.algorithm_to_index.pin();
        for (_, algo) in pinned.iter() {
            algo.insert(&new);
        }
    }

    /// delete old entries from the non linear algorithm indices
    #[tracing::instrument(skip_all)]
    pub(crate) fn delete(&self, old: &[EmbeddingKey]) {
        let pinned = self.algorithm_to_index.pin();
        for (_, algo) in pinned.iter() {
            algo.delete(old);
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn size(&self) -> usize {
        size_of_val(&self)
            + self
                .algorithm_to_index
                .iter(&self.algorithm_to_index.guard())
                .map(|(k, v)| size_of_val(k) + v.size())
                .sum::<usize>()
    }
}
