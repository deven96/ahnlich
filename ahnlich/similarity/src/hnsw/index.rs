// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
// similarity function. It then navigates between these localised lists in DFS manner until it
// gets the values it needs to

#![allow(dead_code)]
use crate::{
    DistanceFn, LinearAlgorithm,
    error::Error,
    hnsw::{HNSWConfig, MinHeapQueue},
};

use super::{LayerIndex, Node, NodeId, NodeIdHashSet, OrderedNode, get_node_id};
use crate::EmbeddingKey;
use crate::heap::BoundedMinHeap;

use papaya::HashSet;
use parking_lot::RwLock;
use smallvec::{SmallVec, smallvec};
use std::{
    cmp::{Reverse, min},
    num::NonZeroUsize,
    sync::atomic::{AtomicU8, Ordering},
};

/// HNSW represents a Hierarchical Navigable Small World graph.
///
/// The graph is organized into multiple layers. Each layer contains a set of node IDs,
/// and each node holds its neighbours per layer along with its embedding vector.
/// This separation allows efficient lookups, prevents duplicate nodes per layer,
/// and supports deletion operations.
///
/// Uses papaya's concurrent HashMap for lock-free concurrent read access to nodes
/// and graph layers. The enter_point is protected by a parking_lot RwLock, and
/// top_most_layer uses an AtomicU8 for lock-free reads.
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
pub struct HNSW<D: DistanceFn> {
    /// Breadth of search during insertion (efConstruction)
    pub ef_construction: usize,

    /// Top-most layer index in the graph (L)
    top_most_layer: AtomicU8,

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
    nodes: papaya::HashMap<NodeId, Node>,

    /// Entry point node ID for search operations.
    ///
    /// This is the node at the highest layer (top_most_layer) and serves as the
    /// starting point for all k-NN searches. Using the top-layer node ensures
    /// efficient hierarchical navigation through the graph.
    ///
    /// Updated whenever a new node is inserted at a higher layer than the current
    /// top_most_layer. Protected by a RwLock for safe concurrent access.
    enter_point: RwLock<SmallVec<[NodeId; 1]>>,

    /// distance algorithm
    /// can be ec
    distance_algorithm: D,

    /// Keep pruned connections:
    keep_pruned_connections: bool,

    /// extend candidates:
    extend_candidates: bool,
}

impl<D: DistanceFn> HNSW<D> {
    pub fn new(distance_algorithm: D) -> Self {
        let config = HNSWConfig::default();
        Self::new_with_config(config, distance_algorithm)
    }

    pub fn new_with_config(config: HNSWConfig, distance_algorithm: D) -> Self {
        assert!(config.maximum_connections > 1, "M must be > 1");

        Self {
            ef_construction: config.ef_construction,
            top_most_layer: AtomicU8::new(0),
            maximum_connections: config.maximum_connections,
            maximum_connections_zero: config.maximum_connections_zero,
            inv_log_m: 1.0 / (config.maximum_connections as f64).ln(),
            graph: papaya::HashMap::new(),
            nodes: papaya::HashMap::new(),
            enter_point: RwLock::new(SmallVec::new()),
            distance_algorithm,
            keep_pruned_connections: config.keep_pruned_connections,
            extend_candidates: config.extend_candidates,
        }
    }

    /// Get the current top-most layer index
    pub fn top_layer(&self) -> u8 {
        self.top_most_layer.load(Ordering::Acquire)
    }

    /// Batch insert new embeddings into the HNSW graph.
    /// Matches the NonLinearAlgorithmWithIndexImpl::insert signature.
    pub fn insert(&self, new: &[EmbeddingKey]) -> Result<(), Error> {
        if new.is_empty() {
            return Ok(());
        }
        for key in new {
            let node = Node::new(key.clone());
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

    /// Insert a new element into the HNSW graph
    /// Corresponds to Algorithm 1 (INSERT)
    ///
    /// If a node with the same embedding is added, we silently skip as it doesn't make sense to do
    /// any work but also we shouldn't fail necessarily
    ///
    pub fn insert_node(&self, value: Node) -> Result<(), Error> {
        let nodes = self.nodes.pin();
        let graph = self.graph.pin();
        let top_layer = self.top_most_layer.load(Ordering::Acquire);

        if nodes.contains_key(&value.id) {
            return Ok(());
        }
        // internally uses SEARCH-LAYER, SELECT-neighbourS
        let inital_ef = 1;

        let mut enter_point = self.enter_point.read().clone();
        let new_elements_lvl = value.level(self.maximum_connections);

        // NOTE: think of this as finding the best hallway in different floors in a building with
        // muiltiple hallways in a floor...
        // We keep finding the best flow until we get to the `new_elements_lvl+1`
        for level_current in (new_elements_lvl + 1..=top_layer).rev() {
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
                    .filter_map(|node_id| nodes.get(node_id)),
                &value,
                self.distance_algorithm,
            )
            .pop()
            .map(|ele| ele.0.0)
            .ok_or(Error::NotFoundError(
                "nearest element not found".to_string(),
            ))?;

            enter_point = smallvec![nearest_ele];
        }

        // Deeper search
        for level_current in (0..=min(top_layer, new_elements_lvl)).rev() {
            let layer_index = LayerIndex(level_current as u16);

            // NOTE: W = search-layer(q, ep, efConstruction, lc)
            let nearest_neighbours =
                self.search_layer(&value, &enter_point, self.ef_construction, &layer_index)?;

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
            let value_neighbours_guard = value.neighbours.pin();
            let value_backlinks_guard = value.back_links.pin();
            for neighbour_id in neighbours.iter() {
                let neighbour_node = nodes
                    .get(neighbour_id)
                    .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

                let n_guard = neighbour_node.neighbours.pin();
                n_guard
                    .get_or_insert_with(layer_index, HashSet::new)
                    .pin()
                    .insert(value.id);

                value_neighbours_guard
                    .get_or_insert_with(layer_index, HashSet::new)
                    .pin()
                    .insert(*neighbour_id);

                // NOTE: invert for backlinks
                value_backlinks_guard.insert(*neighbour_id);
                neighbour_node.back_links.pin().insert(value.id);
            }

            graph
                .get_or_insert(layer_index, HashSet::from([value.id]))
                .pin()
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
                let neighbour_node = nodes
                    .get(neighbour)
                    .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

                let nn_guard = neighbour_node.neighbours.pin();
                let e_conn = nn_guard
                    .get(&layer_index)
                    .ok_or(Error::NotFoundError("Index not found".to_string()))?;

                if e_conn.pin().len() > maximum_connections {
                    let e_conn_vec: Vec<NodeId> = e_conn.pin().iter().copied().collect();
                    // Prune neighbor's connections back to Mmax using heuristic selection
                    // (maximum_connections = Mmax0 at layer 0, M at other layers)
                    let new_neighbour_connections = self.select_neighbours_heuristic(
                        neighbour_node,
                        &e_conn_vec,
                        maximum_connections, // Already set to the correct Mmax value above
                        &layer_index,
                        false,
                        false,
                    )?;

                    let neighbour_node = nodes
                        .get(neighbour)
                        .ok_or(Error::NotFoundError("Node Ref not found".to_string()))?;

                    neighbour_node
                        .neighbours
                        .pin()
                        .insert(layer_index, HashSet::from_iter(new_neighbour_connections));
                }
            }

            // NOTE: Find Best Enter Point
            // Find nearest neighbor from this layer to use as entry point for next layer
            // (Algorithm 1: ep = get_nearest_element_from_W_to_q)
            enter_point = match self.find_best_entry_point(&value, &nearest_neighbours)? {
                None => enter_point,
                Some(new_enter_point) => smallvec![new_enter_point],
            };
        }

        let value_id = value.id;
        nodes.insert(value.id, value);

        // NOTE: given that we use u8 for topmost layer, we want that on first insertion we always
        // set enterpoint else this would be a pain
        //
        // Update enter_point and top_most_layer atomically under the enter_point write lock
        {
            let mut ep = self.enter_point.write();
            let current_top = self.top_most_layer.load(Ordering::Acquire);
            if new_elements_lvl > current_top || nodes.len() == 1 {
                self.top_most_layer
                    .store(new_elements_lvl, Ordering::Release);
                *ep = smallvec![value_id];
            }
        }
        Ok(())
    }

    /// Search for ef nearest neighbours in a specific layer
    /// Corresponds to Algorithm 2 (SEARCH-LAYER)
    pub fn search_layer(
        &self,
        query: &Node,
        entry_points: &[NodeId],
        ef: usize,
        layer: &LayerIndex,
    ) -> Result<Vec<NodeId>, Error> {
        let nodes = self.nodes.pin();
        let mut visited_items: NodeIdHashSet = entry_points.iter().copied().collect();

        // C - candidates (min heap via Reverse: smallest distance pops first)
        let mut candidates = MinHeapQueue::from_nodes(
            entry_points.iter().filter_map(|id| nodes.get(id)),
            query,
            self.distance_algorithm,
        );

        // W - bounded min heap: keeps ef nearest (smallest distance) neighbors
        let ef_nonzero = NonZeroUsize::new(ef).unwrap_or(NonZeroUsize::new(1).unwrap());
        let mut nearest_neighbours: BoundedMinHeap<OrderedNode> = BoundedMinHeap::new(ef_nonzero);
        for node in entry_points.iter().filter_map(|id| nodes.get(id)) {
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

            let visited_node = nodes
                .get(&nearest_id)
                .ok_or(Error::NotFoundError("Node not found".to_string()))?;

            // Explore neighbors
            let vn_neighbours_guard = visited_node.neighbours.pin();
            if let Some(visited_node_neighbours) = vn_neighbours_guard.get(layer) {
                for neighbour_id in visited_node_neighbours.pin().iter() {
                    if visited_items.contains(neighbour_id) {
                        continue;
                    }
                    visited_items.insert(*neighbour_id);

                    let neighbour_node = nodes
                        .get(neighbour_id)
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
        candidates: &[NodeId],
        m: usize,
        layer: &LayerIndex,
        extend_candidates: bool,
        keep_pruned_connections: bool,
    ) -> Result<Vec<NodeId>, Error> {
        let nodes = self.nodes.pin();

        let mut response =
            MinHeapQueue::from_nodes(std::iter::empty(), query, self.distance_algorithm);

        let mut working_queue = MinHeapQueue::from_nodes(
            candidates.iter().filter_map(|id| nodes.get(id)),
            query,
            self.distance_algorithm,
        );

        if extend_candidates {
            for candidate in candidates.iter() {
                // TODO: loop error handling??

                let candidate_node = nodes
                    .get(candidate)
                    .ok_or(Error::NotFoundError(" Node Ref not Found".to_string()))?;

                let cn_guard = candidate_node.neighbours.pin();
                let neighbours_at = cn_guard
                    .get(layer)
                    .ok_or(Error::NotFoundError(format!("{:?} not Found", layer)))?;

                for neighbour_id in neighbours_at.pin().iter() {
                    if !working_queue.contains(neighbour_id)
                        && let Some(neighbour_node) = nodes.get(neighbour_id)
                    {
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
                let node = nodes
                    .get(&candidate_id)
                    .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?;
                response.push(node);
                continue;
            }

            // Get the candidate node to compute distances to already-selected neighbors
            let candidate_node = nodes
                .get(&candidate_id)
                .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?;
            // Check if candidate is closer to query than to any already-selected neighbor
            let mut is_diverse = true;
            for Reverse(OrderedNode((selected_id, _))) in response.heap.iter() {
                let selected_node = nodes
                    .get(selected_id)
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

                let node = nodes
                    .get(&nearest_from_wd_to_q)
                    .ok_or(Error::NotFoundError("Node Ref not Found".to_string()))?;
                response.push(node);
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
        let nodes = self.nodes.pin();
        let valid_len = NonZeroUsize::new(k).expect("K should be a non zero number");

        let ef = ef.unwrap_or_else(|| k.max(50));
        // Ensure ef >= k as per paper requirements
        let ef = ef.max(k);

        // Read enter_point and top_most_layer together under the enter_point read lock
        // to ensure a consistent snapshot
        let (mut enter_point, ep_level) = {
            let ep = self.enter_point.read();
            (ep.clone(), self.top_most_layer.load(Ordering::Acquire))
        };

        for level_current in (1..=ep_level).rev() {
            let layer = LayerIndex(level_current as u16);

            let searched = self.search_layer(query, &enter_point, 1, &layer)?;

            let ep = MinHeapQueue::from_nodes(
                searched.iter().filter_map(|id| nodes.get(id)),
                query,
                self.distance_algorithm,
            )
            .peek()
            .map(|OrderedNode((node_id, _))| *node_id)
            .ok_or(Error::QueueEmpty)?;
            enter_point = smallvec![ep];
        }

        let level_zero = self.search_layer(query, &enter_point, ef, &LayerIndex(0))?;
        let mut current_nearest_elements = MinHeapQueue::from_nodes(
            level_zero.iter().filter_map(|id| nodes.get(id)),
            query,
            self.distance_algorithm,
        );

        Ok(current_nearest_elements
            .pop_n(valid_len)
            .into_iter()
            .map(|OrderedNode((node_id, _))| node_id)
            .collect())
    }

    /// Delete a single element from the HNSW graph by NodeId.
    pub fn delete_node(&self, node_id: &NodeId) {
        let nodes = self.nodes.pin();
        let graph = self.graph.pin();

        if let Some(node) = nodes.get(node_id) {
            for backlink in &node.back_links.pin() {
                let related = nodes.get(backlink).unwrap();

                let guard = related.neighbours.pin();
                let neighbour_keys_inner = guard.keys();

                for layer_index in neighbour_keys_inner {
                    if let Some(set) = guard.get(layer_index) {
                        set.pin().remove(node_id);
                    };

                    if let Some(layer_set) = graph.get(layer_index) {
                        layer_set.pin().remove(node_id);
                    }
                }
                related.back_links.pin().remove(node_id);
            }

            nodes.remove(node_id);
        }
    }

    // finds the best entry point from candidates
    fn find_best_entry_point(
        &self,
        query: &Node,
        candidates: &[NodeId],
    ) -> Result<Option<NodeId>, Error> {
        let nodes = self.nodes.pin();

        if candidates.is_empty() {
            // Edge case: no neighbors found at this layer (shouldn't happen in practice)
            // Keep current entry point
            Ok(None)
        } else {
            let enter_point = MinHeapQueue::from_nodes(
                candidates.iter().filter_map(|node_id| nodes.get(node_id)),
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

    #[cfg(test)]
    /// Get a node by NodeId (cloned from the concurrent map)
    fn get_node(&self, id: &NodeId) -> Option<Node> {
        let nodes = self.nodes.pin();
        nodes.get(id).map(|n| n.clone())
    }
}

impl Default for HNSW<LinearAlgorithm> {
    fn default() -> Self {
        let config = HNSWConfig::default();
        let inv_log_m = 1.0 / f64::ln(config.maximum_connections as f64);

        let distance_algorithm = LinearAlgorithm::EuclideanDistance;

        Self {
            ef_construction: config.ef_construction,
            top_most_layer: AtomicU8::new(0),
            maximum_connections: config.maximum_connections,
            maximum_connections_zero: config.maximum_connections_zero,
            inv_log_m, // ln(1/M)
            graph: papaya::HashMap::new(),
            nodes: papaya::HashMap::new(),
            enter_point: RwLock::new(SmallVec::new()),
            distance_algorithm,

            extend_candidates: config.extend_candidates,
            keep_pruned_connections: config.keep_pruned_connections,
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
    use papaya::HashMap;

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
        assert_eq!(node_hashset.clone(), HashSet::from_iter([node_id.clone()]));
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
                .pin()
                .contains(&b)
        );
        assert!(
            b_node
                .neighbours
                .pin()
                .get(&LayerIndex(0))
                .unwrap()
                .pin()
                .contains(&a)
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
                id: id.clone(),
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
                    let set_guard = neighbours.pin();
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
            id: a.clone(),
            value: EmbeddingKey::new(vec![0.0]),
            neighbours: HashMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.insert_node(Node {
            id: b.clone(),
            value: EmbeddingKey::new(vec![10.0]),
            neighbours: HashMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        let node_id = NodeId(99);
        let query_node = Node {
            id: node_id.clone(),
            value: EmbeddingKey::new(vec![1.0]),
            neighbours: HashMap::new(),
            back_links: HashSet::new(),
        };

        let res = hnsw.knn_search(&query_node, 1, Some(10)).unwrap();
        assert_eq!(res[0], a);
    }

    #[test]
    fn test_delete_leaf_node() {
        let hnsw = HNSW::default();

        let a = NodeId(10);
        let b = NodeId(20);

        hnsw.insert_node(Node {
            id: a.clone(),
            value: EmbeddingKey::new(vec![0.0]),
            neighbours: HashMap::new(),
            back_links: HashSet::new(),
        })
        .unwrap();

        hnsw.insert_node(Node {
            id: b.clone(),
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
                .all(|(_, s)| !s.pin().contains(&b))
        );
        assert!(!a_node.back_links.pin().contains(&b));
    }

    #[test]
    fn test_delete_multi_neighbour_node() {
        let hnsw = HNSW::default();

        let ids = [NodeId(10), NodeId(20), NodeId(30), NodeId(40)].to_vec();

        for id in &ids {
            hnsw.insert_node(Node {
                id: id.clone(),
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
                    .all(|(_, s)| !s.pin().contains(target))
            );
            assert!(!node.back_links.pin().contains(target));
        }
    }

    fn assert_hnsw_invariants<D: DistanceFn>(hnsw: &HNSW<D>) {
        let nodes = hnsw.nodes.pin();
        let graph = hnsw.graph.pin();
        let all_ids: Vec<NodeId> = {
            // Collect all IDs from the graph layers
            let mut ids = std::collections::HashSet::new();
            for (_, layer_nodes) in graph.iter() {
                for id in layer_nodes.pin().iter() {
                    ids.insert(*id);
                }
            }
            ids.into_iter().collect()
        };

        for id in &all_ids {
            if let Some(node) = nodes.get(id) {
                // neighbours must exist
                let neighbours_guard = node.neighbours.pin();
                for (_, neighbours) in neighbours_guard.iter() {
                    for n in neighbours.pin().iter() {
                        assert!(nodes.contains_key(n));
                    }
                }

                // Back-links must exist
                for b in node.back_links.pin().iter() {
                    assert!(nodes.contains_key(b));
                }

                // Bidirectional consistency
                for (lc, neighbours) in neighbours_guard.iter() {
                    assert!(graph.get(lc).unwrap().pin().contains(id));
                    for n in neighbours.pin().iter() {
                        let other = nodes.get(n).unwrap();
                        assert!(other.back_links.pin().contains(id));
                    }
                }
            }
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
