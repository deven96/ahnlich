//! Helpers for the HNSW benchmark experiments: the original hash-set edge storage as a
//! control, graph snapshots for equivalence checks, and allocation counters.

use super::*;

/// Original outgoing HashSet storage, available only to experiments and unit tests.
pub type ControlHNSW<D> = HNSW<D, HashSet<NodeId>>;

impl<D: DistanceFn> HNSW<D, HashSet<NodeId>> {
    pub fn new_control_with_config(config: HNSWConfig, distance_algorithm: D) -> Self {
        Self::build(config, distance_algorithm, None, None)
    }

    pub fn new_control_with_config_and_shared_collector(
        config: HNSWConfig,
        distance_algorithm: D,
    ) -> Self {
        Self::build(
            config,
            distance_algorithm,
            Some(Arc::new(Collector::new())),
            None,
        )
    }

    /// Control with explicit per-node counter shards, matching the candidate fixture.
    pub fn new_control_with_config_and_shared_collector_and_counter_shards(
        config: HNSWConfig,
        distance_algorithm: D,
        shards: NonZeroUsize,
    ) -> Self {
        Self::build(
            config,
            distance_algorithm,
            Some(Arc::new(Collector::new())),
            Some(shards),
        )
    }
}

/// A node's ID, its outgoing edges per layer, and its backlinks.
type NodeTopology = (u64, Vec<(u16, Vec<u64>)>, Vec<u64>);

#[derive(Debug, PartialEq, Eq)]
pub struct HnswTopology {
    top_layer: u8,
    entry_points: Vec<u64>,
    graph: Vec<(u16, Vec<u64>)>,
    nodes: Vec<NodeTopology>,
}

impl<D: DistanceFn, E: EdgeStorage> HNSW<D, E> {
    /// Deterministic graph representation used to prove that two indexes (for example the two
    /// edge storages) built the same graph before their timings are compared.
    pub fn topology_snapshot(&self) -> HnswTopology {
        let mut entry_points: Vec<_> = self.enter_point.load().ids.iter().map(|id| id.0).collect();
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
                        let mut ids: Vec<_> = ids.snapshot().iter().map(|id| id.0).collect();
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
            top_layer: self.top_layer(),
            entry_points,
            graph,
            nodes,
        }
    }

    /// Stored nodes and live logical collectors (retired collections are excluded).
    /// Used by the collector-sharing benchmark's correctness checks and report.
    pub fn collector_sharing_bench_counts(&self) -> (usize, usize) {
        let nodes = self.nodes.pin();
        let collections = 2
            + self.graph.pin().len()
            + nodes
                .values()
                .map(|node| 2 + usize::from(E::HASH_COLLECTION) * node.neighbours.pin().len())
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

    /// Stored nodes and live per-node collections, excluding index-wide maps/sets.
    pub fn counter_shards_bench_counts(&self) -> (usize, usize) {
        let nodes = self.nodes.pin();
        let collections = nodes
            .values()
            .map(|node| 2 + usize::from(E::HASH_COLLECTION) * node.neighbours.pin().len())
            .sum();
        (nodes.len(), collections)
    }

    pub fn compact_adjacency_bench_counts(&self) -> (usize, usize) {
        let nodes = self.nodes.pin();
        let lists = nodes.values().map(|node| node.neighbours.pin().len()).sum();
        let edges = nodes
            .values()
            .map(|node| {
                node.neighbours
                    .pin()
                    .values()
                    .map(|edges| edges.snapshot().len())
                    .sum::<usize>()
            })
            .sum();
        (lists, edges)
    }
}

#[cfg(feature = "bench-experiments")]
impl<E: EdgeStorage> HNSW<LinearAlgorithm, E> {
    /// Score cosine queries with cached norms while traversing this same graph.
    /// Index construction and selective-filter scans still use the control scorer.
    pub fn n_nearest_cached_cosine_with_ef(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        accept_list: Option<std::collections::HashSet<u64>>,
        ef_search: Option<NonZeroUsize>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        if self.distance_algorithm != LinearAlgorithm::CosineSimilarity {
            return self.n_nearest_with_ef(reference_point, n, accept_list, ef_search);
        }
        let score = |first: &Node<E>, second: &Node<E>| {
            crate::distance::cosine_similarity_with_norms(
                first.value.as_slice(),
                second.value.as_slice(),
                first.cosine_norm,
                second.cosine_norm,
            )
        };
        self.n_nearest_with_ef_by(
            reference_point,
            n,
            accept_list,
            ef_search,
            |node, query| score(node, query).into(),
            |query, node| score(query, node).value(),
        )
    }
}

#[cfg(all(test, feature = "bench-experiments"))]
mod cached_cosine_tests {
    use super::*;
    use crate::distance::{cosine_norm, cosine_similarity, cosine_similarity_with_norms};
    use rand::{Rng, SeedableRng, rngs::StdRng};

    #[test]
    fn cached_cosine_matches_fused_scores_with_and_without_simd_tails() {
        let mut rng = StdRng::seed_from_u64(16);
        for dimension in [1, 7, 8, 15, 16, 17, 31, 768, 769] {
            for _ in 0..32 {
                let a: Vec<f32> = (0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect();
                let b: Vec<f32> = (0..dimension).map(|_| rng.gen_range(-1.0..1.0)).collect();
                let control = cosine_similarity(&a, &b).value();
                let cached =
                    cosine_similarity_with_norms(&a, &b, cosine_norm(&a), cosine_norm(&b)).value();
                assert_eq!(cached.to_bits(), control.to_bits(), "dimension {dimension}");
            }
        }
        for a in [[0.0; 8], [f32::INFINITY; 8], [f32::NAN; 8], [f32::MAX; 8]] {
            let b = [1.0; 8];
            let control = cosine_similarity(&a, &b).value();
            let cached =
                cosine_similarity_with_norms(&a, &b, cosine_norm(&a), cosine_norm(&b)).value();
            assert_eq!(cached.to_bits(), control.to_bits());
        }
    }

    #[test]
    fn cached_cosine_returns_the_same_ids_and_scores_before_and_after_restore() {
        let mut rng = StdRng::seed_from_u64(42);
        let data: Vec<_> = (0..128)
            .map(|_| EmbeddingKey::new((0..17).map(|_| rng.gen_range(-1.0..1.0)).collect()))
            .collect();
        let index = HNSW::new_with_startup_counter_shards(
            HNSWConfig::default(),
            LinearAlgorithm::CosineSimilarity,
        );
        index.insert(&data).unwrap();
        let restored: HNSW<LinearAlgorithm> =
            serde_json::from_slice(&serde_json::to_vec(&index).unwrap()).unwrap();
        assert_eq!(index.topology_snapshot(), restored.topology_snapshot());
        let k = NonZeroUsize::new(10).unwrap();
        let ef = NonZeroUsize::new(40);
        for _ in 0..12 {
            let query: Vec<f32> = (0..17).map(|_| rng.gen_range(-1.0..1.0)).collect();
            let control = index.n_nearest_with_ef(&query, k, None, ef).unwrap();
            assert_eq!(control.len(), k.get());
            for graph in [&index, &restored] {
                let actual = graph
                    .n_nearest_cached_cosine_with_ef(&query, k, None, ef)
                    .unwrap();
                assert_eq!(control.len(), actual.len());
                for ((a, a_score), (b, b_score)) in control.iter().zip(&actual) {
                    assert_eq!(get_node_id(a.as_slice()), get_node_id(b.as_slice()));
                    assert_eq!(a_score.to_bits(), b_score.to_bits());
                }
            }
        }
    }
}
