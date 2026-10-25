pub mod edges;
pub mod index;
pub mod utils;

/// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
/// similarity function. It then navigates between these localised lists in DFS manner until it
/// gets the values it needs to
use crate::{Closeness, EmbeddingKey};
pub use edges::CompactEdges;
use edges::{EdgeSnapshot, EdgeStorage};
use papaya::{HashMap, HashSet};
use seize::Collector;
use std::sync::Arc;
use std::{collections::BinaryHeap, hash::Hasher, num::NonZeroUsize};

/// A pass-through hasher for NodeId.
///
/// Since NodeId already contains a well-distributed hash (computed via ahash),
/// re-hashing it with SipHash in std::collections::HashSet is wasted work.
/// This hasher just passes the u64 through directly.
#[derive(Default)]
pub(crate) struct PassThroughHasher(u64);

impl Hasher for PassThroughHasher {
    #[inline]
    fn write_u64(&mut self, n: u64) {
        self.0 = n;
    }

    #[inline]
    fn write(&mut self, _bytes: &[u8]) {
        // NodeId always hashes via write_u64; this arm is unreachable in practice.
        unreachable!("PassThroughHasher only supports write_u64");
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

pub(crate) type NodeIdBuildHasher = std::hash::BuildHasherDefault<PassThroughHasher>;
pub(crate) type NodeIdHashSet = std::collections::HashSet<NodeId, NodeIdBuildHasher>;
/// LayerIndex is just a wrapper around u16 to represent a layer in HNSW.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct LayerIndex(pub u16);

impl Eq for LayerIndex {}

impl PartialOrd for LayerIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for LayerIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&(other.0))
    }
}

/// NodeId wraps a u64 hash of the node's embedding to uniquely identify a node across all layers.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct NodeId(pub u64);

/// Node represents a single element in the HNSW graph.
///
/// Each node stores:
/// - `id`: unique identifier
/// - `value`: embedding vector
/// - `neighbours`: map from layer to set of NodeIds of neighbours in that layer
/// - `back_links`: set of NodeIds of nodes that have an edge to us.
///   Used to efficiently update the graph when deleting this node.
///
/// Example of a node:
/// ```text
/// Node {
///     id: 42,
///     value: [0.12, 0.55, 0.77],
///     neighbours: {
///         0: [10, 55, 71],
///         1: [9, 11],
///         2: [88],
///         3: [200, 201]
///     },
///     back_links: [9, 88]
/// }
/// ```
/// This shows that Node 42 participates in layers 0 through 3.
#[derive(Debug)]
pub struct Node<E: EdgeStorage = CompactEdges> {
    id: NodeId,
    value: EmbeddingKey,
    neighbours: HashMap<LayerIndex, E>,
    back_links: HashSet<NodeId>,
}

impl<E: EdgeStorage> Clone for Node<E> {
    fn clone(&self) -> Self {
        let neighbours = HashMap::new();
        {
            let target = neighbours.pin();
            for (layer, edges) in self.neighbours.pin().iter() {
                target.insert(
                    *layer,
                    E::from_ids(edges.snapshot().iter().copied().collect(), None, None),
                );
            }
        }
        let back_links = HashSet::new();
        {
            let target = back_links.pin();
            for id in self.back_links.pin().iter() {
                target.insert(*id);
            }
        }
        Self {
            id: self.id,
            value: self.value.clone(),
            neighbours,
            back_links,
        }
    }
}
impl Node {
    pub fn new(value: EmbeddingKey) -> Self {
        Self::new_with_edges(value)
    }
}

/// Compute deterministic level for a node based on its ID hash.
///
/// Uses exponential distribution: P(level >= l) ≈ (1/M)^l
/// This ensures hierarchical structure with exponentially fewer nodes at higher levels.
//
// Using the NodeId hash ensures that the following are true
// - Deterministic: same embedding gives the same level always
// - Persistent: levels survive serialization/deserialization.
// - Distribution-friendly: replicas assign same levels.
// - Testable: produces reproducible graph structures.
fn compute_node_level(node_id: &NodeId, m: usize) -> u8 {
    let inv_log_m = 1.0 / (m as f64).ln();
    // Extract uniform random value from NodeId's u64 hash
    // Use lower 53 bits to map cleanly to f64 mantissa
    let hash_bits = node_id.0;
    let uniform_bits = hash_bits & ((1u64 << 53) - 1);
    let unif: f64 = (uniform_bits as f64) / ((1u64 << 53) as f64);
    // Avoid ln(0) which would give infinity
    let adjusted_unif = if unif < 1e-10 { 1e-10 } else { unif };
    // Apply inverse exponential CDF: l = floor(-ln(U) * mL)
    let level = (-adjusted_unif.ln() * inv_log_m).floor();
    // Clamp to u8 range (very very unlikely to exceed 255, but be safe)
    level.min(255.0) as u8
}

impl<E: EdgeStorage> Node<E> {
    /// Get the deterministic level for this node.
    /// Level is computed from the node's ID hash using exponential distribution.
    pub fn level(&self, m: usize) -> u8 {
        compute_node_level(&self.id, m)
    }

    pub(crate) fn new_with_edges(value: EmbeddingKey) -> Self {
        let id = get_node_id(&value.0);
        Self {
            id,
            value,
            neighbours: HashMap::new(),
            back_links: HashSet::with_capacity(1),
        }
    }

    pub(crate) fn new_with_shared_collector(
        value: EmbeddingKey,
        collector: &Arc<Collector>,
    ) -> Self {
        Self {
            id: get_node_id(&value.0),
            value,
            neighbours: HashMap::builder()
                .capacity(1)
                .shared_collector(Arc::clone(collector))
                .build(),
            back_links: HashSet::builder()
                .capacity(1)
                .shared_collector(Arc::clone(collector))
                .build(),
        }
    }

    pub(crate) fn new_with_shared_collector_and_counter_shards(
        value: EmbeddingKey,
        collector: &Arc<Collector>,
        shards: NonZeroUsize,
    ) -> Self {
        Self {
            id: get_node_id(&value.0),
            value,
            neighbours: HashMap::builder()
                .capacity(1)
                .shared_collector(Arc::clone(collector))
                .counter_shards(shards)
                .build(),
            back_links: HashSet::builder()
                .capacity(1)
                .shared_collector(Arc::clone(collector))
                .counter_shards(shards)
                .build(),
        }
    }

    /// get identifier
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    /// get the embedding value
    pub fn value(&self) -> &EmbeddingKey {
        &self.value
    }

    /// Optional helper: add a neighbour at a specific layer
    pub fn add_neighbour(&self, layer: LayerIndex, neighbour: NodeId) {
        let guard = self.neighbours.pin();
        let edges = guard.get_or_insert_with(layer, || E::from_ids(Vec::new(), None, None));
        edges.insert_edge(neighbour);
    }

    /// Optional helper: remove a neighbour at a specific layer
    pub fn remove_neighbour(&self, layer: LayerIndex, neighbour: NodeId) {
        let guard = self.neighbours.pin();
        if let Some(edges) = guard.get(&layer) {
            edges.remove_edge(neighbour);
        }
    }
}

/// A node paired with how close it is to the query.
///
/// Ordered by `closeness` — greater is closer, for every metric — and tie-broken on
/// `NodeId` so replicas agree on the order of equally close nodes. `Eq` and `Ord` agree
/// with each other, which `BinaryHeap` relies on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct OrderedNode {
    pub(crate) id: NodeId,
    pub(crate) closeness: Closeness,
}

impl OrderedNode {
    pub(crate) fn new(id: NodeId, closeness: Closeness) -> Self {
        Self { id, closeness }
    }
}

impl PartialOrd for OrderedNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.closeness
            .cmp(&other.closeness)
            .then_with(|| self.id.0.cmp(&other.id.0))
    }
}

/// Candidate queue that pops the closest node first.
///
/// A plain max-heap: `OrderedNode` orders by closeness, so the closest is always the
/// greatest, whichever metric is in use. There is no min/max variant to pick.
pub(crate) struct NearestFirst {
    heap: BinaryHeap<OrderedNode>,
}

impl NearestFirst {
    /// Build a queue from nodes whose query distance the caller has already computed.
    pub(crate) fn new(nodes: impl Iterator<Item = OrderedNode>) -> Self {
        Self {
            heap: nodes.collect(),
        }
    }

    pub(crate) fn push(&mut self, node: OrderedNode) {
        self.heap.push(node);
    }

    pub(crate) fn pop(&mut self) -> Option<OrderedNode> {
        self.heap.pop()
    }

    pub(crate) fn pop_n(&mut self, n: NonZeroUsize) -> Vec<OrderedNode> {
        (0..n.get()).filter_map(|_| self.heap.pop()).collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.heap.len()
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &OrderedNode> {
        self.heap.iter()
    }

    pub(crate) fn contains(&self, node_id: &NodeId) -> bool {
        self.heap.iter().any(|node| &node.id == node_id)
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }
}

/// Hash a float vector to a deterministic u64.
/// Uses fixed seed for deterministic hashing across restarts and platforms.
pub use ahnlich_types::utils::hash_f32_vec as hash_vec;

pub fn get_node_id(value: &[f32]) -> NodeId {
    NodeId(hash_vec(value))
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone, Copy)]
pub struct HNSWConfig {
    pub ef_construction: usize,
    pub maximum_connections: usize,
    pub maximum_connections_zero: usize,

    pub extend_candidates: bool,
    pub keep_pruned_connections: bool,
}

impl Default for HNSWConfig {
    fn default() -> Self {
        let maximum_connections = 16;
        Self {
            ef_construction: 100,
            maximum_connections,
            maximum_connections_zero: maximum_connections * 2,
            extend_candidates: false,
            keep_pruned_connections: false,
        }
    }
}
