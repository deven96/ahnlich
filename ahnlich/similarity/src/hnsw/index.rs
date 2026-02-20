// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
// similarity function. It then navigates between these localised lists in DFS manner until it
// gets the values it needs to

#![allow(dead_code)]
use crate::{
    error::Error,
    hnsw::{MaxHeapQueue, MinHeapQueue},
};
use rand::Rng;

use super::{LayerIndex, Node, NodeId, euclidean_distance_comp};

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
#[derive(Debug)]
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
    enter_point: Vec<NodeId>,
}

impl HNSW {
    /// Returns a random level for a given element.
    /// TODO: element hashed node id would be used to determine an elements level.
    /// We have to make this determinable that a node would always return a certain level all the
    /// time. Would there be any issues with this??
    ///l = floor(-ln(uniform(0,1)) * mL)
    fn get_element_level(&self) -> u8 {
        let mut rng = rand::thread_rng();
        let unif: f64 = rng.r#gen(); // uniform hopefully
        (-unif.ln() * self.inv_log_m).floor() as u8
    }

    /// Insert a new element into the HNSW graph
    /// Corresponds to Algorithm 1 (INSERT)
    pub fn insert(&mut self, mut value: Node) -> Result<(), Error> {
        // TODO: Research what happens if we try inserting a value multiple times

        // internally uses SEARCH-LAYER, SELECT-neighbourS
        let inital_ef = 1;

        let ef_construction = self.ef_construction.unwrap_or(100);
        let mut enter_point = self.enter_point.clone();
        let new_elements_lvl = self.get_element_level();

        // NOTE: think of this as finding the best hallway in different floors in a building with
        // muiltiple hallways in a floor...
        // We keep finding the best flow until we get to the `new_elements_lvl+1`
        for level_current in (new_elements_lvl + 1..=self.top_most_layer).rev() {
            let enter_points = self.enter_point.clone();

            let nearest_neighbours = self.search_layer(
                &value,
                &enter_points,
                inital_ef,
                &LayerIndex(level_current as u16),
            )?;

            // NOTE: get the nearest element from W to q
            let nearest_ele = MinHeapQueue::from_nodes(
                nearest_neighbours
                    .iter()
                    .filter_map(|node_id| self.get_node(node_id)),
                &value,
                euclidean_distance_comp,
            )
            .pop()
            .map(|ele| ele.0.0.0.clone())
            .ok_or(Error::NotFoundError(
                "nearest element not found".to_string(),
            ))?;

            enter_point = vec![nearest_ele];
        }

        // Deeper search
        for level_current in (0..=min(self.top_most_layer, new_elements_lvl)).rev() {
            let layer_index = LayerIndex(level_current as u16);

            //NOTE: insert node to graph
            self.graph
                .entry(layer_index.clone())
                .or_default()
                .insert(value.id.clone());

            // TODO: error handling in loop??
            // NOTE: W = search-layer(q, ep, efConstruction, lc)
            let nearest_neighbours = self
                .search_layer(&value, &enter_point, ef_construction, &layer_index)?
                .into_iter()
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
            )?;

            // NOTE: add bidirectional connections from neighbours to q at layer lc
            for neighbour_id in neighbours.iter() {
                // TODO: shouldn't return here, we can handle this and move on with the loop,
                // could be a node marked for deletion??
                let neighbour_node = self
                    .get_node_mut(neighbour_id)
                    .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

                neighbour_node
                    .neighbours
                    .entry(layer_index.clone())
                    .or_default()
                    .insert(value.id.clone());

                value
                    .neighbours
                    .entry(layer_index.clone())
                    .or_default()
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
                };

                // TODO: shouldn't return here, we can handle this and move on with the loop,
                // could be a node marked for deletion??
                let neighbour_node = self
                    .get_node(neighbour)
                    .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

                let e_conn = neighbour_node
                    .neighbours_at(&layer_index)
                    .ok_or(Error::NotFoundError("Index not found".to_string()))?;

                if e_conn.len() > maximum_connections {
                    let new_neighbour_connections = self.select_neighbours_heuristic(
                        neighbour_node,
                        e_conn,
                        // TODO: should be Mmax, so we need to know if M and Mmax are diff
                        maximum_connections,
                        &layer_index,
                        false,
                        false,
                    )?;

                    // TODO: same here with looped error
                    let neighbour_node = self
                        .get_node_mut(neighbour)
                        .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

                    neighbour_node
                        .neighbours
                        .entry(layer_index.clone())
                        .insert_entry(HashSet::from_iter(new_neighbour_connections));
                }
            }

            // NOTE: Find Best Enter Point
            // TODO: fix this line here
            enter_point = {
                if self.nodes.len() <= 1 {
                    vec![]
                } else {
                    let enter_point = MinHeapQueue::from_nodes(
                        nearest_neighbours
                            .iter()
                            .filter_map(|node_id| self.get_node(node_id)),
                        &value,
                        euclidean_distance_comp,
                    )
                    .pop()
                    .map(|ele| ele.0.0.0.clone())
                    .ok_or(Error::NotFoundError(
                        "Nearest Element Not Found".to_string(),
                    ))?;

                    vec![enter_point]
                }
            };
        }

        self.nodes.insert(value.id.clone(), value.clone());

        // NOTE: given that we use u8 for topmost layer, we want that on first insertion we always
        // set enterpoint else this would be a pain
        //
        // TODO: should topmost layer of empty hnsw be -1 ??
        if new_elements_lvl > self.top_most_layer || self.nodes.len() == 1 {
            self.top_most_layer = new_elements_lvl;
            self.enter_point = vec![value.id.clone()];
        }
        Ok(())
    }

    /// Search for ef nearest neighbours in a specific layer
    /// Corresponds to Algorithm 2 (SEARCH-LAYER)
    pub fn search_layer<'a>(
        &'a self,
        query: &Node,
        entry_points: &'a [NodeId],
        ef: u8,
        layer: &LayerIndex,
    ) -> Result<HashSet<NodeId>, Error> {
        //
        // v
        let mut visited_items: HashSet<&NodeId> = HashSet::from_iter(entry_points);

        // C
        let mut candidates = MinHeapQueue::from_nodes(
            entry_points.iter().filter_map(|id| self.nodes.get(id)),
            query,
            euclidean_distance_comp,
        );

        // W
        let mut nearest_neighbours = MaxHeapQueue::from_nodes(
            entry_points.iter().filter_map(|id| self.nodes.get(id)),
            query,
            euclidean_distance_comp,
        );

        while candidates.len() > 0 {
            let nearest_ele_from_c_and_to_q = candidates
                .pop()
                .map(|ele| ele.0.0.0)
                .ok_or(Error::QueueEmpty)?;

            let mut furthest_ele_from_nearest_neighbours_to_q = nearest_neighbours
                .peak()
                .map(|ele| ele.0.0.clone())
                .ok_or(Error::NotFoundError("Peaked Node not found".to_string()))?;

            let closest_node = self
                .get_node(&nearest_ele_from_c_and_to_q)
                .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

            let mut furthest_node = self
                .get_node(&furthest_ele_from_nearest_neighbours_to_q)
                .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

            let furthest_distance = euclidean_distance_comp(&furthest_node.value, &query.value);

            if euclidean_distance_comp(&query.value, &closest_node.value) > furthest_distance {
                break;
            }

            let visited_node = self
                .get_node(&nearest_ele_from_c_and_to_q)
                .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

            // NOTE:this would return none in alot of instances and we can't end or break the
            // processing so we have to handle it...
            if let Some(visited_node_neighbours) = visited_node.neighbours_at(layer) {
                for e in visited_node_neighbours {
                    if visited_items.contains(e) {
                        continue;
                    }
                    visited_items.insert(e);

                    furthest_ele_from_nearest_neighbours_to_q =
                        nearest_neighbours.peak().map(|ele| ele.0.0.clone()).ok_or(
                            Error::NotFoundError(" Peaked Element not Found".to_string()),
                        )?;

                    furthest_node = self
                        .get_node(&furthest_ele_from_nearest_neighbours_to_q)
                        .ok_or(Error::NotFoundError(" Node Ref not Found".to_string()))?;

                    let neighbour_node = self
                        .get_node(e)
                        .ok_or(Error::NotFoundError(" Node Ref not Found".to_string()))?;

                    if (euclidean_distance_comp(&neighbour_node.value, &query.value)
                        < euclidean_distance_comp(&furthest_node.value, &query.value))
                        || (nearest_neighbours.len() < ef as usize)
                    {
                        candidates.push(neighbour_node);
                        nearest_neighbours.push(neighbour_node);

                        if nearest_neighbours.len() > ef as usize {
                            // NOTE: remove the furthest node from W(nearest_neighbours) to Q
                            nearest_neighbours.pop().ok_or(Error::QueueEmpty)?;
                        }
                    }
                }
            }
        }

        Ok(nearest_neighbours
            .heap
            .iter()
            .map(|ordered| ordered.0.0.clone())
            .collect())
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
    ) -> Result<Vec<NodeId>, Error> {
        let mut response =
            MinHeapQueue::from_nodes(std::iter::empty(), query, euclidean_distance_comp);

        let mut working_queue = MinHeapQueue::from_nodes(
            candidates.iter().filter_map(|id| self.get_node(id)),
            query,
            euclidean_distance_comp,
        );

        if extend_candidates {
            for candidate in candidates {
                // TODO: loop error handling??

                let candidate_node = self
                    .get_node(candidate)
                    .ok_or(Error::NotFoundError(" Node Ref not Found".to_string()))?;

                for neighbour_id in candidate_node
                    .neighbours_at(layer)
                    .ok_or(Error::NotFoundError(format!("{:?} not Found", layer)))?
                {
                    if !working_queue.contains(neighbour_id) {
                        let neighbour_node = self
                            .get_node(neighbour_id)
                            .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?;
                        working_queue.push(neighbour_node);
                    }
                }
            }
        }

        let mut discarded_candidates =
            MinHeapQueue::from_nodes(std::iter::empty(), query, euclidean_distance_comp);

        // NOTE: if nearest_element_from_w_to_q is closer to q compared to any
        // element in R(use the argmin from R and if nearest_ele_from_w_to_q is closer to q than
        // the argmin then it's assumed it's closer to q than any element in R)
        while working_queue.len() > 0 && response.len() < m {
            let nearest_ele_from_w_to_q = working_queue.pop().ok_or(Error::QueueEmpty)?;

            // NOTE: edge case
            // TODO: loop error handling??
            if response.len() == 0 {
                let node_id = &(nearest_ele_from_w_to_q.0.0.0);
                response.push(
                    self.get_node(node_id)
                        .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?,
                );
                continue;
            }

            let arg_min_res = response.peak().ok_or(Error::QueueEmpty)?;

            if (nearest_ele_from_w_to_q.0.0.1) < (arg_min_res.0.0.1) {
                let node_id = &(nearest_ele_from_w_to_q.0.0.0);
                response.push(
                    self.get_node(node_id)
                        .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?,
                );
            } else {
                let node_id = &(nearest_ele_from_w_to_q.0.0.0);
                discarded_candidates.push(
                    self.get_node(node_id)
                        .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?,
                );
            }
        }

        if keep_pruned_connections {
            while discarded_candidates.len() > 0 && response.len() < m {
                let nearest_from_wd_to_q = discarded_candidates.pop().ok_or(Error::QueueEmpty)?;

                let node_id = &(nearest_from_wd_to_q.0.0.0);
                response.push(
                    self.get_node(node_id)
                        .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?,
                );
            }
        }

        Ok(response
            .heap
            .iter()
            .map(|node| node.0.0.0.clone())
            .collect::<Vec<NodeId>>())
    }

    /// K-Nearest neighbour Search
    /// Corresponds to Algorithm 5 (K-NN-SEARCH)
    pub fn knn_search(&self, query: &Node, k: usize, ef: Option<u8>) -> Result<Vec<NodeId>, Error> {
        let valid_len = NonZeroUsize::new(k).expect("K should be a non zero number");

        // TODO: fix this wierdness
        let ef = ef.unwrap_or(self.ef_construction.unwrap_or(100));

        let mut enter_point = self.enter_point.clone();
        let ep_level = self.top_most_layer;

        for level_current in (1..=ep_level).rev() {
            let layer = LayerIndex(level_current as u16);

            let searched = self.search_layer(query, &enter_point, 1, &layer)?;

            let ep = MinHeapQueue::from_nodes(
                searched.iter().filter_map(|id| self.get_node(id)),
                query,
                euclidean_distance_comp,
            )
            .peak()
            .map(|ele| ele.0.0.0.clone())
            .ok_or(Error::QueueEmpty)?;
            enter_point = vec![ep];
        }

        let level_zero = self.search_layer(query, &enter_point, ef, &LayerIndex(0))?;
        let mut current_nearest_elements = MinHeapQueue::from_nodes(
            level_zero.iter().filter_map(|id| self.get_node(id)),
            query,
            euclidean_distance_comp,
        );

        Ok(current_nearest_elements
            .pop_n(valid_len)
            .into_iter()
            .map(|id| id.0.0.0.clone())
            .collect())
    }

    /// delete an new element from HNSW graph
    pub fn delete(&mut self, node_id: &NodeId) {
        let (backlinks, neighbour_keys) = {
            let node = self.get_node(node_id).unwrap();
            (
                node.back_links.clone(),
                node.neighbours.keys().cloned().collect::<Vec<_>>(),
            )
        };

        for backlink in &backlinks {
            let related_node = self.get_node_mut(backlink).unwrap();
            let neighbour_keys = related_node.neighbours.keys().cloned().collect::<Vec<_>>();
            for layer_index in neighbour_keys {
                related_node
                    .neighbours
                    .entry(layer_index)
                    .and_modify(|set| {
                        set.remove(node_id);
                    });
            }

            related_node.back_links.remove(node_id);
        }

        for layer_index in neighbour_keys.iter() {
            self.graph.entry(layer_index.clone()).and_modify(|set| {
                set.remove(node_id);
            });
        }

        self.nodes.remove(node_id);
    }

    /// Optional helper to get a node by NodeId efficiently
    pub fn get_node(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }

    /// Optional helper to get a node by NodeId efficiently
    pub fn get_node_mut(&mut self, id: &NodeId) -> Option<&mut Node> {
        self.nodes.get_mut(id)
    }
}

impl Default for HNSW {
    fn default() -> Self {
        let maximum_connections = 48;
        let inv_log_m = 1.0 / f64::ln(maximum_connections as f64);

        Self {
            ef_construction: Some(100),
            top_most_layer: 0,
            maximum_connections,
            maximum_connections_zero: 100,
            inv_log_m, // ln(1/M)
            graph: BTreeMap::new(),
            nodes: HashMap::new(),
            enter_point: Vec::with_capacity(1),
        }
    }
}

#[cfg(test)]
pub fn brute_knn(query: &Node, data: &[Node], k: usize) -> Vec<(NodeId, f32)> {
    use itertools::Itertools;

    debug_assert!(k <= data.len());

    data.iter()
        .map(|n| {
            (
                n.id.clone(),
                euclidean_distance_comp(&n.value, &query.value),
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

    #[test]
    fn test_simple_hnsw_state() {
        let node_id = NodeId(1);
        let node = Node {
            id: node_id.clone(),
            value: vec![3.2],
            back_links: HashSet::new(),
            neighbours: BTreeMap::new(),
        };

        let mut hnsw = HNSW::default();
        hnsw.insert(node).unwrap();
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
        let a = NodeId(10);
        let b = NodeId(20);

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
        hnsw.insert(node_a).unwrap();
        hnsw.insert(node_b).unwrap();

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
    fn test_hnsw_basic_invariants() {
        let ids = [NodeId(1), NodeId(2), NodeId(3)].to_vec();

        let mut hnsw = HNSW::default();

        for id in &ids {
            hnsw.insert(Node {
                id: id.clone(),
                value: vec![0.0],
                neighbours: BTreeMap::new(),
                back_links: HashSet::new(),
            })
            .unwrap();
        }

        let layer0 = LayerIndex(0);

        for (id, node) in &hnsw.nodes {
            // Degree bound
            if let Some(neighbours) = node.neighbours.get(&layer0) {
                assert!(neighbours.len() <= hnsw.maximum_connections_zero as usize);

                for n in neighbours {
                    // Neighbour exists
                    assert!(hnsw.nodes.contains_key(n));

                    // Backlink exists
                    let other = hnsw.get_node(n).unwrap();
                    assert!(
                        other.back_links.contains(id),
                        "missing backlink: {:?} <- {:?}",
                        n,
                        id
                    );
                }
            }
        }
    }

    #[test]
    fn test_search_single_nearest() {
        let mut hnsw = HNSW::default();

        let a = NodeId(10);
        let b = NodeId(20);

        hnsw.insert(Node {
            id: a.clone(),
            value: vec![0.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.insert(Node {
            id: b.clone(),
            value: vec![10.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        let node_id = NodeId(99);
        let query_node = Node {
            id: node_id.clone(),
            value: vec![1.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        };

        let res = hnsw.knn_search(&query_node, 1, Some(10)).unwrap();
        assert_eq!(res[0], a);
    }

    #[test]
    fn test_delete_leaf_node() {
        let mut hnsw = HNSW::default();

        let a = NodeId(10);
        let b = NodeId(20);

        hnsw.insert(Node {
            id: a.clone(),
            value: vec![0.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.insert(Node {
            id: b.clone(),
            value: vec![1.0],
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.delete(&b);

        assert!(hnsw.get_node(&b).is_none());

        let a_node = hnsw.get_node(&a).unwrap();
        assert!(a_node.neighbours.values().all(|s| !s.contains(&b)));
        assert!(!a_node.back_links.contains(&b));
    }

    #[test]
    fn test_delete_multi_neighbour_node() {
        let mut hnsw = HNSW::default();

        let ids = [NodeId(10), NodeId(20), NodeId(30), NodeId(40)].to_vec();

        for id in &ids {
            hnsw.insert(Node {
                id: id.clone(),
                value: vec![0.0],
                neighbours: BTreeMap::new(),
                back_links: HashSet::new(),
            })
            .unwrap();
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
