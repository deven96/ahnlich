use std::collections::HashSet;
use std::mem::size_of_val;
use std::num::NonZeroUsize;

use crate::error::Error;
use crate::hnsw::Node;
use crate::{DistanceFn, NonLinearAlgorithmWithIndexImpl, hnsw::index::HNSW};
use crate::{EmbeddingKey, LinearAlgorithm};

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
        if matches!(accept_list.as_ref(), Some(a) if a.is_empty()) {
            return Ok(vec![]);
        }

        // When accept_list is provided, we search for more candidates to account for filtering
        let search_k = match accept_list {
            Some(ref list) => n.get().max(list.len()),
            None => n.get(),
        };

        let query = Node::new(EmbeddingKey::new(reference_point.to_vec()));
        let result_ids = self.knn_search(&query, search_k, None)?;

        let nodes_guard = self.nodes.pin();
        let mut results: Vec<(EmbeddingKey, f32)> = Vec::with_capacity(n.get());
        for node_id in result_ids {
            if results.len() >= n.get() {
                break;
            }
            if let Some(node) = nodes_guard.get(&node_id) {
                let key = node.value().clone();
                // Filter by accept_list if provided
                if let Some(ref accept) = accept_list {
                    if !accept.contains(&key) {
                        continue;
                    }
                }
                let distance = self
                    .distance_algorithm
                    .distance(reference_point, key.as_slice());
                results.push((key, distance));
            }
        }

        Ok(results)
    }

    fn size(&self) -> usize {
        let base = size_of_val(self);
        let nodes_guard = self.nodes.pin();
        let mut node_size: usize = 0;
        for (_, node) in nodes_guard.iter() {
            // size of the Node struct + the heap-allocated Vec<f32> data
            node_size += size_of_val(node) + (node.value().as_slice().len() * size_of_val(&0f32));
        }
        base + node_size
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
