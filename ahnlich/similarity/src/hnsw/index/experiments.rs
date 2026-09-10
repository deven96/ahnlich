//! HNSW search implementation that reuses calculated distances.
//!
//! The benchmark facades in this module retain the previous search path as a control.

use super::*;

impl<D: DistanceFn> HNSW<D> {
    /// SEARCH-LAYER implementation that computes each query/node distance once.
    ///
    /// This carries `OrderedNode` through traversal so both C and W can reuse the same score.
    pub(crate) fn search_layer_reuse_distances(
        &self,
        query: &Node,
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

        let mut candidates = NearestFirst::from_scored(
            scored_entry_points.iter().copied(),
            query,
            self.distance_algorithm,
        );
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

            let visited_node = nodes
                .get(&nearest.id)
                .ok_or_else(|| Error::NotFoundError("Node not found".to_string()))?;
            let neighbours = visited_node.neighbours.pin();
            if let Some(layer_neighbours) = neighbours.get(layer) {
                for neighbour_id in layer_neighbours.pin().iter() {
                    if !visited_items.insert(*neighbour_id) {
                        continue;
                    }

                    let neighbour_node = nodes
                        .get(neighbour_id)
                        .ok_or_else(|| Error::NotFoundError("Neighbor not found".to_string()))?;
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
                        candidates.push_scored(scored);
                        if is_accepted(neighbour_id) {
                            nearest_neighbours.push(scored);
                        }
                    }
                }
            }
        }

        Ok(nearest_neighbours.iter().copied().collect())
    }

    /// K-NN search that preserves SEARCH-LAYER scores between HNSW layers and through
    /// final top-k selection.
    fn knn_search_reuse_distances_scored(
        &self,
        query: &Node,
        k: usize,
        ef: Option<usize>,
        accept_list: Option<&std::collections::HashSet<u64>>,
    ) -> Result<Vec<OrderedNode>, Error> {
        let valid_len = NonZeroUsize::new(k).expect("K should be a non zero number");
        let ef = ef.unwrap_or_else(|| k.max(50)).max(k);
        let (mut enter_point, ep_level) = {
            let ep = self.enter_point.load();
            (
                ep.as_ref().clone(),
                self.top_most_layer.load(Ordering::Acquire),
            )
        };

        for level_current in (1..=ep_level).rev() {
            let searched = self.search_layer_reuse_distances(
                query,
                &enter_point,
                1,
                &LayerIndex(level_current as u16),
                None,
            )?;
            let nearest = searched.into_iter().max().ok_or(Error::QueueEmpty)?;
            enter_point = smallvec![nearest.id];
        }

        let level_zero = self.search_layer_reuse_distances(
            query,
            &enter_point,
            ef,
            &LayerIndex(0),
            accept_list,
        )?;
        let mut current_nearest_elements =
            NearestFirst::from_scored(level_zero.into_iter(), query, self.distance_algorithm);
        Ok(current_nearest_elements.pop_n(valid_len))
    }

    /// Distance-reusing search with the same result type as `knn_search`.
    pub fn knn_search_reuse_distances(
        &self,
        query: &Node,
        k: usize,
        ef: Option<usize>,
        accept_list: Option<&std::collections::HashSet<u64>>,
    ) -> Result<Vec<NodeId>, Error> {
        Ok(self
            .knn_search_reuse_distances_scored(query, k, ef, accept_list)?
            .into_iter()
            .map(|node| node.id)
            .collect())
    }

    fn materialize_ids_with_scores(
        &self,
        reference_point: &[f32],
        ids: Vec<NodeId>,
    ) -> Vec<(EmbeddingKey, f32)> {
        let nodes = self.nodes.pin();
        ids.into_iter()
            .filter_map(|id| {
                nodes.get(&id).map(|node| {
                    let key = node.value.clone();
                    let value = self
                        .distance_algorithm
                        .value(reference_point, key.as_slice());
                    (key, value)
                })
            })
            .collect()
    }

    /// Benchmark facade for the current main implementation at the public-result boundary.
    pub fn n_nearest_control(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        ef: Option<usize>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        let query = Node::new(EmbeddingKey::new(reference_point.to_vec()));
        let ids = self.knn_search(&query, n.get(), ef, None)?;
        Ok(self.materialize_ids_with_scores(reference_point, ids))
    }

    /// Benchmark facade for the distance-reusing implementation at the public-result boundary.
    pub fn n_nearest_reuse_distances(
        &self,
        reference_point: &[f32],
        n: NonZeroUsize,
        ef: Option<usize>,
    ) -> Result<Vec<(EmbeddingKey, f32)>, Error> {
        let query = Node::new(EmbeddingKey::new(reference_point.to_vec()));
        let ids = self.knn_search_reuse_distances(&query, n.get(), ef, None)?;
        Ok(self.materialize_ids_with_scores(reference_point, ids))
    }
}
