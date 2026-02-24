// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
// similarity function. It then navigates between these localised lists in DFS manner until it
// gets the values it needs to

#![allow(dead_code)]
use crate::{DistanceFn, LinearAlgorithm, error::Error, hnsw::MinHeapQueue};

use super::{LayerIndex, Node, NodeId, OrderedNode};
use crate::heap::BoundedMinHeap;

use std::{
    cmp::{Reverse, min},
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
pub struct HNSW<D: DistanceFn> {
    /// Breadth of search during insertion (efConstruction)
    pub ef_construction: usize,

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

    /// Entry point node ID for search operations.
    ///
    /// This is the node at the highest layer (top_most_layer) and serves as the
    /// starting point for all k-NN searches. Using the top-layer node ensures
    /// efficient hierarchical navigation through the graph.
    ///
    /// Updated whenever a new node is inserted at a higher layer than the current
    /// top_most_layer.
    enter_point: Vec<NodeId>,

    /// distance algorithm
    /// can be ec
    distance_algorithm: D,
}

impl<D: DistanceFn> HNSW<D> {
    pub fn new(
        ef_construction: usize,
        maximum_connections: usize,
        maximum_connections_zero: Option<usize>,
        distance_algorithm: D,
    ) -> Self {
        assert!(maximum_connections > 1, "M must be > 1");
        let maximum_connections_zero = maximum_connections_zero.unwrap_or(maximum_connections * 2);

        Self {
            ef_construction,
            top_most_layer: 0,
            maximum_connections,
            maximum_connections_zero,
            inv_log_m: 1.0 / (maximum_connections as f64).ln(),
            graph: BTreeMap::new(),
            nodes: HashMap::new(),
            enter_point: Vec::with_capacity(1),
            distance_algorithm,
        }
    }

    /// Insert a new element into the HNSW graph
    /// Corresponds to Algorithm 1 (INSERT)
    ///
    /// If a node with the same embedding is added, we silently skip as it doesn't make sense to do
    /// any work but also we shouldn't fail necessarily
    pub fn insert(&mut self, mut value: Node) -> Result<(), Error> {
        if self.nodes.contains_key(&value.id) {
            return Ok(());
        }
        // internally uses SEARCH-LAYER, SELECT-neighbourS
        let inital_ef = 1;

        let mut enter_point = self.enter_point.clone();
        let new_elements_lvl = value.level(self.maximum_connections);

        // NOTE: think of this as finding the best hallway in different floors in a building with
        // muiltiple hallways in a floor...
        // We keep finding the best flow until we get to the `new_elements_lvl+1`
        for level_current in (new_elements_lvl + 1..=self.top_most_layer).rev() {
            let nearest_neighbours = self.search_layer(
                &value,
                &enter_point,
                inital_ef,
                &LayerIndex(level_current as u16),
            )?;

            // NOTE: get the nearest element from W to q
            let nearest_ele = MinHeapQueue::from_nodes(
                nearest_neighbours
                    .iter()
                    .filter_map(|node_id| self.get_node(node_id)),
                &value,
                self.distance_algorithm,
            )
            .pop()
            .map(|ele| ele.0.0)
            .ok_or(Error::NotFoundError(
                "nearest element not found".to_string(),
            ))?;

            enter_point = vec![nearest_ele];
        }

        // Deeper search
        for level_current in (0..=min(self.top_most_layer, new_elements_lvl)).rev() {
            let layer_index = LayerIndex(level_current as u16);

            // NOTE: W = search-layer(q, ep, efConstruction, lc)
            let nearest_neighbours = self
                .search_layer(&value, &enter_point, self.ef_construction, &layer_index)?
                .into_iter()
                .collect();

            // Select M neighbors for the new node at this layer
            // (Algorithm 1: neighbors = SELECT-NEIGHBORS(q, W, M, lc))
            let neighbours = self.select_neighbours_heuristic(
                &value,
                &nearest_neighbours,
                self.maximum_connections, // M not Mmax as the latter is for pruning connections.
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
                    .insert(value.id);

                value
                    .neighbours
                    .entry(layer_index.clone())
                    .or_default()
                    .insert(*neighbour_id);

                // NOTE: invert for backlinks
                value.back_links.insert(*neighbour_id);
                neighbour_node.back_links.insert(value.id);
            }

            //NOTE: insert node to graph
            self.graph
                .entry(layer_index.clone())
                .or_default()
                .insert(value.id);

            // NOTE: for each neighbours prune if each exceeds Mmax
            for neighbour in neighbours.iter() {
                // NOTE: if lc = 0, Mmax = Mmax0
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
                    // Prune neighbor's connections back to Mmax using heuristic selection
                    // (maximum_connections = Mmax0 at layer 0, M at other layers)
                    let new_neighbour_connections = self.select_neighbours_heuristic(
                        neighbour_node,
                        e_conn,
                        maximum_connections, // Already set to the correct Mmax value above
                        &layer_index,
                        false,
                        false,
                    )?;

                    let neighbour_node = self
                        .get_node_mut(neighbour)
                        .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

                    neighbour_node.neighbours.insert(
                        layer_index.clone(),
                        HashSet::from_iter(new_neighbour_connections),
                    );
                }
            }

            // NOTE: Find Best Enter Point
            // Find nearest neighbor from this layer to use as entry point for next layer
            // (Algorithm 1: ep = get_nearest_element_from_W_to_q)
            enter_point = match self.find_best_entry_point(&value, &nearest_neighbours)? {
                None => enter_point,
                Some(new_enter_point) => vec![new_enter_point],
            };
        }

        let value_id = value.id;
        self.nodes.insert(value.id, value);

        // NOTE: given that we use u8 for topmost layer, we want that on first insertion we always
        // set enterpoint else this would be a pain
        //
        if new_elements_lvl > self.top_most_layer || self.nodes.len() == 1 {
            self.top_most_layer = new_elements_lvl;
            self.enter_point = vec![value_id];
        }
        Ok(())
    }

    /// Search for ef nearest neighbours in a specific layer
    /// Corresponds to Algorithm 2 (SEARCH-LAYER)
    pub fn search_layer<'a>(
        &'a self,
        query: &Node,
        entry_points: &'a [NodeId],
        ef: usize,
        layer: &LayerIndex,
    ) -> Result<HashSet<NodeId>, Error> {
        let mut visited_items: HashSet<&NodeId> = HashSet::from_iter(entry_points);

        // C - candidates (min heap via Reverse: smallest distance pops first)
        let mut candidates = MinHeapQueue::from_nodes(
            entry_points.iter().filter_map(|id| self.nodes.get(id)),
            query,
            self.distance_algorithm,
        );

        // W - bounded min heap: keeps ef nearest (smallest distance) neighbors
        let ef_nonzero = NonZeroUsize::new(ef).unwrap_or(NonZeroUsize::new(1).unwrap());
        let mut nearest_neighbours: BoundedMinHeap<OrderedNode> = BoundedMinHeap::new(ef_nonzero);
        for node in entry_points.iter().filter_map(|id| self.nodes.get(id)) {
            let distance = self
                .distance_algorithm
                .distance(node.value.as_slice(), query.value.as_slice());
            nearest_neighbours.push(OrderedNode((node.id, distance)));
        }

        while !candidates.is_empty() {
            let OrderedNode((nearest_id, nearest_dist)) =
                candidates.pop().ok_or(Error::QueueEmpty)?;

            // Check stopping condition: if candidate is further than worst in nearest_neighbours
            if let Some(OrderedNode((_, furthest_dist))) = nearest_neighbours.peek()
                && nearest_dist > *furthest_dist
            {
                break;
            }

            let visited_node = self
                .get_node(&nearest_id)
                .ok_or(Error::NotFoundError("Node not found".to_string()))?;

            // Explore neighbors
            if let Some(visited_node_neighbours) = visited_node.neighbours_at(layer) {
                for neighbour_id in visited_node_neighbours {
                    if visited_items.contains(neighbour_id) {
                        continue;
                    }
                    visited_items.insert(neighbour_id);

                    let neighbour_node = self
                        .get_node(neighbour_id)
                        .ok_or(Error::NotFoundError("Neighbor not found".to_string()))?;

                    let neighbour_dist = self
                        .distance_algorithm
                        .distance(neighbour_node.value.as_slice(), query.value.as_slice());

                    // Add if better than worst in nearest_neighbours OR if we haven't filled ef yet
                    let should_add =
                        if let Some(OrderedNode((_, worst_dist))) = nearest_neighbours.peek() {
                            neighbour_dist < *worst_dist || nearest_neighbours.len() < ef
                        } else {
                            true
                        };

                    if should_add {
                        candidates.push(neighbour_node);
                        nearest_neighbours.push(OrderedNode((neighbour_node.id, neighbour_dist)));
                    }
                }
            }
        }

        Ok(nearest_neighbours
            .iter()
            .map(|OrderedNode((node_id, _))| *node_id)
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
            MinHeapQueue::from_nodes(std::iter::empty(), query, self.distance_algorithm);

        let mut working_queue = MinHeapQueue::from_nodes(
            candidates.iter().filter_map(|id| self.get_node(id)),
            query,
            self.distance_algorithm,
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
            MinHeapQueue::from_nodes(std::iter::empty(), query, self.distance_algorithm);

        // NOTE: if nearest_element_from_w_to_q is closer to q compared to any
        // element in R(use the argmin from R and if nearest_ele_from_w_to_q is closer to q than
        // the argmin then it's assumed it's closer to q than any element in R)
        while !working_queue.is_empty() && response.len() < m {
            let OrderedNode((candidate_id, dist_to_query)) =
                working_queue.pop().ok_or(Error::QueueEmpty)?;

            // NOTE: edge case
            // TODO: loop error handling??
            if response.is_empty() {
                response.push(
                    self.get_node(&candidate_id)
                        .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?,
                );
                continue;
            }

            // Get the candidate node to compute distances to already-selected neighbors
            let candidate_node = self
                .get_node(&candidate_id)
                .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?;
            // Check if candidate is closer to query than to any already-selected neighbor
            let mut is_diverse = true;
            for Reverse(OrderedNode((selected_id, _))) in response.heap.iter() {
                let selected_node = self
                    .get_node(selected_id)
                    .ok_or(Error::NotFoundError("Selected node not found".to_string()))?;

                // Compute distance between candidate and this already-selected neighbor
                let dist_to_selected = self.distance_algorithm.distance(
                    candidate_node.value.as_slice(),
                    selected_node.value.as_slice(),
                );

                // If candidate is closer to this selected neighbour than to query then it means
                // that candidate is clustered within the existing selections so reject it
                if dist_to_selected < dist_to_query {
                    is_diverse = false;
                    break;
                }
            }

            if is_diverse {
                response.push(candidate_node);
            } else {
                discarded_candidates.push(candidate_node);
            }
        }

        if keep_pruned_connections {
            while !discarded_candidates.is_empty() && response.len() < m {
                let OrderedNode((nearest_from_wd_to_q, _)) =
                    discarded_candidates.pop().ok_or(Error::QueueEmpty)?;

                response.push(
                    self.get_node(&nearest_from_wd_to_q)
                        .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?,
                );
            }
        }

        Ok(response
            .heap
            .iter()
            .map(|Reverse(OrderedNode((node_id, _)))| *node_id)
            .collect::<Vec<NodeId>>())
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
        query: &Node,
        k: usize,
        ef: Option<usize>,
    ) -> Result<Vec<NodeId>, Error> {
        let valid_len = NonZeroUsize::new(k).expect("K should be a non zero number");

        let ef = ef.unwrap_or_else(|| k.max(50));
        // Ensure ef >= k as per paper requirements
        let ef = ef.max(k); //.min(255) as u8; 

        let mut enter_point = self.enter_point.clone();
        let ep_level = self.top_most_layer;

        for level_current in (1..=ep_level).rev() {
            let layer = LayerIndex(level_current as u16);

            let searched = self.search_layer(query, &enter_point, 1, &layer)?;

            let ep = MinHeapQueue::from_nodes(
                searched.iter().filter_map(|id| self.get_node(id)),
                query,
                self.distance_algorithm,
            )
            .peek()
            .map(|OrderedNode((node_id, _))| *node_id)
            .ok_or(Error::QueueEmpty)?;
            enter_point = vec![ep];
        }

        let level_zero = self.search_layer(query, &enter_point, ef, &LayerIndex(0))?;
        let mut current_nearest_elements = MinHeapQueue::from_nodes(
            level_zero.iter().filter_map(|id| self.get_node(id)),
            query,
            self.distance_algorithm,
        );

        Ok(current_nearest_elements
            .pop_n(valid_len)
            .into_iter()
            .map(|OrderedNode((node_id, _))| node_id)
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

    // finds the best entry point from candidates
    fn find_best_entry_point(
        &self,
        query: &Node,
        candidates: &HashSet<NodeId>,
    ) -> Result<Option<NodeId>, Error> {
        if candidates.is_empty() {
            // Edge case: no neighbors found at this layer (shouldn't happen in practice)
            // Keep current entry point
            Ok(None)
        } else {
            let enter_point = MinHeapQueue::from_nodes(
                candidates
                    .iter()
                    .filter_map(|node_id| self.get_node(node_id)),
                query,
                self.distance_algorithm,
            )
            .pop()
            .map(|OrderedNode((node_id, _))| node_id)
            .ok_or(Error::NotFoundError(
                "Nearest Element Not Found".to_string(),
            ))?;

            Ok(Some(enter_point))
        }
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

impl Default for HNSW<LinearAlgorithm> {
    fn default() -> Self {
        let maximum_connections = 48;
        let inv_log_m = 1.0 / f64::ln(maximum_connections as f64);

        let distance_algorithm = LinearAlgorithm::EuclideanDistance;

        Self {
            ef_construction: 100,
            top_most_layer: 0,
            maximum_connections,
            maximum_connections_zero: 100,
            inv_log_m, // ln(1/M)
            graph: BTreeMap::new(),
            nodes: HashMap::new(),
            enter_point: Vec::with_capacity(1),
            distance_algorithm,
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
                LinearAlgorithm::EuclideanDistance
                    .distance(n.value.as_slice(), query.value.as_slice()),
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

    #[test]
    fn test_simple_hnsw_state() {
        let node = Node::new(vec![3.2]);
        let node_id = *node.id();

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
        let node_a = Node::new(vec![0.0]);
        let a = *node_a.id();

        let node_b = Node::new(vec![1.0]);
        let b = *node_b.id();

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
                value: EmbeddingKey::new(vec![0.0]),
                neighbours: BTreeMap::new(),
                back_links: HashSet::new(),
            })
            .unwrap();
        }

        let layer0 = LayerIndex(0);

        for (id, node) in &hnsw.nodes {
            // Degree bound
            if let Some(neighbours) = node.neighbours.get(&layer0) {
                assert!(neighbours.len() <= hnsw.maximum_connections_zero);

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
            value: EmbeddingKey::new(vec![0.0]),
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.insert(Node {
            id: b.clone(),
            value: EmbeddingKey::new(vec![10.0]),
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        let node_id = NodeId(99);
        let query_node = Node {
            id: node_id.clone(),
            value: EmbeddingKey::new(vec![1.0]),
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
            value: EmbeddingKey::new(vec![0.0]),
            neighbours: BTreeMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.insert(Node {
            id: b.clone(),
            value: EmbeddingKey::new(vec![1.0]),
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
                value: EmbeddingKey::new(vec![0.0]),
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

    fn assert_hnsw_invariants<D: DistanceFn>(hnsw: &HNSW<D>) {
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

    #[test]
    fn test_level_assignment_is_deterministic() {
        // Same embedding should always produce same level
        let embedding = vec![1.0, 2.0, 3.0];

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
            let embedding = vec![i as f32, (i * 2) as f32, (i * 3) as f32];
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
        let embedding = vec![5.5, 10.2, 3.7];
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
        let node1 = Node::new(vec![1.0, 2.0, 3.0]);
        let node2 = Node::new(vec![4.0, 5.0, 6.0]);
        let node3 = Node::new(vec![7.0, 8.0, 9.0]);

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
