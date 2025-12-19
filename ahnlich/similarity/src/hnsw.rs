#![allow(dead_code)]
/// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
/// similarity function. It then navigates between these localised lists in DFS manner until it
/// gets the values it needs to
use std::{
    collections::{BinaryHeap, HashMap, HashSet, btree_map::BTreeMap},
    hash::Hash,
};

/// LayerIndex is just a wrapper around u16 to represent a layer in HNSW.
#[derive(Debug, Clone)]
pub struct LayerIndex(pub u16);

impl PartialEq for LayerIndex {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&(other.0))
    }
}
impl Eq for LayerIndex {}

impl PartialOrd for LayerIndex {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&(other.0))
    }
}
impl Ord for LayerIndex {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&(other.0))
    }
}

/// NodeId wraps String(hash of node embeddings) to uniquely identify a node across all layers.
#[derive(Debug, Clone)]
pub struct NodeId(pub String);

impl PartialEq for NodeId {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&(other.0))
    }
}
impl Eq for NodeId {}

impl PartialOrd for NodeId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&(other.0))
    }
}
impl Ord for NodeId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&(other.0))
    }
}

impl Hash for NodeId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

/// HNSW represents a Hierarchical Navigable Small World graph.
///
/// The graph is organized into multiple layers. Each layer contains a set of node IDs,
/// and each node holds its neighbors per layer along with its embedding vector.
/// This separation allows efficient lookups, prevents duplicate nodes per layer,
/// and supports deletion operations.
///
/// Design rationale:
/// 1. `nodes` is the single source of truth: all Node structs live here, keyed by NodeId.
/// 2. `graph` maps each layer to a `HashSet` of NodeIds, ensuring uniqueness per layer
///    and fast removal when deleting nodes.
/// 3. Deletion is fully supported:
///    - Remove the node ID from the `graph` for all layers where it exists.
///    - Remove the node from `nodes`.
///    - Remove the node ID from all neighbors of other nodes (using back-links/referrals).
///      This ensures no stale references remain in the graph.
///
/// Example of usage:
/// ```text
/// Layer 0: {42, 10, 55}
/// Layer 1: {42, 11, 9}
/// Layer 2: {42, 88}
/// Layer 3: {42, 200, 201}
///
/// Node 42 participates in layers 0–3, with neighbors stored per layer and
/// back-links automatically updated upon deletion.
/// ```
#[derive(Default)]
pub struct HNSW {
    /// Breadth of search during insertion (efConstruction)
    pub ef_construction: Option<u8>,

    /// Top-most layer index in the graph (L)
    pub top_most_layer: u8,

    /// Maximum number of connections per node (M)
    pub maximum_connections: u8,

    /// Precomputed value 1 / ln(M) used in level generation
    pub inv_log_m: f64,

    /// Nodes in each layer
    ///
    /// Each layer index maps to a set of NodeIds.
    /// This ensures uniqueness per layer and allows easy removal during deletion.
    graph: BTreeMap<LayerIndex, HashSet<NodeId>>,

    /// All nodes in the HNSW
    ///
    /// The single source of truth for all node data.
    /// Keys are NodeId, values are the Node structs containing embeddings and neighbors.
    nodes: HashMap<NodeId, Node>,
}

impl HNSW {
    /// Insert a new element into the HNSW graph
    /// Corresponds to Algorithm 1 (INSERT)
    pub fn insert(&mut self, value: Node) -> NodeId {
        // internally uses SEARCH-LAYER, SELECT-NEIGHBORS
        todo!()
    }

    /// Search for ef nearest neighbors in a specific layer
    /// Corresponds to Algorithm 2 (SEARCH-LAYER)
    pub fn search_layer(
        &self,
        query: &Vec<f64>,
        entry_points: &[NodeId],
        ef: usize,
        layer: LayerIndex,
    ) -> Vec<NodeId> {
        todo!()
    }

    /// Select M neighbors simply based on distance
    /// Corresponds to Algorithm 3 (SELECT-NEIGHBORS-SIMPLE)
    pub fn select_neighbors_simple(
        &self,
        base: NodeId,
        candidates: &[NodeId],
        m: usize,
    ) -> Vec<NodeId> {
        todo!()
    }

    /// Select M neighbors using heuristic for diversity and pruning
    /// Corresponds to Algorithm 4 (SELECT-NEIGHBORS-HEURISTIC)
    pub fn select_neighbors_heuristic(
        &self,
        base: NodeId,
        candidates: &[NodeId],
        m: usize,
        layer: LayerIndex,
        extend_candidates: bool,
        keep_pruned_connections: bool,
    ) -> Vec<NodeId> {
        todo!()
    }

    /// K-Nearest Neighbor Search
    /// Corresponds to Algorithm 5 (K-NN-SEARCH)
    pub fn knn_search(&self, query: &Vec<f64>, k: usize, ef: Option<usize>) -> Vec<NodeId> {
        todo!()
    }

    /// delete an new element from HNSW graph
    pub fn delete(&mut self, node_id: &NodeId) {
        todo!()
    }

    /// Optional helper to get a node by NodeId efficiently
    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        todo!()
    }
}

/// Node represents a single element in the HNSW graph.
///
/// Each node stores:
/// - `id`: unique identifier
/// - `value`: embedding vector
/// - `neighbours`: map from layer to set of NodeIds of neighbors in that layer
/// - `back_links`: set of NodeIds of nodes that consider us a neighbor.
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
pub struct Node {
    id: NodeId,
    value: Vec<f64>,
    neighbours: BTreeMap<LayerIndex, HashSet<NodeId>>,
    back_links: HashSet<NodeId>,
}

impl Node {
    /// Optional helper: get neighbors at a specific layer
    pub fn neighbors_at(&self, layer: LayerIndex) -> Option<&HashSet<NodeId>> {
        self.neighbours.get(&layer)
    }

    /// Optional helper: add a neighbor at a specific layer
    pub fn add_neighbor(&mut self, layer: LayerIndex, neighbor: NodeId) {
        self.neighbours
            .entry(layer)
            .or_insert(HashSet::from_iter([neighbor]));
    }

    /// Optional helper: remove a neighbor at a specific layer
    pub fn remove_neighbor(&mut self, layer: LayerIndex, neighbor: NodeId) {
        if let Some(set) = self.neighbours.get_mut(&layer) {
            set.remove(&neighbor);
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_simple_hnsw_state() {
        let node_id = NodeId(String::from("Node1"));
        let node = Node {
            id: node_id.clone(),
            value: vec![3.2],
            back_links: HashSet::new(),
            neighbours: BTreeMap::new(),
        };

        let mut hnsw = HNSW::default();

        hnsw.insert(node);
        let graph_values: Vec<_> = hnsw.graph.values().collect();

        assert_eq!(hnsw.nodes.len(), 1, "Nodes size does not match");
        let node_hashset = graph_values.first().cloned().unwrap().clone();
        assert_eq!(node_hashset, HashSet::from_iter([node_id.clone()]));
        let node = hnsw.get_node(&node_id).unwrap();

        assert!(node.neighbours.is_empty());
        assert!(node.back_links.is_empty());

        assert_eq!(hnsw.top_most_layer, 0);
    }

    #[test]
    fn test_two_nodes_bidirectional() {
        let a = NodeId("A".into());
        let b = NodeId("B".into());

        let node_a = Node {
            id: a.clone(),
            value: vec![0.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        };

        let node_b = Node {
            id: b.clone(),
            value: vec![1.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        };

        let mut hnsw = HNSW::default();
        hnsw.insert(node_a);
        hnsw.insert(node_b);

        let a_node = hnsw.get_node(&a).unwrap();
        let b_node = hnsw.get_node(&b).unwrap();

        // Layer 0 neighbors
        assert!(a_node.neighbours.get(&LayerIndex(0)).unwrap().contains(&b));
        assert!(b_node.neighbours.get(&LayerIndex(0)).unwrap().contains(&a));

        // Back-links
        assert!(a_node.back_links.contains(&b));
        assert!(b_node.back_links.contains(&a));
    }

    #[test]
    fn test_triangle_graph() {
        let ids = ["A", "B", "C"]
            .iter()
            .map(|s| NodeId((*s).into()))
            .collect::<Vec<_>>();

        let mut hnsw = HNSW::default();

        for id in &ids {
            hnsw.insert(Node {
                id: id.clone(),
                value: vec![0.0],
                neighbours: BTreeMap::new(),
                back_links: HashSet::new(),
            });
        }

        for id in &ids {
            let node = hnsw.get_node(id).unwrap();
            let n = node.neighbours.get(&LayerIndex(0)).unwrap();
            assert_eq!(n.len(), 2, "each node must have 2 neighbors");

            for other in &ids {
                if other != id {
                    assert!(n.contains(other));
                    assert!(node.back_links.contains(other));
                }
            }
        }
    }

    #[test]
    fn test_search_single_nearest() {
        let mut hnsw = HNSW::default();

        let a = NodeId("A".into());
        let b = NodeId("B".into());

        hnsw.insert(Node {
            id: a.clone(),
            value: vec![0.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        });

        hnsw.insert(Node {
            id: b.clone(),
            value: vec![10.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        });

        let res = hnsw.knn_search(&vec![1.0], 1, Some(10));
        assert_eq!(res[0], a);
    }

    #[test]
    fn test_delete_leaf_node() {
        let mut hnsw = HNSW::default();

        let a = NodeId("A".into());
        let b = NodeId("B".into());

        hnsw.insert(Node {
            id: a.clone(),
            value: vec![0.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        });

        hnsw.insert(Node {
            id: b.clone(),
            value: vec![1.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        });

        hnsw.delete(&b);

        assert!(hnsw.get_node(&b).is_none());

        let a_node = hnsw.get_node(&a).unwrap();
        assert!(a_node.neighbours.values().all(|s| !s.contains(&b)));
        assert!(!a_node.back_links.contains(&b));
    }

    #[test]
    fn test_delete_multi_neighbor_node() {
        let mut hnsw = HNSW::default();

        let ids = ["A", "B", "C", "D"]
            .iter()
            .map(|s| NodeId((*s).into()))
            .collect::<Vec<_>>();

        for id in &ids {
            hnsw.insert(Node {
                id: id.clone(),
                value: vec![0.0],
                neighbours: BTreeMap::new(),
                back_links: HashSet::new(),
            });
        }

        let target = &ids[1]; // delete B
        hnsw.delete(target);

        assert!(hnsw.get_node(target).is_none());

        for id in &ids {
            if id == target {
                continue;
            }
            let node = hnsw.get_node(id).unwrap();
            assert!(node.neighbours.values().all(|s| !s.contains(target)));
            assert!(!node.back_links.contains(target));
        }
    }

    fn assert_hnsw_invariants(hnsw: &HNSW) {
        for (id, node) in &hnsw.nodes {
            // Neighbors must exist
            for neighbours in node.neighbours.values() {
                for n in neighbours {
                    assert!(hnsw.nodes.contains_key(n));
                }
            }

            // Back-links must exist
            for b in &node.back_links {
                assert!(hnsw.nodes.contains_key(b));
            }

            // Bidirectional consistency
            for (lc, neighbours) in &node.neighbours {
                assert!(hnsw.graph.get(lc).unwrap().contains(id));
                for n in neighbours {
                    let other = hnsw.get_node(n).unwrap();
                    assert!(other.back_links.contains(id));
                }
            }
        }
    }
}
