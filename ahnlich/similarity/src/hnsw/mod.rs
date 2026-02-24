#![allow(dead_code)]

pub mod index;

/// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
/// similarity function. It then navigates between these localised lists in DFS manner until it
/// gets the values it needs to
use crate::{DistanceFn, EmbeddingKey};
use std::{
    cmp::Reverse,
    collections::{BinaryHeap, HashSet, btree_map::BTreeMap},
    hash::Hasher,
    num::NonZeroUsize,
};

/// LayerIndex is just a wrapper around u16 to represent a layer in HNSW.
#[derive(Debug, Clone, PartialEq)]
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
pub struct NodeId(pub u64);

/// Node represents a single element in the HNSW graph.
///
/// Each node stores:
/// - `id`: unique identifier
/// - `value`: embedding vector
/// - `neighbours`: map from layer to set of NodeIds of neighbours in that layer
/// - `back_links`: set of NodeIds of nodes that consider us a neighbour.
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
#[derive(Debug, Clone)]
pub struct Node {
    id: NodeId,
    value: EmbeddingKey,
    neighbours: BTreeMap<LayerIndex, HashSet<NodeId>>,
    back_links: HashSet<NodeId>,
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

impl Node {
    /// Get the deterministic level for this node.
    /// Level is computed from the node's ID hash using exponential distribution.
    pub fn level(&self, m: usize) -> u8 {
        compute_node_level(&self.id, m)
    }

    pub fn new(value: Vec<f32>) -> Self {
        let id = get_node_id(&value);
        Self {
            id,
            value: EmbeddingKey::new(value),
            neighbours: BTreeMap::new(),
            back_links: HashSet::with_capacity(1),
        }
    }

    /// get identifier
    pub fn id(&self) -> &NodeId {
        &self.id
    }

    /// Optional helper: get neighbours at a specific layer
    pub fn neighbours_at(&self, layer: &LayerIndex) -> Option<&HashSet<NodeId>> {
        self.neighbours.get(layer)
    }

    /// Optional helper: add a neighbour at a specific layer
    pub fn add_neighbour(&mut self, layer: LayerIndex, neighbour: NodeId) {
        self.neighbours
            .entry(layer)
            .or_insert(HashSet::from_iter([neighbour]));
    }

    /// Optional helper: remove a neighbour at a specific layer
    pub fn remove_neighbour(&mut self, layer: LayerIndex, neighbour: NodeId) {
        if let Some(set) = self.neighbours.get_mut(&layer) {
            set.remove(&neighbour);
        }
    }
}

// TODO: Hnsw needs to define a similarity algorithm to compare two nodes
// - Queues needs

pub(crate) struct OrderedNode(pub(crate) (NodeId, f32));

impl PartialEq for OrderedNode {
    fn eq(&self, other: &Self) -> bool {
        ((self.0).0) == ((other.0).0)
    }
}

impl Eq for OrderedNode {}

impl PartialOrd for OrderedNode {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedNode {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0)
            .1
            .partial_cmp(&(other.0).1)
            .unwrap_or(std::cmp::Ordering::Less)
    }
}

struct MaxHeapQueue<F>
where
    F: DistanceFn,
{
    heap: BinaryHeap<OrderedNode>,
    distance_algorithm: F,

    query: Vec<f32>,
}

impl<F: DistanceFn> MaxHeapQueue<F> {
    fn from_nodes<'a>(
        nodes: impl Iterator<Item = &'a Node>,
        query: &Node,
        distance_algorithm: F,
    ) -> Self {
        let heap = nodes
            .map(|node| {
                let similarity =
                    distance_algorithm.distance(node.value.as_slice(), query.value.as_slice());
                OrderedNode((node.id, similarity))
            })
            .collect::<BinaryHeap<_>>();
        Self {
            heap,
            distance_algorithm,
            query: query.value.as_slice().to_vec(),
        }
    }

    fn pop(&mut self) -> Option<OrderedNode> {
        self.heap.pop()
    }

    fn pop_n(&mut self, n: NonZeroUsize) -> Vec<OrderedNode> {
        (0..n.get()).filter_map(|_| self.heap.pop()).collect()
    }

    fn len(&self) -> usize {
        self.heap.len()
    }

    fn peek(&self) -> Option<&OrderedNode> {
        self.heap.peek()
    }

    fn push(&mut self, node: &Node) {
        let distance = self
            .distance_algorithm
            .distance(node.value.as_slice(), &self.query);
        let ordered = OrderedNode((node.id, distance));
        self.heap.push(ordered)
    }

    fn contains(&self, node_id: &NodeId) -> bool {
        self.heap.iter().any(|x| &(x.0.0) == node_id)
    }
}

struct MinHeapQueue<F>
where
    F: DistanceFn,
{
    heap: BinaryHeap<Reverse<OrderedNode>>,
    distance_algorithm: F,
    query: Vec<f32>,
}

impl<F: DistanceFn> MinHeapQueue<F> {
    fn from_nodes<'a>(
        nodes: impl Iterator<Item = &'a Node>,
        query: &Node,
        distance_algorithm: F,
    ) -> Self {
        let heap = nodes
            .map(|node| {
                let similarity =
                    distance_algorithm.distance(node.value.as_slice(), query.value.as_slice());
                let ordered_node = OrderedNode((node.id, similarity));
                Reverse(ordered_node)
            })
            .collect::<BinaryHeap<_>>();
        Self {
            heap,
            distance_algorithm,
            query: query.value.as_slice().to_vec(),
        }
    }

    fn push(&mut self, node: &Node) {
        let distance = self
            .distance_algorithm
            .distance(node.value.as_slice(), &self.query);
        let ordered = OrderedNode((node.id, distance));
        self.heap.push(Reverse(ordered))
    }

    fn pop(&mut self) -> Option<Reverse<OrderedNode>> {
        self.heap.pop()
    }

    fn pop_n(&mut self, n: NonZeroUsize) -> Vec<Reverse<OrderedNode>> {
        (0..n.get()).filter_map(|_| self.heap.pop()).collect()
    }

    fn len(&self) -> usize {
        self.heap.len()
    }

    fn peek(&self) -> Option<&Reverse<OrderedNode>> {
        self.heap.peek()
    }

    fn contains(&self, node_id: &NodeId) -> bool {
        self.heap.iter().any(|x| &(x.0.0.0) == node_id)
    }
}

pub fn get_node_id(value: &[f32]) -> NodeId {
    use ahash::RandomState;
    use std::hash::BuildHasher;
    // Fixed seed so NodeId is deterministic across restarts and platforms.
    // AHasher::default() is randomly seeded per-process (DoS protection for maps),
    // which would break any future snapshot/persistence of NodeIds.
    let mut hasher = RandomState::with_seeds(0, 0, 0, 0).build_hasher();
    for element in value.iter() {
        hasher.write_u32(element.to_bits());
    }
    NodeId(hasher.finish())
}
