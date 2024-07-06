use super::FindSimilarN;
use ahnlich_similarity::kdtree::KDTree;
use ahnlich_similarity::utils::Array1F32Ordered;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::similarity::NonLinearAlgorithm;
use flurry::HashMap as ConcurrentHashMap;
use ndarray::Array1;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::num::NonZeroUsize;

#[derive(Debug, Serialize, Deserialize)]
pub(crate) enum NonLinearAlgorithmWithIndex {
    KDTree(KDTree),
}
impl NonLinearAlgorithmWithIndex {
    pub(crate) fn create(algorithm: NonLinearAlgorithm, dimension: NonZeroUsize) -> Self {
        match algorithm {
            NonLinearAlgorithm::KDTree => NonLinearAlgorithmWithIndex::KDTree(
                KDTree::new(dimension, dimension)
                    .expect("Impossible dimension happened during initalization of kdtree"),
            ),
        }
    }

    fn insert(&self, new: &[Array1<f32>]) {
        match self {
            NonLinearAlgorithmWithIndex::KDTree(kdtree) => {
                kdtree
                    .insert_multi(new.to_vec())
                    .expect("Impossible dimension happened during insert of kdtree");
            }
        }
    }

    fn delete(&self, new: &[Array1<f32>]) {
        match self {
            NonLinearAlgorithmWithIndex::KDTree(kdtree) => {
                kdtree
                    .delete_multi(new)
                    .expect("Impossible dimension happened during delete of kdtree");
            }
        }
    }
}

impl FindSimilarN for NonLinearAlgorithmWithIndex {
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &StoreKey,
        search_list: impl Iterator<Item = &'a StoreKey>,
        n: NonZeroUsize,
    ) -> Vec<(StoreKey, f32)> {
        let accept_list: HashSet<_> = search_list
            .cloned()
            .map(|key| Array1F32Ordered(key.0))
            .collect();
        match self {
            NonLinearAlgorithmWithIndex::KDTree(kdtree) => {
                kdtree
                    .n_nearest(&search_vector.0, n, Some(accept_list))
                    // we expect that algorithm shapes have already been confirmed before hand
                    .expect("KDTree does not have the same size as reference_point")
                    .into_iter()
                    .map(|(arr, sim)| (StoreKey(arr), sim))
                    .collect()
            }
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NonLinearAlgorithmIndices {
    pub algorithm_to_index: ConcurrentHashMap<NonLinearAlgorithm, NonLinearAlgorithmWithIndex>,
}

impl NonLinearAlgorithmIndices {
    pub fn create(input: HashSet<NonLinearAlgorithm>, dimension: NonZeroUsize) -> Self {
        let algorithm_to_index = ConcurrentHashMap::new();
        for algo in input {
            let with_index = NonLinearAlgorithmWithIndex::create(algo, dimension);
            algorithm_to_index.insert(algo, with_index, &algorithm_to_index.guard());
        }
        Self { algorithm_to_index }
    }

    /// insert new entries into the non linear algorithm indices
    pub(crate) fn insert(&self, new: Vec<Array1<f32>>) {
        let pinned = self.algorithm_to_index.pin();
        for (_, algo) in pinned.iter() {
            algo.insert(&new);
        }
    }

    /// delete old entries from the non linear algorithm indices
    pub(crate) fn delete(&self, old: &[Array1<f32>]) {
        let pinned = self.algorithm_to_index.pin();
        for (_, algo) in pinned.iter() {
            algo.delete(old);
        }
    }
}
