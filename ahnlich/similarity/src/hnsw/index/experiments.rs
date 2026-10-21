//! Benchmark facades for the HNSW distance-reuse experiment.
//!
//! These retain the previous search path as a control against the production candidate.

use super::*;

mod compact_adjacency;
#[allow(unused_imports)] // Used by benchmark crates; lib tests use the child directly.
pub use compact_adjacency::CompactAdjacencyHNSW;

#[derive(Debug, PartialEq, Eq)]
pub struct HnswTopology {
    top_layer: u8,
    entry_points: Vec<u64>,
    graph: Vec<(u16, Vec<u64>)>,
    nodes: Vec<(u64, Vec<(u16, Vec<u64>)>, Vec<u64>)>,
}

impl<D: DistanceFn> HNSW<D> {
    /// Benchmark facade for the unchanged insertion implementation.
    pub fn insert_control_for_benchmark(&self, new: &[EmbeddingKey]) -> Result<(), Error> {
        for key in new {
            self.insert_node(Node::new(key.clone()))?;
        }
        Ok(())
    }

    /// Deep-copy an index outside the timed part of an incremental insertion benchmark.
    pub fn clone_for_benchmark(&self) -> Self {
        TempHNSW::from(self).into()
    }

    /// Benchmark facade for the candidate insertion implementation.
    pub fn insert_reuse_distances_candidate(&self, new: &[EmbeddingKey]) -> Result<(), Error> {
        for key in new {
            self.insert_node_with_reused_distances(Node::new(key.clone()))?;
        }
        Ok(())
    }

    /// Deterministic graph representation used to prove that the insertion variants build the
    /// same graph before their timings are compared.
    pub fn topology_snapshot(&self) -> HnswTopology {
        let mut entry_points: Vec<_> = self.enter_point.load().iter().map(|id| id.0).collect();
        entry_points.sort_unstable();

        let graph_guard = self.graph.pin();
        let mut graph: Vec<_> = graph_guard
            .iter()
            .map(|(layer, ids)| {
                let mut ids: Vec<_> = ids.pin().iter().map(|id| id.0).collect();
                ids.sort_unstable();
                (layer.0, ids)
            })
            .collect();
        graph.sort_unstable_by_key(|(layer, _)| *layer);

        let nodes_guard = self.nodes.pin();
        let mut nodes: Vec<_> = nodes_guard
            .iter()
            .map(|(id, node)| {
                let neighbours_guard = node.neighbours.pin();
                let mut neighbours: Vec<_> = neighbours_guard
                    .iter()
                    .map(|(layer, ids)| {
                        let mut ids: Vec<_> = ids.pin().iter().map(|id| id.0).collect();
                        ids.sort_unstable();
                        (layer.0, ids)
                    })
                    .collect();
                neighbours.sort_unstable_by_key(|(layer, _)| *layer);

                let mut back_links: Vec<_> = node.back_links.pin().iter().map(|id| id.0).collect();
                back_links.sort_unstable();
                (id.0, neighbours, back_links)
            })
            .collect();
        nodes.sort_unstable_by_key(|(id, _, _)| *id);

        HnswTopology {
            top_layer: self.top_most_layer.load(Ordering::Acquire),
            entry_points,
            graph,
            nodes,
        }
    }

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

impl<D: DistanceFn> HNSW<D> {
    /// Stored nodes and live per-node collections, excluding index-wide maps/sets.
    pub fn counter_shards_bench_counts(&self) -> (usize, usize) {
        let nodes = self.nodes.pin();
        let collections = nodes
            .values()
            .map(|node| 2 + node.neighbours.pin().len())
            .sum();
        (nodes.len(), collections)
    }

    /// Count per-layer outgoing lists and their edges for adjacency experiments.
    pub fn compact_adjacency_bench_counts(&self) -> (usize, usize) {
        let nodes = self.nodes.pin();
        let lists = nodes.values().map(|node| node.neighbours.pin().len()).sum();
        let edges = nodes
            .values()
            .map(|node| {
                let layers = node.neighbours.pin();
                layers
                    .values()
                    .map(|edges| edges.pin().len())
                    .sum::<usize>()
            })
            .sum();
        (lists, edges)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn vectors(start: usize, count: usize, dimensions: usize) -> Vec<EmbeddingKey> {
        (start..start + count)
            .map(|id| {
                EmbeddingKey::new(
                    (0..dimensions)
                        .map(|dimension| {
                            let mixed = id
                                .wrapping_mul(1_664_525)
                                .wrapping_add(dimension.wrapping_mul(1_013_904_223));
                            ((mixed % 20_003) as f32 / 10_001.5) - 1.0
                        })
                        .collect(),
                )
            })
            .collect()
    }

    #[test]
    fn insertion_distance_reuse_preserves_graph_topology() {
        let base = vectors(0, 128, 64);
        let additions = vectors(128, 32, 64);

        for shared_collector in [false, true] {
            for metric in [
                LinearAlgorithm::EuclideanDistance,
                LinearAlgorithm::CosineSimilarity,
                LinearAlgorithm::DotProductSimilarity,
            ] {
                let new_index = || {
                    if shared_collector {
                        HNSW::new_with_config_and_shared_collector_and_counter_shards(
                            HNSWConfig::default(),
                            metric,
                            NonZeroUsize::new(1).unwrap(),
                        )
                    } else {
                        HNSW::new_with_config(HNSWConfig::default(), metric)
                    }
                };
                let control = new_index();
                control.insert_control_for_benchmark(&base).unwrap();
                control.insert_control_for_benchmark(&additions).unwrap();

                let candidate = new_index();
                candidate.insert_reuse_distances_candidate(&base).unwrap();
                candidate
                    .insert_reuse_distances_candidate(&additions)
                    .unwrap();

                assert_eq!(candidate.topology_snapshot(), control.topology_snapshot());
            }
        }
    }

    #[test]
    fn compact_adjacency_candidate_preserves_topology_and_results() {
        let data = vectors(0, 256, 64);
        let config = HNSWConfig {
            ef_construction: 100,
            maximum_connections: 16,
            maximum_connections_zero: 32,
            extend_candidates: false,
            keep_pruned_connections: false,
        };

        for metric in [
            LinearAlgorithm::EuclideanDistance,
            LinearAlgorithm::CosineSimilarity,
            LinearAlgorithm::DotProductSimilarity,
        ] {
            let control = HNSW::new_with_config_and_shared_collector_and_counter_shards(
                config,
                metric,
                NonZeroUsize::new(1).unwrap(),
            );
            control.insert(&data).unwrap();

            let candidate = CompactAdjacencyHNSW::new(config, metric);
            candidate.insert(&data).unwrap();

            assert_eq!(candidate.topology_snapshot(), control.topology_snapshot());
            assert_eq!(
                candidate.edge_counts().1,
                candidate
                    .topology_snapshot()
                    .nodes
                    .iter()
                    .map(|(_, layers, _)| layers.iter().map(|(_, ids)| ids.len()).sum::<usize>())
                    .sum::<usize>()
            );

            for query in data.iter().step_by(31) {
                assert_eq!(
                    candidate
                        .n_nearest_with_ef(
                            query.as_slice(),
                            NonZeroUsize::new(10).unwrap(),
                            NonZeroUsize::new(100)
                        )
                        .unwrap(),
                    control
                        .n_nearest_with_ef(
                            query.as_slice(),
                            NonZeroUsize::new(10).unwrap(),
                            None,
                            NonZeroUsize::new(100)
                        )
                        .unwrap(),
                );
            }
        }
    }
}
