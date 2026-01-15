#![allow(dead_code)]
use crate::hnsw::{MaxHeapQueue, MinHeapQueue};

pub use super::{LayerIndex, Node, NodeId};
/// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
/// similarity function. It then navigates between these localised lists in DFS manner until it
/// gets the values it needs to
use std::{
    cmp::min,
    collections::{HashMap, HashSet, btree_map::BTreeMap},
    num::NonZeroUsize,
};

/// HNSW represents a Hierarchical Navigable Small World graph.
///
/// The graph is organized into multiple layers. Each layer contains a set of node IDs,
/// and each node holds its neighbours per layer along with its embedding vector.
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
///    - Remove the node ID from all neighbours of other nodes (using back-links/referrals).
///      This ensures no stale references remain in the graph.
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
#[derive(Default)]
pub struct HNSW {
    /// Breadth of search during insertion (efConstruction)
    pub ef_construction: Option<u8>,

    /// Top-most layer index in the graph (L)
    pub top_most_layer: u8,

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
    graph: BTreeMap<LayerIndex, HashSet<NodeId>>,

    /// All nodes in the HNSW
    ///
    /// The single source of truth for all node data.
    /// Keys are NodeId, values are the Node structs containing embeddings and neighbours.
    nodes: HashMap<NodeId, Node>,

    /// node id of the starting point usually the element at the top most layer
    /// TODO: should this be saved or randomly selected since we know the topmost layer
    enter_point: NodeId,
}

fn similarity_function(first: &[f32], second: &[f32]) -> f32 {
    todo!()
}

impl HNSW {
    /// Returns a random level for a given element.
    /// TODO: element hashed node id would be used to determine an elements level.
    /// We have to make this determinable that a node would always return a certain level all the
    /// time. Would there be any issues with this??
    fn get_element_level(&self) -> u8 {
        todo!()
    }

    /// Insert a new element into the HNSW graph
    /// Corresponds to Algorithm 1 (INSERT)
    pub fn insert(&mut self, mut value: Node) {
        // internally uses SEARCH-LAYER, SELECT-neighbourS
        let inital_ef = 1;

        let mut enter_point = self.enter_point.clone();
        let new_elements_lvl = self.get_element_level();

        // NOTE: think of this as finding the best hallway in different floors in a building with
        // muiltiple hallways in a floor...
        // We keep finding the best flow until we get to the `new_elements_lvl+1`
        for level_current in (new_elements_lvl + 1..=self.top_most_layer).rev() {
            let enter_points = &[enter_point.clone()];

            let nearest_neighbours = self.search_layer(
                &value,
                enter_points,
                inital_ef,
                &LayerIndex(level_current as u16),
            );

            let nearest_neighbours_nodes = nearest_neighbours
                .iter()
                .filter_map(|node_id| self.get_node(node_id).cloned())
                .collect::<Vec<Node>>();

            // NOTE: get the nearest element from W to q
            let enter_points =
                MinHeapQueue::from_nodes(&nearest_neighbours_nodes, &value, similarity_function)
                    .pop_n(NonZeroUsize::new(1).unwrap());

            assert!(enter_points.len() == 1);

            let tmp_enterpoint = enter_points.first().unwrap();
            enter_point = tmp_enterpoint.0.0.0.clone();
        }

        // Deeper search
        for level_current in (0..min(self.top_most_layer, new_elements_lvl)).rev() {
            let layer_index = LayerIndex(level_current as u16);

            //NOTE: insert node to graph
            self.graph
                .entry(layer_index.clone())
                .or_insert_with(HashSet::new)
                .insert(value.id.clone());

            // NOTE: W = search-layer(q, ep, efConstruction, lc)
            let nearest_neighbours = self
                .search_layer(
                    &value,
                    &[enter_point.clone()],
                    self.ef_construction.unwrap(),
                    &layer_index,
                )
                .into_iter()
                .map(|d| d.clone())
                .collect();

            // NOTE: neighbours =  SELECT-neighbourS(q, W, M, lc)
            let neighbours = self.select_neighbours_heuristic(
                &value,
                &nearest_neighbours,
                // TODO: is M and Mmax same here?
                self.maximum_connections,
                &layer_index,
                false,
                false,
            );

            // NOTE: add bidirectional connections from neighbours to q at layer lc
            for neighbour_id in neighbours.iter() {
                let neighbour_node = self.get_node_mut(neighbour_id).unwrap();
                neighbour_node
                    .neighbours
                    .entry(layer_index.clone())
                    .or_insert_with(HashSet::new)
                    .insert(value.id.clone());

                value
                    .neighbours
                    .entry(layer_index.clone())
                    .or_insert_with(HashSet::new)
                    .insert(neighbour_id.clone());

                // NOTE: invert for backlinks
                value.back_links.insert(neighbour_id.clone());
                neighbour_node.back_links.insert(value.id.clone());
            }

            // NOTE: for each neighbours prune if each exceeds Mmax
            for neighbour in neighbours.iter() {
                // NOTE: if lc = 0, mmax = mmax0
                let maximum_connections = if level_current == 0 {
                    self.maximum_connections_zero
                } else {
                    self.maximum_connections
                } as usize;

                let neighbour_node = self.get_node_mut(neighbour).unwrap();

                let e_conn = neighbour_node.neighbours_at(&layer_index).unwrap();

                if e_conn.len() > maximum_connections {
                    let new_neighbour_connections = self.select_neighbours_heuristic(
                        &neighbour_node,
                        &e_conn,
                        // TODO: should be Mmax, so we need to know if M and Mmax are diff
                        maximum_connections,
                        &layer_index,
                        false,
                        false,
                    );

                    neighbour_node
                        .neighbours
                        .entry(layer_index.clone())
                        .insert_entry(HashSet::from_iter(new_neighbour_connections));
                }
            }

            // NOTE: Find Best Enter Point
            enter_point = {
                let nearest_neighbours_nodes = nearest_neighbours
                    .iter()
                    .filter_map(|node_id| self.get_node(node_id).cloned())
                    .collect::<Vec<Node>>();

                let enter_points = MinHeapQueue::from_nodes(
                    &nearest_neighbours_nodes,
                    &value,
                    similarity_function,
                )
                .pop_n(NonZeroUsize::new(1).unwrap());

                assert!(enter_points.len() == 1);

                let tmp_enterpoint = enter_points.first().unwrap();
                tmp_enterpoint.0.0.0.clone()
            };
        }

        if new_elements_lvl > self.top_most_layer {
            self.top_most_layer = new_elements_lvl;
            self.enter_point = value.id.clone()
        }

        self.nodes.insert(value.id.clone(), value);
    }

    /// Search for ef nearest neighbours in a specific layer
    /// Corresponds to Algorithm 2 (SEARCH-LAYER)
    pub fn search_layer<'a>(
        &'a self,
        query: &Node,
        entry_points: &'a [NodeId],
        ef: u8,
        layer: &LayerIndex,
    ) -> HashSet<NodeId> {
        //
        let nodes = entry_points
            .iter()
            .filter_map(|id| self.nodes.get(id))
            .map(|node| node.clone())
            .collect::<Vec<Node>>();
        // v
        let mut visited_items: HashSet<&NodeId> = HashSet::from_iter(entry_points);
        // C

        let mut candidates = MinHeapQueue::from_nodes(&nodes, query, similarity_function);
        // W
        let mut nearest_neighbours = MaxHeapQueue::from_nodes(&nodes, query, similarity_function);

        while candidates.len() > 0 {
            let nearest_ele_from_c_and_to_q = candidates.pop().unwrap().0.0.0;
            let mut furthest_ele_from_nearest_neighbours_to_q =
                nearest_neighbours.peak().unwrap().0.0.clone();

            let closest_node = self.nodes.get(&nearest_ele_from_c_and_to_q).unwrap();
            let mut furthest_node = self
                .nodes
                .get(&furthest_ele_from_nearest_neighbours_to_q)
                .unwrap();

            let furthest_distance = similarity_function(&furthest_node.value, &query.value);

            if similarity_function(&query.value, &closest_node.value) > furthest_distance {
                break;
            }

            let visited_node = self.get_node(&nearest_ele_from_c_and_to_q).unwrap();

            for e in visited_node.neighbours_at(layer).unwrap() {
                if visited_items.contains(e) {
                    continue;
                }
                visited_items.insert(e);

                furthest_ele_from_nearest_neighbours_to_q =
                    nearest_neighbours.peak().unwrap().0.0.clone();

                furthest_node = self
                    .nodes
                    .get(&furthest_ele_from_nearest_neighbours_to_q)
                    .unwrap();

                let neighbour_node = self.get_node(e).unwrap();

                if (similarity_function(&neighbour_node.value, &query.value)
                    < similarity_function(&furthest_node.value, &query.value))
                    || (nearest_neighbours.len() < ef as usize)
                {
                    candidates.push(neighbour_node);
                    nearest_neighbours.push(neighbour_node);

                    if nearest_neighbours.len() > ef as usize {
                        // NOTE: remove the furthest node from W(nearest_neighbours) to Q
                        nearest_neighbours.pop().unwrap();
                    }
                }
            }
        }

        return nearest_neighbours
            .heap
            .iter()
            .map(|ordered| ordered.0.0.clone())
            .collect();
    }

    /// Select M neighbours using heuristic for diversity and pruning
    /// Corresponds to Algorithm 4 (SELECT-neighbourS-HEURISTIC)
    pub fn select_neighbours_heuristic(
        &self,
        query: &Node,
        candidates: &HashSet<NodeId>,
        m: usize,
        layer: &LayerIndex,
        extend_candidates: bool,
        keep_pruned_connections: bool,
    ) -> Vec<NodeId> {
        let mut response = MinHeapQueue::from_nodes(&vec![], query, similarity_function);

        let nodes = candidates
            .iter()
            .filter_map(|id| self.get_node(id).cloned())
            .collect::<Vec<_>>();
        let mut working_queue = MinHeapQueue::from_nodes(&nodes, query, similarity_function);

        if extend_candidates {
            for candidate in candidates {
                let candidate_node = self.get_node(candidate).unwrap();
                for neighbour_id in candidate_node.neighbours_at(layer).unwrap() {
                    if !working_queue.contains(neighbour_id) {
                        let neighbour_node = self.get_node(neighbour_id).unwrap();
                        working_queue.push(&neighbour_node);
                    }
                }
            }
        }

        let mut discarded_candidates =
            MinHeapQueue::from_nodes(&vec![], query, similarity_function);

        // NOTE: if nearest_element_from_w_to_q is closer to q compared to any
        // element in R(use the argmin from R and if nearest_ele_from_w_to_q is closer to q than
        // the argmin then it's assumed it's closer to q than any element in R)
        while working_queue.len() > 0 && response.len() < m {
            let nearest_ele_from_w_to_q = working_queue.pop().unwrap();

            // NOTE: edge case
            if response.len() == 0 {
                let node_id = &(nearest_ele_from_w_to_q.0.0.0);
                response.push(self.get_node(node_id).unwrap());
                continue;
            }

            let arg_min_res = response.peak().unwrap();

            if (nearest_ele_from_w_to_q.0.0.1) < (arg_min_res.0.0.1) {
                let node_id = &(nearest_ele_from_w_to_q.0.0.0);
                response.push(self.get_node(node_id).unwrap());
            } else {
                let node_id = &(nearest_ele_from_w_to_q.0.0.0);
                discarded_candidates.push(self.get_node(node_id).unwrap());
            }
        }

        if keep_pruned_connections {
            while discarded_candidates.len() > 0 && response.len() < m {
                let nearest_from_wd_to_q = discarded_candidates.pop().unwrap();

                let node_id = &(nearest_from_wd_to_q.0.0.0);
                response.push(self.get_node(node_id).unwrap());
            }
        }

        response
            .heap
            .iter()
            .map(|node| node.0.0.0.clone())
            .collect::<Vec<NodeId>>()
    }

    /// K-Nearest neighbour Search
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

    /// Optional helper to get a node by NodeId efficiently
    pub fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        todo!()
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

        // Layer 0 neighbours
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
            assert_eq!(n.len(), 2, "each node must have 2 neighbours");

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
    fn test_delete_multi_neighbour_node() {
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
            // neighbours must exist
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
