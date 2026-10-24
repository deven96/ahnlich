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
