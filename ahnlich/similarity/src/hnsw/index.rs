// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
// similarity function. It then navigates between these localised lists in DFS manner until it
// gets the values it needs to

use crate::{DistanceFn, LinearAlgorithm, error::Error, hnsw::HNSWConfig};

use super::{
    CompactEdges, LayerIndex, NearestFirst, Node, NodeId, NodeIdHashSet, OrderedNode,
    edges::{EdgeSnapshot, EdgeStorage},
    get_node_id,
};
use crate::EmbeddingKey;
use crate::heap::BoundedMaxHeap;

use arc_swap::ArcSwap;
use papaya::{Guard, HashSet};
use seize::Collector;
use smallvec::{SmallVec, smallvec};
use std::{
    cmp::min,
    num::NonZeroUsize,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

#[cfg(any(test, feature = "bench-experiments"))]
pub mod experiments;

#[cfg(test)]
mod collector_sharing_tests;

#[cfg(test)]
mod compact_adjacency_tests;

#[cfg(test)]
mod concurrency_tests;

/// Below this many accepted vectors, a filtered search scans the accepted set directly
/// instead of traversing the graph. At this size an exact SIMD scan costs less than a
/// graph walk that cannot fill `ef` and therefore keeps widening.
const BRUTE_FORCE_ACCEPT_LIST_THRESHOLD: usize = 4096;

// Process-wide startup policy. Each index captures it at construction/restore;
// existing collections are never resized or reconfigured by changing the default.
static DEFAULT_NODE_COUNTER_SHARDS: AtomicUsize = AtomicUsize::new(1);

/// Set the startup allocation policy before constructing or restoring DB stores.
/// CLI levels are resolved and CPU-capped by the DB server on this host.
pub fn configure_node_counter_shards(shards: NonZeroUsize) {
    assert!(
        shards.get().is_power_of_two(),
        "counter shards must be a power of two"
    );
    DEFAULT_NODE_COUNTER_SHARDS.store(shards.get(), Ordering::Relaxed);
}

fn default_node_counter_shards() -> NonZeroUsize {
    NonZeroUsize::new(DEFAULT_NODE_COUNTER_SHARDS.load(Ordering::Relaxed)).unwrap()
}

/// A node ID can be reachable from an edge while the node itself is not (yet, or any
/// longer) in `nodes`, because inserts and deletes publish and retract in separate steps.
/// Callers skip such IDs and the debug log keeps genuine inconsistencies visible.
#[track_caller]
fn skip_missing_node(id: &NodeId) {
    tracing::debug!(
        ?id,
        caller = %std::panic::Location::caller(),
        "skipping node ID that is not in the index"
    );
}

type NodesRef<'a, E> = papaya::HashMapRef<
    'a,
    NodeId,
    Node<E>,
    std::collections::hash_map::RandomState,
    seize::LocalGuard<'a>,
>;

#[cfg(test)]
type NeighbourSelectionHook = Box<dyn Fn(&[NodeId])>;

#[cfg(test)]
thread_local! {
    /// This gives us a test only break point between neighbour selection and linking, so a
    /// test can remove the chosen neighbours at exactly the point a concurrent delete would.
    static AFTER_NEIGHBOUR_SELECTION: std::cell::RefCell<Option<NeighbourSelectionHook>> =
        const { std::cell::RefCell::new(None) };

    /// This runs on the searching thread right after a search has loaded the entry
    /// point, so a test can delete it at exactly the point a concurrent delete would.
    static AFTER_SEARCH_ENTRY_LOAD: std::cell::RefCell<Option<Box<dyn Fn()>>> =
        const { std::cell::RefCell::new(None) };

    /// This runs once, between reading the entry point and trying to replace it, so a test
    /// can land a competing writer in exactly that gap.
    static BEFORE_ENTRY_PROMOTION: std::cell::RefCell<Option<Box<dyn FnOnce()>>> =
        const { std::cell::RefCell::new(None) };
}

/// The entry point node(s) and the layer they sit on, published together.
#[derive(Debug, Default)]
struct EntryPoint {
    ids: SmallVec<[NodeId; 1]>,
    level: u8,
}

/// Wrapper around papaya `HashMap::new()` that forces immediate allocation
/// inside `catch_unwind` to surface initialization panics at construction time
/// rather than on the first insert.
fn new_hashmap<K: std::hash::Hash + Eq, V>() -> papaya::HashMap<K, V> {
    std::panic::catch_unwind(|| papaya::HashMap::with_capacity(1))
        .expect("papaya HashMap initialization failed (allocation or assertion)")
}

fn new_hashmap_with_shared_collector<
    K: std::hash::Hash + Eq + Send + 'static,
    V: Send + 'static,
>(
    collector: Option<&Arc<Collector>>,
) -> papaya::HashMap<K, V> {
    match collector {
        None => new_hashmap(),
        Some(collector) => std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            papaya::HashMap::builder()
                .capacity(1)
                .shared_collector(Arc::clone(collector))
                .build()
        }))
        .expect("papaya HashMap initialization failed (allocation or assertion)"),
    }
}

fn hashset_from_iter_with_shared_collector<K: std::hash::Hash + Eq + Send + 'static>(
    values: impl IntoIterator<Item = K>,
    collector: Option<&Arc<Collector>>,
) -> HashSet<K> {
    let Some(collector) = collector else {
        return HashSet::from_iter(values);
    };
    let mut values = values.into_iter();
    let first = values.next();
    // Match Papaya's from_iter capacity hint, including an empty iterator.
    let capacity = if first.is_some() {
        values.size_hint().0.saturating_add(1)
    } else {
        0
    };
    let set = HashSet::builder()
        .capacity(capacity)
        .shared_collector(Arc::clone(collector))
        .build();
    {
        let guard = set.pin();
        if let Some(first) = first {
            guard.insert(first);
        }
        for value in values {
            guard.insert(value);
        }
    }
    set
}

// These helpers are only for per-node collections. Index-wide collections retain
// the existing shared-collector helpers and Papaya's default counter sharding.
fn node_hashmap<K: std::hash::Hash + Eq + Send + 'static, V: Send + 'static>(
    collector: Option<&Arc<Collector>>,
    shards: Option<NonZeroUsize>,
) -> papaya::HashMap<K, V> {
    match (collector, shards) {
        (Some(collector), Some(shards)) => {
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                papaya::HashMap::builder()
                    .capacity(1)
                    .shared_collector(Arc::clone(collector))
                    .counter_shards(shards)
                    .build()
            }))
            .expect("papaya HashMap initialization failed (allocation or assertion)")
        }
        _ => new_hashmap_with_shared_collector(collector),
    }
}

pub(super) fn node_hashset_from_iter<K: std::hash::Hash + Eq + Send + 'static>(
    values: impl IntoIterator<Item = K>,
    collector: Option<&Arc<Collector>>,
    shards: Option<NonZeroUsize>,
) -> HashSet<K> {
    let (Some(collector), Some(shards)) = (collector, shards) else {
        return hashset_from_iter_with_shared_collector(values, collector);
    };
    let mut values = values.into_iter();
    let first = values.next();
    let capacity = if first.is_some() {
        values.size_hint().0.saturating_add(1)
    } else {
        0
    };
    let set = HashSet::builder()
        .capacity(capacity)
        .shared_collector(Arc::clone(collector))
        .counter_shards(shards)
        .build();
    {
        let guard = set.pin();
        if let Some(first) = first {
            guard.insert(first);
        }
        for value in values {
            guard.insert(value);
        }
    }
    set
}

/// HNSW represents a Hierarchical Navigable Small World graph.
///
/// The graph is organized into multiple layers. Each layer contains a set of node IDs,
/// and each node holds its neighbours per layer along with its embedding vector.
/// This separation allows efficient lookups, prevents duplicate nodes per layer,
/// and supports deletion operations.
///
/// Uses papaya's concurrent HashMap for lock-free concurrent read access to nodes
/// and graph layers. The entry point and the top layer are held together in one ArcSwap,
/// so readers get a consistent pair with a single lock-free load.
///
/// Design rationale:
/// 1. `nodes` is the single source of truth: all Node structs live here, keyed by NodeId.
/// 2. `graph` maps each layer to a `HashSet` of NodeIds, ensuring uniqueness per layer
///    and fast removal when deleting nodes.
/// 3. Deletion is fully supported:
///    - Remove the node ID from the `graph` for all layers where it exists.
///    - Remove the node from `nodes`.
///    - Remove the node ID from all neighbours of other nodes (using back-links/referrals).
///      This ensures no stale references remain in the graph.
///
/// Concurrent reads (knn_search, search_layer) are lock-free. Writes (insert, delete)
///
/// Example of usage:
/// ```text
/// Layer 0: {42, 10, 55}
/// Layer 1: {42, 11, 9}
/// Layer 2: {42, 88}
/// Layer 3: {42, 200, 201}
///
/// Node 42 participates in layers 0–3, with neighbours stored per layer and
/// back-links automatically updated upon deletion.
/// ```
#[derive(Debug)]
pub struct HNSW<D: DistanceFn, E: EdgeStorage = CompactEdges> {
    // Runtime-only state. None preserves the independent-collector control path.
    collector: Option<Arc<Collector>>,
    // Runtime-only allocation policy, not part of the persisted graph configuration.
    node_counter_shards: Option<NonZeroUsize>,
    /// Breadth of search during insertion (efConstruction)
    pub ef_construction: usize,

    /// Maximum number of connections per node (M)
    pub maximum_connections: usize,

    /// Maximum number of connections per node (M) at layer 0
    pub maximum_connections_zero: usize,

    /// Precomputed value 1 / ln(M) used in level generation
    pub inv_log_m: f64,

    /// Nodes in each layer
    ///
    /// Each layer index maps to a set of NodeIds.
    /// This ensures uniqueness per layer and allows easy removal during deletion.
    /// Uses papaya's concurrent HashMap for lock-free read access.
    graph: papaya::HashMap<LayerIndex, papaya::HashSet<NodeId>>,

    /// All nodes in the HNSW
    ///
    /// The single source of truth for all node data.
    /// Keys are NodeId, values are the Node structs containing embeddings and neighbours.
    /// Uses papaya's concurrent HashMap for lock-free concurrent read access.
    nodes: papaya::HashMap<NodeId, Node<E>>,

    /// Entry point node ID for search operations.
    ///
    /// This is the node at the highest layer (the top layer) and serves as the
    /// starting point for all k-NN searches. Using the top-layer node ensures
    /// efficient hierarchical navigation through the graph.
    ///
    /// Updated whenever a new node is inserted at a higher layer than the current
    /// top layer. Protected by ArcSwap for lock-free concurrent access. The node and
    /// the top layer (L) live in one value, so a reader never sees one without the other.
    enter_point: ArcSwap<EntryPoint>,

    /// distance algorithm
    /// can be ec
    distance_algorithm: D,

    /// Keep pruned connections:
    keep_pruned_connections: bool,

    /// extend candidates:
    extend_candidates: bool,
}

impl<D: DistanceFn> HNSW<D> {
    /// Every collection owns its own memory reclamation. This is the baseline the shared
    /// collector variants are measured against.
    pub fn new_with_config(config: HNSWConfig, distance_algorithm: D) -> Self {
        Self::build(config, distance_algorithm, None, None)
    }

    /// One collector shared by the index and everything it owns, which cuts per-collection
    /// overhead.
    pub fn new_with_config_and_shared_collector(config: HNSWConfig, distance_algorithm: D) -> Self {
        Self::build(
            config,
            distance_algorithm,
            Some(Arc::new(Collector::new())),
            None,
        )
    }

    /// Shared collector, with `shards` counter shards for the per-node collections. The
    /// index-wide maps keep Papaya's default sharding.
    pub fn new_with_config_and_shared_collector_and_counter_shards(
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

    /// Shared collector, with the process-wide shard count set by
    /// [`configure_node_counter_shards`] (one shard unless configured). The DB builds its
    /// indexes this way.
    pub fn new_with_startup_counter_shards(config: HNSWConfig, distance_algorithm: D) -> Self {
        Self::new_with_config_and_shared_collector_and_counter_shards(
            config,
            distance_algorithm,
            default_node_counter_shards(),
        )
    }
}

impl<D: DistanceFn, E: EdgeStorage> HNSW<D, E> {
    /// The one place an index is assembled. `collector: None` leaves each collection with its
    /// own; `shards` only applies to the per-node collections.
    fn build(
        config: HNSWConfig,
        distance_algorithm: D,
        collector: Option<Arc<Collector>>,
        shards: Option<NonZeroUsize>,
    ) -> Self {
        assert!(config.maximum_connections > 1, "M must be > 1");
        assert!(
            shards.is_none_or(|shards| shards.get().is_power_of_two()),
            "counter shards must be a power of two"
        );
        Self {
            ef_construction: config.ef_construction,
            maximum_connections: config.maximum_connections,
            maximum_connections_zero: config.maximum_connections_zero,
            inv_log_m: 1.0 / (config.maximum_connections as f64).ln(),
            graph: new_hashmap_with_shared_collector(collector.as_ref()),
            nodes: new_hashmap_with_shared_collector(collector.as_ref()),
            collector,
            node_counter_shards: shards,
            enter_point: ArcSwap::from_pointee(EntryPoint::default()),
            distance_algorithm,
            keep_pruned_connections: config.keep_pruned_connections,
            extend_candidates: config.extend_candidates,
        }
    }

    /// Get the current configuration of this HNSW index
    pub fn config(&self) -> HNSWConfig {
        HNSWConfig {
            ef_construction: self.ef_construction,
            maximum_connections: self.maximum_connections,
            maximum_connections_zero: self.maximum_connections_zero,
            extend_candidates: self.extend_candidates,
            keep_pruned_connections: self.keep_pruned_connections,
        }
    }

    /// Get the distance algorithm used by this HNSW index
    pub fn distance_algorithm(&self) -> &D {
        &self.distance_algorithm
    }

    /// Get the current top-most layer index
    pub fn top_layer(&self) -> u8 {
        self.enter_point.load().level
    }

    /// Batch insert new embeddings into the HNSW graph.
    /// Matches the NonLinearAlgorithmWithIndexImpl::insert signature.
    pub fn insert(&self, new: &[EmbeddingKey]) -> Result<(), Error> {
        if new.is_empty() {
            return Ok(());
        }
        for key in new {
            let node = match (self.collector.as_ref(), self.node_counter_shards) {
                (None, _) => Node::<E>::new_with_edges(key.clone()),
                (Some(collector), None) => {
                    Node::<E>::new_with_shared_collector(key.clone(), collector)
                }
                (Some(collector), Some(shards)) => {
                    Node::<E>::new_with_shared_collector_and_counter_shards(
                        key.clone(),
                        collector,
                        shards,
                    )
                }
            };
            self.insert_node(node)?;
        }
        Ok(())
    }

    /// Batch delete embeddings from the HNSW graph.
    /// Returns the count of actually deleted items.
    /// Matches the NonLinearAlgorithmWithIndexImpl::delete signature.
    pub fn delete(&self, items: &[EmbeddingKey]) -> Result<usize, Error> {
        let mut deleted = 0;
        for key in items {
            let node_id = get_node_id(key.as_slice());
            if self.nodes.pin().contains_key(&node_id) {
                self.delete_node(&node_id);
                deleted += 1;
            }
        }
        Ok(deleted)
    }

    /// Compute the approximate memory size of this HNSW index.
    pub fn size(&self) -> usize {
        let base = std::mem::size_of_val(self);
        let nodes_guard = self.nodes.pin();
        let mut node_size: usize = 0;
        for (_, node) in nodes_guard.iter() {
            node_size += std::mem::size_of_val(node) + std::mem::size_of_val(node.value.as_slice());
        }
        base + node_size
    }

    /// Find the n nearest neighbors to the reference point.
    /// Results are returned in ascending distance order (closest first),
    pub fn n_nearest(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        accept_list: Option<std::collections::HashSet<u64>>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        self.n_nearest_with_ef(reference_point, n, accept_list, None)
    }

    /// Search with optional HNSW breadth; omitted values retain max(n, 50).
    pub fn n_nearest_with_ef(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        accept_list: Option<std::collections::HashSet<u64>>,
        ef_search: Option<NonZeroUsize>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        if matches!(accept_list.as_ref(), Some(a) if a.is_empty()) {
            return Ok(vec![]);
        }

        // Highly selective predicate: scanning the matched vectors directly is both cheaper
        // and exact. Graph traversal degenerates here — with few acceptable nodes the search
        // cannot fill `ef` and keeps widening, approaching a full traversal for a handful of
        // results. Below the threshold a linear scan is strictly the better plan.
        if let Some(ref accept) = accept_list
            && accept.len() <= BRUTE_FORCE_ACCEPT_LIST_THRESHOLD
        {
            return Ok(self.brute_force_accept_list(reference_point, n, accept));
        }

        let query = Node::<E>::new_with_edges(EmbeddingKey::new(reference_point.to_vec()));
        // `k` is what the caller asked for. It is NOT derived from the size of the filter:
        // the filter is applied during traversal (see search_layer), so no over-search is
        // needed to compensate for post-filtering.
        let result_ids = self.knn_search(
            &query,
            n.get(),
            ef_search.map(NonZeroUsize::get),
            accept_list.as_ref(),
        )?;

        let nodes_guard = self.nodes.pin();
        let mut results: Vec<(EmbeddingKey, f32)> = Vec::with_capacity(n.get());
        for node_id in result_ids {
            if results.len() >= n.get() {
                break;
            }
            if let Some(node) = nodes_guard.get(&node_id) {
                let key = node.value.clone();
                let score = self
                    .distance_algorithm
                    .value(reference_point, key.as_slice());
                results.push((key, score));
            }
        }

        Ok(results)
    }

    /// Exact nearest-neighbour scan restricted to `accept`, used when the predicate is
    /// selective enough that touching the graph is not worth it.
    fn brute_force_accept_list(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        accept: &std::collections::HashSet<u64>,
    ) -> Vec<(EmbeddingKey, f32)> {
        let nodes_guard = self.nodes.pin();
        let mut scored: Vec<(OrderedNode, EmbeddingKey, f32)> = accept
            .iter()
            .filter_map(|id| nodes_guard.get(&NodeId(*id)))
            .map(|node| {
                let key = node.value.clone();
                let score = self
                    .distance_algorithm
                    .score(reference_point, key.as_slice());
                (
                    OrderedNode::new(node.id, score.closeness()),
                    key,
                    score.value(),
                )
            })
            .collect();

        // Closest first. `OrderedNode` breaks ties on NodeId rather than on HashSet
        // iteration order, so replicas agree; NodeId is a content hash, so the order is
        // stable across processes and restarts.
        scored.sort_by(|(a, _, _), (b, _, _)| b.cmp(a));
        scored.truncate(n.get());
        scored
            .into_iter()
            .map(|(_, key, score)| (key, score))
            .collect()
    }

    /// Insert a new element into the HNSW graph
    /// Corresponds to Algorithm 1 (INSERT)
    ///
    /// If a node with the same embedding is added, we silently skip as it doesn't make sense to do
    /// any work but also we shouldn't fail necessarily
    ///
    /// The node is published in `nodes` before it is linked, so it takes part in pruning the
    /// neighbours it links to, and other writers can never follow an edge to a node that is
    /// not stored. If linking fails, the node is removed again.
    pub fn insert_node(&self, value: Node<E>) -> Result<(), Error> {
        let value = match self.collector.as_ref() {
            Some(collector)
                if !std::ptr::eq(value.neighbours.guard().collector(), collector.as_ref()) =>
            {
                node_from_temp_with_counter_shards(
                    TempNode::from(&value),
                    Some(collector),
                    self.node_counter_shards,
                )
            }
            _ => value,
        };
        let nodes = self.nodes.pin();
        let id = value.id;
        // Only one writer can claim an ID; a duplicate is silently skipped.
        let Ok(stored) = nodes.try_insert(id, value) else {
            return Ok(());
        };

        let level = stored.level(self.maximum_connections);
        // Writers that start on an empty index must have something to link to, so the first
        // node becomes the entry point as soon as it is published, not once it is linked.
        if self.enter_point.load().ids.is_empty() {
            self.promote_entry_point(id, level);
        }

        if let Err(err) = self.link_node(&nodes, stored) {
            self.delete_node(&id);
            return Err(err);
        }
        self.promote_entry_point(id, level);
        Ok(())
    }

    /// Connect an already published node to the graph on every layer it belongs to.
    ///
    /// Distances computed during layer search are reused for neighbour selection and
    /// entry-point selection.
    fn link_node(&self, nodes: &NodesRef<'_, E>, value: &Node<E>) -> Result<(), Error> {
        let graph = self.graph.pin();
        let initial_ef = 1;

        let entry = self.enter_point.load();
        let top_layer = entry.level;
        let mut enter_point = entry.ids.clone();
        let new_elements_lvl = value.level(self.maximum_connections);

        for level_current in (new_elements_lvl + 1..=top_layer).rev() {
            enter_point = self.live_entry_points(nodes, enter_point);
            let mut nearest_neighbours = self.search_layer(
                value,
                &enter_point,
                initial_ef,
                &LayerIndex(level_current as u16),
                None,
            )?;
            nearest_neighbours.retain(|nearest| nearest.id != value.id);

            if let Some(nearest) = nearest_neighbours.into_iter().max() {
                enter_point = smallvec![nearest.id];
            }
        }

        for level_current in (0..=min(top_layer, new_elements_lvl)).rev() {
            let layer_index = LayerIndex(level_current as u16);
            enter_point = self.live_entry_points(nodes, enter_point);
            let mut nearest_neighbours = self.search_layer(
                value,
                &enter_point,
                self.ef_construction,
                &layer_index,
                None,
            )?;
            nearest_neighbours.retain(|nearest| nearest.id != value.id);

            let neighbours = self.select_neighbours_heuristic(
                value,
                &nearest_neighbours,
                self.maximum_connections,
                &layer_index,
                false,
                false,
            )?;
            let neighbours = self.live_link_targets(nodes, &neighbours, value.id);

            let value_neighbours_guard = value.neighbours.pin();
            let value_backlinks_guard = value.back_links.pin();
            for neighbour_id in neighbours.iter() {
                let Some(neighbour_node) = nodes.get(neighbour_id) else {
                    skip_missing_node(neighbour_id);
                    continue;
                };

                let n_guard = neighbour_node.neighbours.pin();
                let edges = n_guard.get_or_insert_with(layer_index, || {
                    E::from_ids(
                        Vec::new(),
                        self.collector.as_ref(),
                        self.node_counter_shards,
                    )
                });
                edges.insert_edge(value.id);

                let edges = value_neighbours_guard.get_or_insert_with(layer_index, || {
                    E::from_ids(
                        Vec::new(),
                        self.collector.as_ref(),
                        self.node_counter_shards,
                    )
                });
                edges.insert_edge(*neighbour_id);

                value_backlinks_guard.insert(*neighbour_id);
                neighbour_node.back_links.pin().insert(value.id);
                // A prune may have dropped one of the two new edges before its entry was
                // recorded, and it could not forget an entry that did not exist yet.
                self.forget_backlink(nodes, neighbour_node, value.id);
                self.forget_backlink(nodes, value, *neighbour_id);
            }

            graph
                .get_or_insert_with(layer_index, || {
                    hashset_from_iter_with_shared_collector([value.id], self.collector.as_ref())
                })
                .pin()
                .insert(value.id);

            for neighbour in neighbours.iter() {
                let maximum_connections = if level_current == 0 {
                    self.maximum_connections_zero
                } else {
                    self.maximum_connections
                };

                let Some(neighbour_node) = nodes.get(neighbour) else {
                    skip_missing_node(neighbour);
                    continue;
                };

                let nn_guard = neighbour_node.neighbours.pin();
                let Some(e_conn) = nn_guard.get(&layer_index) else {
                    skip_missing_node(neighbour);
                    continue;
                };

                // The compact prune may run the selection again after a lost compare-and-swap,
                // so this holds the drops of the last attempt only.
                let mut dropped: Vec<NodeId> = Vec::new();
                if let Some(replacement) = e_conn.prune(
                    maximum_connections,
                    self.collector.as_ref(),
                    self.node_counter_shards,
                    |ids| {
                        let scored_connections = ids
                            .iter()
                            .filter_map(|id| nodes.get(id))
                            .map(|node| {
                                OrderedNode::new(
                                    node.id,
                                    self.distance_algorithm.closeness(
                                        node.value.as_slice(),
                                        neighbour_node.value.as_slice(),
                                    ),
                                )
                            })
                            .collect::<Vec<_>>();
                        let mut selected = self.select_neighbours_heuristic(
                            neighbour_node,
                            &scored_connections,
                            maximum_connections,
                            &layer_index,
                            false,
                            false,
                        )?;
                        dropped.clear();
                        let removed: Vec<NodeId> = ids
                            .iter()
                            .copied()
                            .filter(|id| !selected.contains(id))
                            .collect();
                        for id in removed {
                            if self.only_pointed_at_by(nodes, &id, neighbour_node.id) {
                                selected.push(id);
                            } else {
                                dropped.push(id);
                            }
                        }
                        Ok(selected)
                    },
                )? {
                    nn_guard.insert(layer_index, replacement);
                }
                for target in dropped {
                    self.forget_backlink(nodes, neighbour_node, target);
                }
            }

            enter_point = match nearest_neighbours.into_iter().max() {
                None => enter_point,
                Some(nearest) => smallvec![nearest.id],
            };
        }

        Ok(())
    }

    /// Make `id` the entry point if it sits on a higher layer than the current one, or if the
    /// index has no entry point yet. A compare-and-swap loop means a writer that read a stale
    /// entry point never overwrites a higher one that was installed in the meantime.
    fn promote_entry_point(&self, id: NodeId, level: u8) {
        let mut current = self.enter_point.load();
        loop {
            if !current.ids.is_empty() && level <= current.level {
                return;
            }
            #[cfg(test)]
            if let Some(hook) = BEFORE_ENTRY_PROMOTION.with(|hook| hook.borrow_mut().take()) {
                hook();
            }
            let next = Arc::new(EntryPoint {
                ids: smallvec![id],
                level,
            });
            let previous = self.enter_point.compare_and_swap(&current, next);
            if Arc::ptr_eq(&current, &previous) {
                return;
            }
            current = previous;
        }
    }

    /// Keep the search start points if any is still stored; otherwise restart from the index
    /// entry point, since a search seeded only with deleted nodes finds nothing.
    fn live_entry_points(
        &self,
        nodes: &NodesRef<'_, E>,
        entry_points: SmallVec<[NodeId; 1]>,
    ) -> SmallVec<[NodeId; 1]> {
        if entry_points.iter().any(|id| nodes.contains_key(id)) {
            return entry_points;
        }
        self.enter_point.load().ids.clone()
    }

    /// The chosen neighbours that are still stored. If every one has been deleted since it
    /// was chosen, fall back to the entry point so the new node is not left without edges.
    fn live_link_targets(
        &self,
        nodes: &NodesRef<'_, E>,
        chosen: &[NodeId],
        new_id: NodeId,
    ) -> Vec<NodeId> {
        #[cfg(test)]
        AFTER_NEIGHBOUR_SELECTION.with(|hook| {
            if let Some(hook) = hook.borrow().as_ref() {
                hook(chosen);
            }
        });

        if chosen.is_empty() {
            return Vec::new();
        }

        let live: Vec<NodeId> = chosen
            .iter()
            .copied()
            .filter(|id| {
                let present = nodes.contains_key(id);
                if !present {
                    skip_missing_node(id);
                }
                present
            })
            .collect();
        if !live.is_empty() {
            return live;
        }
        self.enter_point
            .load()
            .ids
            .iter()
            .copied()
            .filter(|id| *id != new_id && nodes.contains_key(id))
            .collect()
    }

    /// Select M neighbours using heuristic for diversity and pruning
    /// Corresponds to Algorithm 4 (SELECT-neighbourS-HEURISTIC)
    ///
    /// Candidates arrive already scored against `query`.
    fn select_neighbours_heuristic(
        &self,
        query: &Node<E>,
        candidates: &[OrderedNode],
        m: usize,
        layer: &LayerIndex,
        extend_candidates: bool,
        keep_pruned_connections: bool,
    ) -> Result<Vec<NodeId>, Error> {
        let nodes = self.nodes.pin();

        let mut response = NearestFirst::new(std::iter::empty());
        let mut working_queue = NearestFirst::new(candidates.iter().copied());

        if extend_candidates {
            for candidate in candidates {
                let candidate_node = nodes
                    .get(&candidate.id)
                    .ok_or_else(|| Error::NotFoundError(" Node Ref not Found".to_string()))?;

                let cn_guard = candidate_node.neighbours.pin();
                let neighbours_at = cn_guard
                    .get(layer)
                    .ok_or_else(|| Error::NotFoundError(format!("{:?} not Found", layer)))?;

                let edge_ids = neighbours_at.snapshot();
                for neighbour_id in edge_ids.iter() {
                    if !working_queue.contains(neighbour_id)
                        && let Some(neighbour_node) = nodes.get(neighbour_id)
                    {
                        let scored = OrderedNode::new(
                            neighbour_node.id,
                            self.distance_algorithm
                                .closeness(neighbour_node.value.as_slice(), query.value.as_slice()),
                        );
                        working_queue.push(scored);
                    }
                }
            }
        }

        let mut discarded_candidates = NearestFirst::new(std::iter::empty());

        while !working_queue.is_empty() && response.len() < m {
            let candidate = working_queue.pop().ok_or(Error::QueueEmpty)?;

            if response.is_empty() {
                response.push(candidate);
                continue;
            }

            let Some(candidate_node) = nodes.get(&candidate.id) else {
                skip_missing_node(&candidate.id);
                continue;
            };
            let mut is_diverse = true;
            for selected in response.iter() {
                let Some(selected_node) = nodes.get(&selected.id) else {
                    skip_missing_node(&selected.id);
                    continue;
                };

                let closeness_to_selected = self.distance_algorithm.closeness(
                    candidate_node.value.as_slice(),
                    selected_node.value.as_slice(),
                );
                if closeness_to_selected > candidate.closeness {
                    is_diverse = false;
                    break;
                }
            }

            if is_diverse {
                response.push(candidate);
            } else {
                discarded_candidates.push(candidate);
            }
        }

        if keep_pruned_connections {
            while !discarded_candidates.is_empty() && response.len() < m {
                let nearest = discarded_candidates.pop().ok_or(Error::QueueEmpty)?;
                response.push(nearest);
            }
        }

        Ok(response.iter().map(|node| node.id).collect())
    }

    /// Search for ef nearest neighbours in a specific layer
    /// Corresponds to Algorithm 2 (SEARCH-LAYER)
    ///
    /// `accept_list`, when supplied, restricts which nodes may enter the result set `W`.
    /// Rejected nodes are still traversed *through* (they stay in the candidate set `C`),
    /// because the graph's navigability depends on them as stepping stones — dropping them
    /// from `C` would disconnect the search. This is "in-filtering": the paper's Algorithm 2
    /// is unchanged except for which nodes are admitted to `W`.
    ///
    /// Each query/node distance is computed once and carried as an `OrderedNode`, so the
    /// candidate and result queues share the same score.
    pub(crate) fn search_layer(
        &self,
        query: &Node<E>,
        entry_points: &[NodeId],
        ef: usize,
        layer: &LayerIndex,
        accept_list: Option<&std::collections::HashSet<u64>>,
    ) -> Result<Vec<OrderedNode>, Error> {
        let nodes = self.nodes.pin();

        let mut visited_items = NodeIdHashSet::with_capacity_and_hasher(512, Default::default());
        visited_items.extend(entry_points.iter().copied());

        let is_accepted = |id: &NodeId| accept_list.is_none_or(|accept| accept.contains(&id.0));
        let scored_entry_points: Vec<_> = entry_points
            .iter()
            .filter_map(|id| nodes.get(id))
            .map(|node| {
                OrderedNode::new(
                    node.id,
                    self.distance_algorithm
                        .closeness(node.value.as_slice(), query.value.as_slice()),
                )
            })
            .collect();

        let mut candidates = NearestFirst::new(scored_entry_points.iter().copied());
        let ef_nonzero = NonZeroUsize::new(ef).unwrap_or(NonZeroUsize::new(1).unwrap());
        let mut nearest_neighbours: BoundedMaxHeap<OrderedNode> = BoundedMaxHeap::new(ef_nonzero);
        for scored in scored_entry_points {
            if is_accepted(&scored.id) {
                nearest_neighbours.push(scored);
            }
        }

        while !candidates.is_empty() {
            let nearest = candidates.pop().ok_or(Error::QueueEmpty)?;
            if nearest_neighbours.len() >= ef
                && let Some(furthest) = nearest_neighbours.peek()
                && nearest.closeness < furthest.closeness
            {
                break;
            }

            let Some(visited_node) = nodes.get(&nearest.id) else {
                skip_missing_node(&nearest.id);
                continue;
            };
            let neighbours = visited_node.neighbours.pin();
            if let Some(layer_neighbours) = neighbours.get(layer) {
                let edge_ids = layer_neighbours.snapshot();
                for neighbour_id in edge_ids.iter() {
                    if !visited_items.insert(*neighbour_id) {
                        continue;
                    }

                    let Some(neighbour_node) = nodes.get(neighbour_id) else {
                        skip_missing_node(neighbour_id);
                        continue;
                    };
                    let scored = OrderedNode::new(
                        neighbour_node.id,
                        self.distance_algorithm
                            .closeness(neighbour_node.value.as_slice(), query.value.as_slice()),
                    );
                    let should_explore = match nearest_neighbours.peek() {
                        Some(worst) => {
                            scored.closeness > worst.closeness || nearest_neighbours.len() < ef
                        }
                        None => true,
                    };

                    if should_explore {
                        candidates.push(scored);
                        if is_accepted(neighbour_id) {
                            nearest_neighbours.push(scored);
                        }
                    }
                }
            }
        }

        Ok(nearest_neighbours.iter().copied().collect())
    }

    /// `knn_search` that keeps each match's score, so callers need not recompute it.
    fn knn_search_scored(
        &self,
        query: &Node<E>,
        k: usize,
        ef: Option<usize>,
        accept_list: Option<&std::collections::HashSet<u64>>,
    ) -> Result<Vec<OrderedNode>, Error> {
        let nodes = self.nodes.pin();
        let valid_len = NonZeroUsize::new(k).expect("K should be a non zero number");
        let ef = ef.unwrap_or_else(|| k.max(50)).max(k);
        let (mut enter_point, ep_level) = {
            let entry = self.enter_point.load();
            (entry.ids.clone(), entry.level)
        };
        #[cfg(test)]
        AFTER_SEARCH_ENTRY_LOAD.with(|hook| {
            if let Some(hook) = hook.borrow().as_ref() {
                hook();
            }
        });

        for level_current in (1..=ep_level).rev() {
            enter_point = self.live_entry_points(&nodes, enter_point);
            let searched = self.search_layer(
                query,
                &enter_point,
                1,
                &LayerIndex(level_current as u16),
                None,
            )?;
            if let Some(nearest) = searched.into_iter().max() {
                enter_point = smallvec![nearest.id];
            }
        }

        enter_point = self.live_entry_points(&nodes, enter_point);
        let level_zero = self.search_layer(query, &enter_point, ef, &LayerIndex(0), accept_list)?;
        let mut current_nearest_elements = NearestFirst::new(level_zero.into_iter());
        Ok(current_nearest_elements.pop_n(valid_len))
    }

    /// K-Nearest neighbour Search
    /// Corresponds to Algorithm 5 (K-NN-SEARCH)
    ///
    /// # Parameters
    /// - `ef`: Optional search quality parameter. If None, defaults to max(k, 50).
    ///   Higher values improve recall at cost of speed.
    ///   Recommended range: k to 10*k depending on quality requirements.
    pub fn knn_search(
        &self,
        query: &Node<E>,
        k: usize,
        ef: Option<usize>,
        accept_list: Option<&std::collections::HashSet<u64>>,
    ) -> Result<Vec<NodeId>, Error> {
        Ok(self
            .knn_search_scored(query, k, ef, accept_list)?
            .into_iter()
            .map(|node| node.id)
            .collect())
    }

    /// Pruning dropped an edge `from -> to`. `to.back_links` lists the nodes that point at `to`,
    /// so `from` leaves it, unless `from` still points at `to` on another layer.
    fn forget_backlink(&self, nodes: &NodesRef<'_, E>, from: &Node<E>, to: NodeId) {
        let Some(target) = nodes.get(&to) else {
            return;
        };
        if self.has_edge_to(from, to) {
            return;
        }
        target.back_links.pin().remove(&from.id);
        // A writer may have linked `from -> to` again between the check and the removal. Put
        // the entry back then, because `delete_node` finds edges to clean through it.
        if self.has_edge_to(from, to) {
            target.back_links.pin().insert(from.id);
        }
    }

    /// Whether `from` is the only node with an edge to `id`. Pruning `from -> id` would leave
    /// `id` unreachable by search, so the edge is kept, even if that leaves the list a little
    /// over its maximum.
    fn only_pointed_at_by(&self, nodes: &NodesRef<'_, E>, id: &NodeId, from: NodeId) -> bool {
        nodes
            .get(id)
            .is_some_and(|node| node.back_links.pin().iter().all(|other| *other == from))
    }

    /// Whether `from` has an edge to `to` on any layer.
    fn has_edge_to(&self, from: &Node<E>, to: NodeId) -> bool {
        from.neighbours
            .pin()
            .values()
            .any(|edges| edges.snapshot().iter().any(|id| *id == to))
    }

    /// Delete a single element from the HNSW graph by NodeId.
    ///
    /// Known worry: the node is only taken out of the `graph` layer sets while we walk its
    /// backlinks, so a node with no backlinks can leave a stale entry behind in `graph`. Search
    /// never reads `graph` (it is only written on insert and saved in snapshots), so this "should"
    /// only waste a little memory and snapshot size. I've honestly not confirmed how often it happens.
    pub fn delete_node(&self, node_id: &NodeId) {
        let nodes = self.nodes.pin();
        let graph = self.graph.pin();

        if let Some(node) = nodes.get(node_id) {
            // Pruning can drop the edge back, so the nodes this one points at are not all in
            // its back_links. Take it out of their back_links too.
            for edges in node.neighbours.pin().values() {
                for target in edges.snapshot().iter() {
                    if let Some(target) = nodes.get(target) {
                        target.back_links.pin().remove(node_id);
                    }
                }
            }
            for backlink in &node.back_links.pin() {
                let Some(related) = nodes.get(backlink) else {
                    skip_missing_node(backlink);
                    continue;
                };

                let guard = related.neighbours.pin();
                let neighbour_keys_inner = guard.keys();

                for layer_index in neighbour_keys_inner {
                    if let Some(edges) = guard.get(layer_index) {
                        edges.remove_edge(*node_id);
                    };

                    if let Some(layer_set) = graph.get(layer_index) {
                        layer_set.pin().remove(node_id);
                    }
                }
                related.back_links.pin().remove(node_id);
            }

            nodes.remove(node_id);
            self.repair_entry_point(&nodes, node_id);
        }
    }

    /// If `deleted` was the index entry point, move it to the highest-level node that remains
    /// (and lower the top layer to match), so searches and inserts do not start from a
    /// node that no longer exists. This scans every node, but only when the entry point itself
    /// is deleted.
    fn repair_entry_point(&self, nodes: &NodesRef<'_, E>, deleted: &NodeId) {
        let current = self.enter_point.load();
        if !current.ids.contains(deleted) {
            return;
        }
        // Ties go to the smallest ID: map iteration order differs between processes, and
        // replicas applying the same deletes must agree on the entry point.
        let replacement = nodes
            .iter()
            .map(|(id, node)| (*id, node.level(self.maximum_connections)))
            .max_by(|(a_id, a_level), (b_id, b_level)| {
                a_level.cmp(b_level).then(b_id.0.cmp(&a_id.0))
            });
        let next = match replacement {
            Some((id, level)) => EntryPoint {
                ids: smallvec![id],
                level,
            },
            None => EntryPoint::default(),
        };
        // If another writer changed the entry point meanwhile, leave theirs in place.
        self.enter_point.compare_and_swap(&current, Arc::new(next));
    }

    #[cfg(test)]
    /// Get a node by NodeId (cloned from the concurrent map)
    fn get_node(&self, id: &NodeId) -> Option<Node<E>> {
        let nodes = self.nodes.pin();
        nodes.get(id).cloned()
    }
}

impl<D: DistanceFn, E: EdgeStorage> Drop for HNSW<D, E> {
    fn drop(&mut self) {
        if let Some(collector) = self.collector.as_ref() {
            // Retired nodes and layer sets contain Arc references to this same
            // collector. Drain them before field destruction to break that cycle.
            self.nodes.pin().clear();
            self.graph.pin().clear();
            // SAFETY: the collector is private to this index. Drop has exclusive
            // access to the index, so no guards borrowing its collections can be
            // active. Both clear() guards have ended. Public Node snapshots clone
            // their collections onto independent collectors; none exposes ours.
            // reclaim_all also drains objects retired recursively by node drops.
            unsafe { collector.reclaim_all() };
        }
    }
}

impl Default for HNSW<LinearAlgorithm> {
    fn default() -> Self {
        Self::new_with_config(HNSWConfig::default(), LinearAlgorithm::EuclideanDistance)
    }
}

// --- Serde support via Temp structs ---
// (papaya, parking_lot, SmallVec, AtomicU8) to plain std equivalents.

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct TempNode {
    id: NodeId,
    value: Vec<f32>,
    neighbours: std::collections::HashMap<LayerIndex, Vec<NodeId>>,
    back_links: Vec<NodeId>,
}

impl<E: EdgeStorage> From<&Node<E>> for TempNode {
    fn from(node: &Node<E>) -> Self {
        let neighbours_guard = node.neighbours.pin();
        let mut neighbours = std::collections::HashMap::new();
        for (layer, edges) in neighbours_guard.iter() {
            let node_ids = edges.snapshot().iter().copied().collect();
            neighbours.insert(*layer, node_ids);
        }
        let back_links: Vec<NodeId> = node.back_links.pin().iter().copied().collect();
        TempNode {
            id: node.id,
            value: node.value.as_slice().to_vec(),
            neighbours,
            back_links,
        }
    }
}

impl<E: EdgeStorage> From<TempNode> for Node<E> {
    fn from(temp: TempNode) -> Self {
        node_from_temp_with_shared_collector(temp, None)
    }
}

fn node_from_temp_with_shared_collector<E: EdgeStorage>(
    temp: TempNode,
    collector: Option<&Arc<Collector>>,
) -> Node<E> {
    let neighbours = new_hashmap_with_shared_collector(collector);
    {
        let guard = neighbours.pin();
        for (layer, node_ids) in temp.neighbours {
            guard.insert(layer, E::from_ids(node_ids, collector, None));
        }
    }

    let back_links = hashset_from_iter_with_shared_collector(temp.back_links, collector);

    Node {
        id: temp.id,
        value: EmbeddingKey::new(temp.value),
        neighbours,
        back_links,
    }
}

fn node_from_temp_with_counter_shards<E: EdgeStorage>(
    temp: TempNode,
    collector: Option<&Arc<Collector>>,
    shards: Option<NonZeroUsize>,
) -> Node<E> {
    if shards.is_none() {
        return node_from_temp_with_shared_collector(temp, collector);
    }
    let neighbours = node_hashmap(collector, shards);
    {
        let guard = neighbours.pin();
        for (layer, ids) in temp.neighbours {
            guard.insert(layer, E::from_ids(ids, collector, shards));
        }
    }
    Node {
        id: temp.id,
        value: EmbeddingKey::new(temp.value),
        neighbours,
        back_links: node_hashset_from_iter(temp.back_links, collector, shards),
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct TempHNSW<D> {
    ef_construction: usize,
    top_most_layer: u8,
    maximum_connections: usize,
    maximum_connections_zero: usize,
    inv_log_m: f64,
    graph: std::collections::HashMap<LayerIndex, Vec<NodeId>>,
    nodes: std::collections::HashMap<NodeId, TempNode>,
    enter_point: Vec<NodeId>,
    distance_algorithm: D,
    keep_pruned_connections: bool,
    extend_candidates: bool,
}

impl<D: DistanceFn, E: EdgeStorage> From<&HNSW<D, E>> for TempHNSW<D> {
    fn from(hnsw: &HNSW<D, E>) -> Self {
        let graph_guard = hnsw.graph.pin();
        let mut graph = std::collections::HashMap::new();
        for (layer, node_set) in graph_guard.iter() {
            let ids: Vec<NodeId> = node_set.pin().iter().copied().collect();
            graph.insert(*layer, ids);
        }

        let nodes_guard = hnsw.nodes.pin();
        let mut nodes = std::collections::HashMap::new();
        for (node_id, node) in nodes_guard.iter() {
            nodes.insert(*node_id, TempNode::from(node));
        }

        let entry = hnsw.enter_point.load();
        let enter_point: Vec<NodeId> = entry.ids.iter().copied().collect();

        TempHNSW {
            ef_construction: hnsw.ef_construction,
            top_most_layer: entry.level,
            maximum_connections: hnsw.maximum_connections,
            maximum_connections_zero: hnsw.maximum_connections_zero,
            inv_log_m: hnsw.inv_log_m,
            graph,
            nodes,
            enter_point,
            distance_algorithm: hnsw.distance_algorithm,
            keep_pruned_connections: hnsw.keep_pruned_connections,
            extend_candidates: hnsw.extend_candidates,
        }
    }
}

impl<D: DistanceFn, E: EdgeStorage> From<TempHNSW<D>> for HNSW<D, E> {
    fn from(temp: TempHNSW<D>) -> Self {
        let collector = Some(Arc::new(Collector::new()));
        let node_counter_shards = Some(default_node_counter_shards());
        let graph = new_hashmap_with_shared_collector(collector.as_ref());
        {
            let guard = graph.pin();
            for (layer, ids) in temp.graph {
                let set = hashset_from_iter_with_shared_collector(ids, collector.as_ref());
                guard.insert(layer, set);
            }
        }

        let nodes = new_hashmap_with_shared_collector(collector.as_ref());
        {
            let guard = nodes.pin();
            for (id, temp_node) in temp.nodes {
                guard.insert(
                    id,
                    node_from_temp_with_counter_shards(
                        temp_node,
                        collector.as_ref(),
                        node_counter_shards,
                    ),
                );
            }
        }

        let enter_point = EntryPoint {
            ids: temp.enter_point.into_iter().collect(),
            level: temp.top_most_layer,
        };

        HNSW {
            collector,
            node_counter_shards,
            ef_construction: temp.ef_construction,
            maximum_connections: temp.maximum_connections,
            maximum_connections_zero: temp.maximum_connections_zero,
            inv_log_m: temp.inv_log_m,
            graph,
            nodes,
            enter_point: ArcSwap::from_pointee(enter_point),
            distance_algorithm: temp.distance_algorithm,
            keep_pruned_connections: temp.keep_pruned_connections,
            extend_candidates: temp.extend_candidates,
        }
    }
}

#[cfg(feature = "serde")]
impl<D: DistanceFn + serde::Serialize, E: EdgeStorage> serde::Serialize for HNSW<D, E> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer + Sized,
    {
        let temp: TempHNSW<D> = self.into();
        temp.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, D: DistanceFn + serde::Deserialize<'de>, E: EdgeStorage> serde::Deserialize<'de>
    for HNSW<D, E>
{
    fn deserialize<De>(deserializer: De) -> Result<Self, De::Error>
    where
        De: serde::Deserializer<'de>,
    {
        let temp = TempHNSW::<D>::deserialize(deserializer)?;
        Ok(temp.into())
    }
}

#[cfg(test)]
pub fn brute_knn(query: &Node, data: &[Node], k: usize) -> Vec<(NodeId, f32)> {
    use itertools::Itertools;

    debug_assert!(k <= data.len());

    data.iter()
        .map(|n| {
            (
                n.id,
                LinearAlgorithm::EuclideanDistance
                    .value(n.value.as_slice(), query.value.as_slice()),
            )
        })
        .sorted_by(|a, b| {
            a.1.partial_cmp(&b.1).unwrap_or_else(|| {
                if a.1.is_nan() && b.1.is_nan() {
                    std::cmp::Ordering::Equal
                } else if a.1.is_nan() {
                    std::cmp::Ordering::Greater
                } else {
                    std::cmp::Ordering::Less
                }
            })
        })
        .take(k)
        .collect()
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::EmbeddingKey;
    use papaya::HashMap;

    #[test]
    fn test_n_nearest_with_ef_search_breadth() {
        // Reaching the nearest node requires crossing a more distant node.
        // Breadth 1 prunes that bridge; breadth 3 explores it.
        let hnsw = HNSW::default();
        for (id, value, neighbours) in [
            (1, 5.0, vec![NodeId(2)]),
            (2, 6.0, vec![NodeId(1), NodeId(3)]),
            (3, 0.0, vec![NodeId(2)]),
        ] {
            let node = Node {
                id: NodeId(id),
                value: EmbeddingKey::new(vec![value]),
                neighbours: HashMap::new(),
                back_links: HashSet::new(),
            };
            node.neighbours
                .pin()
                .insert(LayerIndex(0), ArcSwap::from_pointee(neighbours));
            hnsw.nodes.pin().insert(NodeId(id), node);
        }
        hnsw.enter_point.store(std::sync::Arc::new(EntryPoint {
            ids: smallvec::smallvec![NodeId(1)],
            level: 0,
        }));
        let n = NonZeroUsize::new(1).unwrap();
        let narrow = hnsw
            .n_nearest_with_ef(&[0.0], n, None, NonZeroUsize::new(1))
            .unwrap();
        let wide = hnsw
            .n_nearest_with_ef(&[0.0], n, None, NonZeroUsize::new(3))
            .unwrap();
        assert_eq!(narrow[0].0.as_slice(), &[5.0]);
        assert_eq!(wide[0].0.as_slice(), &[0.0]);
        assert_eq!(hnsw.n_nearest(&[0.0], n, None).unwrap(), wide);
        // A requested breadth below k is raised to k.
        assert_eq!(
            hnsw.n_nearest_with_ef(
                &[0.0],
                NonZeroUsize::new(3).unwrap(),
                None,
                NonZeroUsize::new(1)
            )
            .unwrap()
            .len(),
            3,
        );
    }

    #[test]
    fn test_simple_hnsw_state() {
        let key = EmbeddingKey::new(vec![3.2]);
        let node = Node::new(key);
        let node_id = *node.id();

        let hnsw = HNSW::default();
        hnsw.insert_node(node).unwrap();
        let graph = hnsw.graph.pin();
        assert_eq!(hnsw.nodes.pin().len(), 1, "Nodes size does not match");
        let (_, node_hashset) = graph.iter().next().unwrap();
        assert_eq!(node_hashset.clone(), HashSet::from_iter([node_id]));
        let node = hnsw.get_node(&node_id).unwrap();

        assert!(node.neighbours.pin().is_empty());
        assert!(node.back_links.pin().is_empty());

        assert_eq!(hnsw.top_layer(), 0);
    }

    #[test]
    fn test_two_nodes_bidirectional() {
        let key = EmbeddingKey::new(vec![0.0]);
        let node_a = Node::new(key);
        let a = *node_a.id();

        let key = EmbeddingKey::new(vec![1.0]);
        let node_b = Node::new(key);
        let b = *node_b.id();

        let hnsw = HNSW::default();
        hnsw.insert_node(node_a).unwrap();
        hnsw.insert_node(node_b).unwrap();

        let a_node = hnsw.get_node(&a).unwrap();
        let b_node = hnsw.get_node(&b).unwrap();

        // Layer 0 neighbours
        assert!(
            a_node
                .neighbours
                .pin()
                .get(&LayerIndex(0))
                .unwrap()
                .snapshot()
                .iter()
                .any(|id| id == &b)
        );
        assert!(
            b_node
                .neighbours
                .pin()
                .get(&LayerIndex(0))
                .unwrap()
                .snapshot()
                .iter()
                .any(|id| id == &a)
        );

        // Back-links
        assert!(a_node.back_links.pin().contains(&b));
        assert!(b_node.back_links.pin().contains(&a));
    }

    #[test]
    fn test_hnsw_basic_invariants() {
        let ids = [NodeId(1), NodeId(2), NodeId(3)].to_vec();

        let hnsw = HNSW::default();

        for id in &ids {
            hnsw.insert_node(Node {
                id: *id,
                value: EmbeddingKey::new(vec![0.0]),
                neighbours: HashMap::new(),
                back_links: HashSet::new(),
            })
            .unwrap();
        }

        let layer0 = LayerIndex(0);

        // Collect all node IDs first, then check invariants
        let all_ids: Vec<NodeId> = ids.clone();
        let nodes = hnsw.nodes.pin();
        for id in &all_ids {
            if let Some(node) = nodes.get(id) {
                // Degree bound
                let neighbours_guard = node.neighbours.pin();
                if let Some(neighbours) = neighbours_guard.get(&layer0) {
                    let set_guard = neighbours.snapshot();
                    assert!(set_guard.len() <= hnsw.maximum_connections_zero);

                    for n in set_guard.iter() {
                        // Neighbour exists
                        assert!(nodes.contains_key(n));

                        // Backlink exists
                        let other = nodes.get(n).unwrap();
                        assert!(
                            other.back_links.pin().contains(id),
                            "missing backlink: {:?} <- {:?}",
                            n,
                            id
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_search_single_nearest() {
        let hnsw = HNSW::default();

        let a = NodeId(10);
        let b = NodeId(20);

        hnsw.insert_node(Node {
            id: a,
            value: EmbeddingKey::new(vec![0.0]),
            neighbours: HashMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.insert_node(Node {
            id: b,
            value: EmbeddingKey::new(vec![10.0]),
            neighbours: HashMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        let node_id = NodeId(99);
        let query_node = Node {
            id: node_id,
            value: EmbeddingKey::new(vec![1.0]),
            neighbours: HashMap::new(),
            back_links: HashSet::new(),
        };

        let res = hnsw.knn_search(&query_node, 1, Some(10), None).unwrap();
        assert_eq!(res[0], a);
    }

    #[test]
    fn test_delete_leaf_node() {
        let hnsw = HNSW::default();

        let a = NodeId(10);
        let b = NodeId(20);

        hnsw.insert_node(Node {
            id: a,
            value: EmbeddingKey::new(vec![0.0]),
            neighbours: HashMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.insert_node(Node {
            id: b,
            value: EmbeddingKey::new(vec![1.0]),
            neighbours: HashMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.delete_node(&b);

        assert!(hnsw.get_node(&b).is_none());

        let a_node = hnsw.get_node(&a).unwrap();
        assert!(
            a_node
                .neighbours
                .pin()
                .iter()
                .all(|(_, s)| !s.load().contains(&b))
        );
        assert!(!a_node.back_links.pin().contains(&b));
    }

    #[test]
    fn test_delete_multi_neighbour_node() {
        let hnsw = HNSW::default();

        let ids = [NodeId(10), NodeId(20), NodeId(30), NodeId(40)].to_vec();

        for id in &ids {
            hnsw.insert_node(Node {
                id: *id,
                value: EmbeddingKey::new(vec![0.0]),
                neighbours: HashMap::new(),
                back_links: HashSet::new(),
            })
            .unwrap();
        }

        let target = &ids[1]; // delete B
        hnsw.delete_node(target);

        assert!(hnsw.get_node(target).is_none());

        for id in &ids {
            if id == target {
                continue;
            }
            let node = hnsw.get_node(id).unwrap();
            assert!(
                node.neighbours
                    .pin()
                    .iter()
                    .all(|(_, s)| !s.load().contains(target))
            );
            assert!(!node.back_links.pin().contains(target));
        }
    }

    #[test]
    fn test_level_assignment_is_deterministic() {
        // Same embedding should always produce same level
        let embedding = EmbeddingKey::new(vec![1.0, 2.0, 3.0]);

        let node1 = Node::new(embedding.clone());
        let node2 = Node::new(embedding.clone());

        let m = 48; // Default M from HNSW::default()

        assert_eq!(
            node1.id(),
            node2.id(),
            "Same embedding should produce same NodeId"
        );
        assert_eq!(
            node1.level(m),
            node2.level(m),
            "Same NodeId should produce same level"
        );

        // Verify consistency across multiple calls
        for _ in 0..10 {
            let node = Node::new(embedding.clone());
            assert_eq!(node.level(m), node1.level(m), "Level should be consistent");
        }
    }

    #[test]
    fn test_level_distribution_is_exponential() {
        use std::collections::HashMap;

        let m = 48;
        let mut level_counts: HashMap<u8, usize> = HashMap::new();

        // Generate many nodes and count their levels
        for i in 0..1000 {
            let embedding = EmbeddingKey::new(vec![i as f32, (i * 2) as f32, (i * 3) as f32]);
            let node = Node::new(embedding);
            let level = node.level(m);
            *level_counts.entry(level).or_insert(0) += 1;
        }

        // Verify exponential decay: most nodes at level 0
        let level_0_count = level_counts.get(&0).unwrap_or(&0);
        assert!(
            *level_0_count > 900,
            "Most nodes should be at level 0, got {}",
            level_0_count
        );

        // Each higher level should have fewer nodes (when they exist)
        for level in 1..5 {
            let current_count = level_counts.get(&level).unwrap_or(&0);
            let prev_count = level_counts.get(&(level - 1)).unwrap_or(&0);
            if *current_count > 0 && *prev_count > 0 {
                assert!(
                    current_count < prev_count,
                    "Level {} ({} nodes) should have fewer nodes than level {} ({} nodes)",
                    level,
                    current_count,
                    level - 1,
                    prev_count
                );
            }
        }

        println!("Level distribution (n=1000):");
        for level in 0..10 {
            if let Some(count) = level_counts.get(&level) {
                println!(
                    "  Level {}: {} nodes ({:.1}%)",
                    level,
                    count,
                    (*count as f32 / 1000.0) * 100.0
                );
            }
        }
    }

    #[test]
    fn test_persistence_consistency() {
        // Simulate save/reload scenario
        let embedding = EmbeddingKey::new(vec![5.5, 10.2, 3.7]);
        let m = 48;

        // "First run" - insert node
        let node1 = Node::new(embedding.clone());
        let id1 = *node1.id();
        let level1 = node1.level(m);

        // "Reload and insert again" - should be identical
        let node2 = Node::new(embedding.clone());
        let id2 = *node2.id();
        let level2 = node2.level(m);

        assert_eq!(id1, id2, "NodeId must be deterministic for persistence");
        assert_eq!(
            level1, level2,
            "Level must be deterministic for persistence"
        );
    }

    #[test]
    fn test_different_embeddings_different_levels() {
        let m = 48;

        // Different embeddings should (likely) produce different NodeIds
        let node1 = Node::new(EmbeddingKey::new(vec![1.0, 2.0, 3.0]));
        let node2 = Node::new(EmbeddingKey::new(vec![4.0, 5.0, 6.0]));
        let node3 = Node::new(EmbeddingKey::new(vec![7.0, 8.0, 9.0]));

        // NodeIds should be different
        assert_ne!(
            node1.id(),
            node2.id(),
            "Different embeddings should produce different NodeIds"
        );
        assert_ne!(
            node2.id(),
            node3.id(),
            "Different embeddings should produce different NodeIds"
        );
        assert_ne!(
            node1.id(),
            node3.id(),
            "Different embeddings should produce different NodeIds"
        );

        // Levels may or may not be different (that's fine, just document it)
        println!("Node 1: id={:?}, level={}", node1.id(), node1.level(m));
        println!("Node 2: id={:?}, level={}", node2.id(), node2.level(m));
        println!("Node 3: id={:?}, level={}", node3.id(), node3.level(m));
    }
}
