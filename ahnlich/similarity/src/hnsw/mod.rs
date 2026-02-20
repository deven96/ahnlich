#![allow(dead_code)]

pub mod index;

/// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
/// similarity function. It then navigates between these localised lists in DFS manner until it
/// gets the values it needs to
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
    value: Vec<f32>,
    neighbours: BTreeMap<LayerIndex, HashSet<NodeId>>,
    back_links: HashSet<NodeId>,
}

impl Node {
    pub fn new(value: Vec<f32>) -> Self {
        Self {
            id: get_node_id(&value),
            value,
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

struct OrderedNode((NodeId, f32));

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
    F: Fn(&[f32], &[f32]) -> f32,
{
    heap: BinaryHeap<OrderedNode>,
    similarity: F,

    query: Vec<f32>,
}

impl<F> MaxHeapQueue<F>
where
    F: Fn(&[f32], &[f32]) -> f32,
{
    fn from_nodes(nodes: &[Node], query: &Node, similarity_function: F) -> Self {
        let heap = nodes
            .iter()
            .map(|node| {
                let similarity = similarity_function(&node.value, &query.value);
                OrderedNode((node.id.clone(), similarity))
            })
            .collect::<BinaryHeap<_>>();
        Self {
            heap,
            similarity: similarity_function,
            query: query.value.clone(),
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

    fn peak(&self) -> Option<&OrderedNode> {
        self.heap.peek()
    }

    fn push(&mut self, node: &Node) {
        let distance = (self.similarity)(&node.value, &self.query);
        let ordered = OrderedNode((node.id.clone(), distance));
        self.heap.push(ordered)
    }

    fn contains(&self, node_id: &NodeId) -> bool {
        self.heap.iter().any(|x| &(x.0.0) == node_id)
    }
}

struct MinHeapQueue<F>
where
    F: Fn(&[f32], &[f32]) -> f32,
{
    heap: BinaryHeap<Reverse<OrderedNode>>,
    similarity: F,
    query: Vec<f32>,
}

impl<F> MinHeapQueue<F>
where
    F: Fn(&[f32], &[f32]) -> f32,
{
    fn from_nodes(nodes: &[Node], query: &Node, similarity_function: F) -> Self {
        let heap = nodes
            .iter()
            .map(|node| {
                let similarity = similarity_function(&node.value, &query.value);
                let ordered_node = OrderedNode((node.id.clone(), similarity));
                Reverse(ordered_node)
            })
            .collect::<BinaryHeap<_>>();
        Self {
            heap,
            similarity: similarity_function,
            query: query.value.clone(),
        }
    }

    fn push(&mut self, node: &Node) {
        let distance = (self.similarity)(&node.value, &self.query);
        let ordered = OrderedNode((node.id.clone(), distance));
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

    fn peak(&self) -> Option<&Reverse<OrderedNode>> {
        self.heap.peek()
    }

    fn contains(&self, node_id: &NodeId) -> bool {
        self.heap.iter().any(|x| &(x.0.0.0) == node_id)
    }
}

fn euclidean_distance_comp(first: &[f32], second: &[f32]) -> f32 {
    // Calculate the sum of squared differences for each dimension
    let mut sum_of_squared_differences = 0.0;
    for (&coord1, &coord2) in first.iter().zip(second.iter()) {
        let diff = coord1 - coord2;
        sum_of_squared_differences += diff * diff;
    }

    // Calculate the square root of the sum of squared differences
    f32::sqrt(sum_of_squared_differences)
}

fn dot_product_comp(first: &[f32], second: &[f32]) -> f32 {
    first
        .iter()
        .zip(&second.to_vec())
        .map(|(&x, &y)| x * y)
        .sum::<f32>()
}

fn get_node_id(value: &[f32]) -> NodeId {
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
