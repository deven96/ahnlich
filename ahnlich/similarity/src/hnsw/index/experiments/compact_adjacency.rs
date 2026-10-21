//! Whole-graph candidate for compact, immutable outgoing adjacency lists.
//!
//! This is intentionally benchmark-only. It mirrors production's serial,
//! distance-reusing insertion path while leaving [`super::super::HNSW`] and its
//! persisted representation untouched. In particular, it does not claim to fix
//! the separate concurrent graph-insertion race.

use super::{super::*, HnswTopology};
use arc_swap::ArcSwap;
use std::{cmp::min, fmt, num::NonZeroUsize, sync::Arc};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// One layer's outgoing edges. Readers use an immutable snapshot. Writers copy
/// and publish with CAS so a pruning retry cannot overwrite an intervening add.
struct CompactEdges {
    ids: ArcSwap<Vec<NodeId>>,
}

impl CompactEdges {
    fn new() -> Self {
        Self {
            ids: ArcSwap::from_pointee(Vec::new()),
        }
    }

    fn from_ids(ids: Vec<NodeId>) -> Self {
        Self {
            ids: ArcSwap::from_pointee(ids),
        }
    }

    fn snapshot(&self) -> arc_swap::Guard<Arc<Vec<NodeId>>> {
        self.ids.load()
    }

    fn insert(&self, id: NodeId) {
        let mut current = self.ids.load();
        loop {
            if current.contains(&id) {
                return;
            }
            let mut next = Vec::with_capacity(current.len() + 1);
            next.extend_from_slice(&current);
            next.push(id);
            let previous = self.ids.compare_and_swap(&*current, Arc::new(next));
            if Arc::ptr_eq(&current, &previous) {
                return;
            }
            current = previous;
        }
    }

    /// Recompute selection against every CAS-retried snapshot. The selector must
    /// be pure: `ArcSwap` can call it more than once when another writer wins.
    fn prune_with(
        &self,
        maximum_connections: usize,
        mut select: impl FnMut(&[NodeId]) -> Result<Vec<NodeId>, Error>,
    ) -> Result<(), Error> {
        let mut current = self.ids.load();
        loop {
            if current.len() <= maximum_connections {
                return Ok(());
            }
            let selected = select(&current)?;
            let previous = self.ids.compare_and_swap(&*current, Arc::new(selected));
            if Arc::ptr_eq(&current, &previous) {
                return Ok(());
            }
            current = previous;
        }
    }
}

struct CompactNode {
    id: NodeId,
    value: EmbeddingKey,
    // Keep the production outer layer map; only its per-layer values change.
    neighbours: papaya::HashMap<LayerIndex, CompactEdges>,
    back_links: HashSet<NodeId>,
}

impl CompactNode {
    fn new(value: EmbeddingKey, collector: &Arc<Collector>, shards: NonZeroUsize) -> Self {
        Self {
            id: get_node_id(value.as_slice()),
            value,
            neighbours: node_hashmap(Some(collector), Some(shards)),
            back_links: node_hashset(Some(collector), Some(shards)),
        }
    }
}

/// Candidate HNSW used only by benchmark experiments.
///
/// It shares the production index-wide maps, shared collector, counter-shard
/// policy, construction algorithm, and search/pruning heuristic. `nodes` use a
/// compact per-layer edge value instead of Papaya's nested `HashSet`.
pub struct CompactAdjacencyHNSW<D: DistanceFn> {
    collector: Arc<Collector>,
    node_counter_shards: NonZeroUsize,
    pub ef_construction: usize,
    top_most_layer: AtomicU8,
    pub maximum_connections: usize,
    pub maximum_connections_zero: usize,
    graph: papaya::HashMap<LayerIndex, HashSet<NodeId>>,
    nodes: papaya::HashMap<NodeId, CompactNode>,
    enter_point: ArcSwap<SmallVec<[NodeId; 1]>>,
    distance_algorithm: D,
    keep_pruned_connections: bool,
    extend_candidates: bool,
}

impl<D: DistanceFn> fmt::Debug for CompactAdjacencyHNSW<D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CompactAdjacencyHNSW")
            .field("ef_construction", &self.ef_construction)
            .field("maximum_connections", &self.maximum_connections)
            .field("maximum_connections_zero", &self.maximum_connections_zero)
            .finish_non_exhaustive()
    }
}

impl<D: DistanceFn> CompactAdjacencyHNSW<D> {
    pub fn new(config: HNSWConfig, distance_algorithm: D) -> Self {
        let shards = default_node_counter_shards();
        assert!(config.maximum_connections > 1, "M must be > 1");
        let collector = Arc::new(Collector::new());
        Self {
            graph: new_hashmap_with_shared_collector(Some(&collector)),
            nodes: new_hashmap_with_shared_collector(Some(&collector)),
            collector,
            node_counter_shards: shards,
            ef_construction: config.ef_construction,
            top_most_layer: AtomicU8::new(0),
            maximum_connections: config.maximum_connections,
            maximum_connections_zero: config.maximum_connections_zero,
            enter_point: ArcSwap::new(SmallVec::new().into()),
            distance_algorithm,
            keep_pruned_connections: config.keep_pruned_connections,
            extend_candidates: config.extend_candidates,
        }
    }

    pub fn insert(&self, values: &[EmbeddingKey]) -> Result<(), Error> {
        for value in values {
            self.insert_node(CompactNode::new(
                value.clone(),
                &self.collector,
                self.node_counter_shards,
            ))?;
        }
        Ok(())
    }

    pub fn config(&self) -> HNSWConfig {
        HNSWConfig {
            ef_construction: self.ef_construction,
            maximum_connections: self.maximum_connections,
            maximum_connections_zero: self.maximum_connections_zero,
            extend_candidates: self.extend_candidates,
            keep_pruned_connections: self.keep_pruned_connections,
        }
    }

    pub fn distance_algorithm(&self) -> D {
        self.distance_algorithm
    }

    pub fn size(&self) -> usize {
        let nodes = self.nodes.pin();
        std::mem::size_of_val(self)
            + nodes
                .values()
                .map(|node| std::mem::size_of_val(node) + node.value.as_slice().len() * 4)
                .sum::<usize>()
    }

    fn insert_node(&self, value: CompactNode) -> Result<(), Error> {
        let nodes = self.nodes.pin();
        let graph = self.graph.pin();
        if nodes.contains_key(&value.id) {
            return Ok(());
        }

        let top_layer = self.top_most_layer.load(Ordering::Acquire);
        let new_level = value_level(value.id, self.maximum_connections);
        let mut entry_point = self.enter_point.load().as_ref().clone();

        for level in (new_level + 1..=top_layer).rev() {
            let nearest =
                self.search_layer_reuse(&value.value, &entry_point, 1, LayerIndex(level as u16))?;
            entry_point = smallvec![
                nearest
                    .into_iter()
                    .max()
                    .ok_or_else(|| {
                        Error::NotFoundError("nearest element not found".to_string())
                    })?
                    .id
            ];
        }

        for level in (0..=min(top_layer, new_level)).rev() {
            let layer = LayerIndex(level as u16);
            let nearest =
                self.search_layer_reuse(&value.value, &entry_point, self.ef_construction, layer)?;
            let selected =
                self.select_neighbours(&value.value, &nearest, self.maximum_connections, layer)?;

            let value_neighbours = value.neighbours.pin();
            let value_backlinks = value.back_links.pin();
            for neighbour_id in &selected {
                let neighbour = nodes
                    .get(neighbour_id)
                    .ok_or_else(|| Error::NotFoundError("Node Ref not found".to_string()))?;
                neighbour
                    .neighbours
                    .pin()
                    .get_or_insert_with(layer, CompactEdges::new)
                    .insert(value.id);
                value_neighbours
                    .get_or_insert_with(layer, CompactEdges::new)
                    .insert(*neighbour_id);
                value_backlinks.insert(*neighbour_id);
                neighbour.back_links.pin().insert(value.id);
            }

            graph
                .get_or_insert_with(layer, || {
                    hashset_from_iter_with_shared_collector([value.id], Some(&self.collector))
                })
                .pin()
                .insert(value.id);

            let maximum = if level == 0 {
                self.maximum_connections_zero
            } else {
                self.maximum_connections
            };
            for neighbour_id in &selected {
                let neighbour = nodes
                    .get(neighbour_id)
                    .ok_or_else(|| Error::NotFoundError("Node Ref not found".to_string()))?;
                let neighbour_layers = neighbour.neighbours.pin();
                let layer_edges = neighbour_layers
                    .get(&layer)
                    .ok_or_else(|| Error::NotFoundError("Index not found".to_string()))?;
                layer_edges.prune_with(maximum, |connections| {
                    let scored = connections
                        .iter()
                        .filter_map(|id| nodes.get(id))
                        .map(|node| {
                            OrderedNode::new(
                                node.id,
                                self.distance_algorithm
                                    .closeness(node.value.as_slice(), neighbour.value.as_slice()),
                            )
                        })
                        .collect::<Vec<_>>();
                    self.select_neighbours(&neighbour.value, &scored, maximum, layer)
                })?;
            }

            entry_point = match nearest.into_iter().max() {
                Some(nearest) => smallvec![nearest.id],
                None => entry_point,
            };
        }

        let value_id = value.id;
        nodes.insert(value_id, value);
        let current_top = self.top_most_layer.load(Ordering::Acquire);
        if new_level > current_top || nodes.len() == 1 {
            self.top_most_layer.store(new_level, Ordering::Release);
            self.enter_point.swap(smallvec![value_id].into());
        }
        Ok(())
    }

    fn search_layer_reuse(
        &self,
        query: &EmbeddingKey,
        entry_points: &[NodeId],
        ef: usize,
        layer: LayerIndex,
    ) -> Result<Vec<OrderedNode>, Error> {
        let nodes = self.nodes.pin();
        let mut visited = NodeIdHashSet::with_capacity_and_hasher(512, Default::default());
        visited.extend(entry_points.iter().copied());
        let scored_entries = entry_points
            .iter()
            .filter_map(|id| nodes.get(id))
            .map(|node| {
                OrderedNode::new(
                    node.id,
                    self.distance_algorithm
                        .closeness(node.value.as_slice(), query.as_slice()),
                )
            })
            .collect::<Vec<_>>();
        let mut candidates = NearestFirst::from_scored_with_embedding(
            scored_entries.iter().copied(),
            query.clone(),
            self.distance_algorithm,
        );
        let mut nearest = BoundedMaxHeap::new(NonZeroUsize::new(ef).unwrap_or(NonZeroUsize::MIN));
        for score in scored_entries {
            nearest.push(score);
        }

        while let Some(candidate) = candidates.pop() {
            if nearest.len() >= ef
                && let Some(furthest) = nearest.peek()
                && candidate.closeness < furthest.closeness
            {
                break;
            }
            let node = nodes
                .get(&candidate.id)
                .ok_or_else(|| Error::NotFoundError("Node not found".to_string()))?;
            let node_layers = node.neighbours.pin();
            let Some(edges) = node_layers.get(&layer) else {
                continue;
            };
            let edge_ids = edges.snapshot();
            for id in edge_ids.iter() {
                if !visited.insert(*id) {
                    continue;
                }
                let neighbour = nodes
                    .get(id)
                    .ok_or_else(|| Error::NotFoundError("Neighbor not found".to_string()))?;
                let scored = OrderedNode::new(
                    neighbour.id,
                    self.distance_algorithm
                        .closeness(neighbour.value.as_slice(), query.as_slice()),
                );
                let should_explore = nearest
                    .peek()
                    .is_none_or(|worst| scored.closeness > worst.closeness || nearest.len() < ef);
                if should_explore {
                    candidates.push_scored(scored);
                    nearest.push(scored);
                }
            }
        }
        Ok(nearest.iter().copied().collect())
    }

    fn select_neighbours(
        &self,
        query: &EmbeddingKey,
        candidates: &[OrderedNode],
        m: usize,
        layer: LayerIndex,
    ) -> Result<Vec<NodeId>, Error> {
        let nodes = self.nodes.pin();
        let mut response = NearestFirst::from_scored_with_embedding(
            std::iter::empty(),
            query.clone(),
            self.distance_algorithm,
        );
        let mut working = NearestFirst::from_scored_with_embedding(
            candidates.iter().copied(),
            query.clone(),
            self.distance_algorithm,
        );
        if self.extend_candidates {
            for candidate in candidates {
                let node = nodes
                    .get(&candidate.id)
                    .ok_or_else(|| Error::NotFoundError("Node Ref not found".to_string()))?;
                if let Some(edges) = node.neighbours.pin().get(&layer) {
                    for id in edges.snapshot().iter() {
                        if !working.contains(id)
                            && let Some(neighbour) = nodes.get(id)
                        {
                            working.push_scored(OrderedNode::new(
                                neighbour.id,
                                self.distance_algorithm
                                    .closeness(neighbour.value.as_slice(), query.as_slice()),
                            ));
                        }
                    }
                }
            }
        }

        let mut discarded = NearestFirst::from_scored_with_embedding(
            std::iter::empty(),
            query.clone(),
            self.distance_algorithm,
        );
        while !working.is_empty() && response.len() < m {
            let candidate = working.pop().ok_or(Error::QueueEmpty)?;
            if response.is_empty() {
                response.push_scored(candidate);
                continue;
            }
            let candidate_node = nodes
                .get(&candidate.id)
                .ok_or_else(|| Error::NotFoundError("Node Ref not found".to_string()))?;
            let diverse = response.iter().all(|selected| {
                let selected = nodes.get(&selected.id).expect("selected node disappeared");
                self.distance_algorithm
                    .closeness(candidate_node.value.as_slice(), selected.value.as_slice())
                    <= candidate.closeness
            });
            if diverse {
                response.push_scored(candidate);
            } else {
                discarded.push_scored(candidate);
            }
        }
        if self.keep_pruned_connections {
            while !discarded.is_empty() && response.len() < m {
                response.push_scored(discarded.pop().ok_or(Error::QueueEmpty)?);
            }
        }
        Ok(response.iter().map(|node| node.id).collect())
    }

    pub fn n_nearest_with_ef(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        ef_search: Option<NonZeroUsize>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        let query = EmbeddingKey::new(reference_point.to_vec());
        let ef = ef_search
            .map(NonZeroUsize::get)
            .unwrap_or_else(|| n.get().max(50))
            .max(n.get());
        let (mut entry, top) = {
            let entry = self.enter_point.load();
            (
                entry.as_ref().clone(),
                self.top_most_layer.load(Ordering::Acquire),
            )
        };
        for level in (1..=top).rev() {
            let nearest = self.search_layer_reuse(&query, &entry, 1, LayerIndex(level as u16))?;
            entry = smallvec![nearest.into_iter().max().ok_or(Error::QueueEmpty)?.id];
        }
        let final_scores = self.search_layer_reuse(&query, &entry, ef, LayerIndex(0))?;
        let mut queue = NearestFirst::from_scored_with_embedding(
            final_scores.into_iter(),
            query.clone(),
            self.distance_algorithm,
        );
        let nodes = self.nodes.pin();
        Ok(queue
            .pop_n(n)
            .into_iter()
            .filter_map(|score| {
                nodes.get(&score.id).map(|node| {
                    (
                        node.value.clone(),
                        self.distance_algorithm
                            .value(reference_point, node.value.as_slice()),
                    )
                })
            })
            .collect())
    }

    pub fn topology_snapshot(&self) -> HnswTopology {
        let mut entry_points = self
            .enter_point
            .load()
            .iter()
            .map(|id| id.0)
            .collect::<Vec<_>>();
        entry_points.sort_unstable();
        let mut graph = self
            .graph
            .pin()
            .iter()
            .map(|(layer, ids)| {
                let mut ids = ids.pin().iter().map(|id| id.0).collect::<Vec<_>>();
                ids.sort_unstable();
                (layer.0, ids)
            })
            .collect::<Vec<_>>();
        graph.sort_unstable_by_key(|(layer, _)| *layer);
        let mut nodes = self
            .nodes
            .pin()
            .iter()
            .map(|(id, node)| {
                let mut neighbours = node
                    .neighbours
                    .pin()
                    .iter()
                    .map(|(layer, edges)| {
                        let mut ids = edges.snapshot().iter().map(|id| id.0).collect::<Vec<_>>();
                        ids.sort_unstable();
                        (layer.0, ids)
                    })
                    .collect::<Vec<_>>();
                neighbours.sort_unstable_by_key(|(layer, _)| *layer);
                let mut backlinks = node
                    .back_links
                    .pin()
                    .iter()
                    .map(|id| id.0)
                    .collect::<Vec<_>>();
                backlinks.sort_unstable();
                (id.0, neighbours, backlinks)
            })
            .collect::<Vec<_>>();
        nodes.sort_unstable_by_key(|(id, _, _)| *id);
        HnswTopology {
            top_layer: self.top_most_layer.load(Ordering::Acquire),
            entry_points,
            graph,
            nodes,
        }
    }

    pub fn edge_counts(&self) -> (usize, usize) {
        let nodes = self.nodes.pin();
        let lists = nodes.values().map(|node| node.neighbours.pin().len()).sum();
        let edges = nodes
            .values()
            .map(|node| {
                let layers = node.neighbours.pin();
                layers
                    .values()
                    .map(|edges| edges.snapshot().len())
                    .sum::<usize>()
            })
            .sum();
        (lists, edges)
    }
}

impl<D: DistanceFn> Drop for CompactAdjacencyHNSW<D> {
    fn drop(&mut self) {
        self.nodes.pin().clear();
        self.graph.pin().clear();
        // SAFETY: candidate collectors are private and Drop has exclusive access.
        unsafe { self.collector.reclaim_all() };
    }
}

#[cfg(feature = "serde")]
impl<D: DistanceFn> Serialize for CompactAdjacencyHNSW<D> {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(serde::ser::Error::custom(
            "compact adjacency is a non-persistent benchmark experiment",
        ))
    }
}

#[cfg(feature = "serde")]
impl<'de, D: DistanceFn> Deserialize<'de> for CompactAdjacencyHNSW<D> {
    fn deserialize<De>(_deserializer: De) -> Result<Self, De::Error>
    where
        De: Deserializer<'de>,
    {
        Err(serde::de::Error::custom(
            "compact adjacency is a non-persistent benchmark experiment",
        ))
    }
}

fn value_level(node_id: NodeId, m: usize) -> u8 {
    let inv_log_m = 1.0 / (m as f64).ln();
    let uniform = (node_id.0 & ((1u64 << 53) - 1)) as f64 / (1u64 << 53) as f64;
    (-uniform.max(1e-10).ln() * inv_log_m).floor().min(255.0) as u8
}
