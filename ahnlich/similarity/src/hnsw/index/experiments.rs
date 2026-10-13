//! Benchmark facades for the HNSW distance-reuse experiment.
//!
//! These retain the previous search path as a control against the production candidate.

use super::*;

impl<D: DistanceFn> HNSW<D> {
    fn materialize_ids_with_scores(
        &self,
        reference_point: &[f32],
        ids: Vec<NodeId>,
    ) -> Vec<(EmbeddingKey, f32)> {
        let nodes = self.nodes.pin();
        ids.into_iter()
            .filter_map(|id| {
                nodes.get(&id).map(|node| {
                    let key = node.value.clone();
                    let value = self
                        .distance_algorithm
                        .value(reference_point, key.as_slice());
                    (key, value)
                })
            })
            .collect()
    }

    /// Benchmark facade for the current main implementation at the public-result boundary.
    pub fn n_nearest_control(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        ef: Option<usize>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        let query = Node::new(EmbeddingKey::new(reference_point.to_vec()));
        let ids = self.knn_search(&query, n.get(), ef, None)?;
        Ok(self.materialize_ids_with_scores(reference_point, ids))
    }

    /// Benchmark facade for the distance-reusing implementation at the public-result boundary.
    pub fn n_nearest_reuse_distances(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        ef: Option<usize>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        let query = Node::new(EmbeddingKey::new(reference_point.to_vec()));
        let ids = self.knn_search_reuse_distances(&query, n.get(), ef, None)?;
        Ok(self.materialize_ids_with_scores(reference_point, ids))
    }
}

impl<D: DistanceFn> HNSW<D> {
    /// Stored nodes and live logical collectors (retired collections are excluded).
    /// Used by the collector-sharing benchmark's correctness checks and report.
    pub fn collector_sharing_bench_counts(&self) -> (usize, usize) {
        let nodes = self.nodes.pin();
        let collections = 2
            + self.graph.pin().len()
            + nodes
                .values()
                .map(|node| 2 + node.neighbours.pin().len())
                .sum::<usize>();
        (
            nodes.len(),
            if self.collector.is_some() {
                1
            } else {
                collections
            },
        )
    }
}
