//! INT8 cosine traversal with float32 reranking.
use super::{BatchedCosineScorer, HNSW, SearchScorer};
use crate::distance::int8_cosine::{DIM, Int8CosineVector, has_avx2_kernel, score_int8};
use crate::hnsw::{EdgeStorage, HNSWConfig, Node, NodeId, OrderedNode};
use crate::{Closeness, DistanceFn, EmbeddingKey, LinearAlgorithm, error::Error};
use std::{collections::HashSet, num::NonZeroUsize, sync::OnceLock};

pub(super) fn cache_requested(algorithm: impl DistanceFn) -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    algorithm.supports_int8_cosine()
        && *ENABLED.get_or_init(|| {
            std::env::var("AHNLICH_BENCH_INT8_TRAVERSAL").as_deref() == Ok("1") && has_avx2_kernel()
        })
}

impl<E: EdgeStorage> Node<E> {
    pub(super) fn cache_int8_cosine(&mut self) {
        if let Ok(vector) = self.value.as_slice().try_into() {
            self.int8_cosine = Int8CosineVector::encode(vector).map(Box::new);
        }
    }
}

#[derive(Clone, Copy)]
struct Int8Scorer<'a>(&'a Int8CosineVector);

impl<E: EdgeStorage> SearchScorer<E, 4> for Int8Scorer<'_> {
    #[inline]
    fn single(self, node: &Node<E>, query: &Node<E>) -> Closeness {
        match node.int8_cosine.as_deref() {
            Some(candidate) => score_int8(self.0, [candidate])[0].into(),
            None => BatchedCosineScorer.single(node, query),
        }
    }

    #[inline]
    fn batch(self, nodes: [&Node<E>; 4], query: &Node<E>) -> [Closeness; 4] {
        let [Some(a), Some(b), Some(c), Some(d)] = nodes.map(|node| node.int8_cosine.as_deref())
        else {
            return nodes.map(|node| self.single(node, query));
        };
        score_int8(self.0, [a, b, c, d]).map(Into::into)
    }
}

impl HNSW<LinearAlgorithm> {
    /// Construct a cosine index with the INT8 cache for the Criterion comparison.
    pub fn new_with_int8_cosine(config: HNSWConfig, shards: NonZeroUsize) -> Option<Self> {
        if !has_avx2_kernel() {
            return None;
        }
        let mut index = Self::new_with_config_and_shared_collector_and_counter_shards(
            config,
            LinearAlgorithm::CosineSimilarity,
            shards,
        );
        index.cache_int8_cosine = true;
        Some(index)
    }
}

impl<E: EdgeStorage> HNSW<LinearAlgorithm, E> {
    /// Score the traversal with INT8 cosine and rerank the entire shortlist with float32.
    /// Queries outside the candidate's supported scope use the existing scorer.
    pub fn n_nearest_ids_with_int8_cosine(
        &self,
        vector: &[f32],
        k: NonZeroUsize,
        accept_list: Option<HashSet<u64>>,
        ef: Option<NonZeroUsize>,
    ) -> Result<Vec<(NodeId, f32)>, Error> {
        if self.cache_int8_cosine
            && accept_list.is_none()
            && let Some(result) = self.int8_nearest_ids(vector, k, ef)
        {
            return result;
        }
        self.n_nearest_ids_with_ef(vector, k, accept_list, ef)
    }

    fn int8_nearest_ids(
        &self,
        vector: &[f32],
        k: NonZeroUsize,
        ef: Option<NonZeroUsize>,
    ) -> Option<Result<Vec<(NodeId, f32)>, Error>> {
        let query_int8 = Int8CosineVector::encode(<&[f32; DIM]>::try_from(vector).ok()?)?;
        let query = Node::<E>::new_with_edges(EmbeddingKey::new(vector.to_vec()));
        let shortlist_size = ef
            .map_or_else(|| k.get().max(50), NonZeroUsize::get)
            .max(k.get());
        let shortlist = self.knn_search_scored_by(
            &query,
            shortlist_size,
            Some(shortlist_size),
            None,
            Int8Scorer(&query_int8),
        );
        Some(shortlist.map(|candidates| self.rerank_int8_shortlist(&query, candidates, k)))
    }

    fn rerank_int8_shortlist(
        &self,
        query: &Node<E>,
        candidates: Vec<OrderedNode>,
        k: NonZeroUsize,
    ) -> Vec<(NodeId, f32)> {
        let nodes = self.nodes.pin();
        let mut exact: Vec<_> = candidates
            .into_iter()
            .filter_map(|candidate| {
                let node = nodes.get(&candidate.id)?;
                let score = BatchedCosineScorer::score(query, node);
                Some((OrderedNode::new(candidate.id, score.into()), score.value()))
            })
            .collect();
        exact.sort_unstable_by(|(a, _), (b, _)| b.cmp(a));
        exact
            .into_iter()
            .take(k.get())
            .map(|(node, score)| (node.id, score))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn int8_cache_is_populated_for_inserts_and_filtered_queries_use_float32() {
        let mut index =
            HNSW::new_with_config(HNSWConfig::default(), LinearAlgorithm::CosineSimilarity);
        index.cache_int8_cosine = true;
        let mut vector = vec![0.0; DIM];
        vector[0] = 1.0;
        let key = EmbeddingKey::new(vector);
        let query = Node::new(key.clone());
        index.insert(std::slice::from_ref(&key)).unwrap();
        let id = *query.id();
        assert!(index.nodes.pin().get(&id).unwrap().int8_cosine.is_some());
        let k = NonZeroUsize::MIN;
        let filter = Some(HashSet::from([id.0]));
        assert_eq!(
            index
                .n_nearest_ids_with_int8_cosine(key.as_slice(), k, filter.clone(), None)
                .unwrap(),
            index
                .n_nearest_ids_with_ef(key.as_slice(), k, filter, None)
                .unwrap(),
        );
        assert_eq!(
            index
                .n_nearest_ids_with_int8_cosine(key.as_slice(), k, None, None)
                .unwrap(),
            vec![(id, 1.0)],
        );
    }

    #[test]
    fn int8_rerank_selects_from_the_whole_shortlist() {
        let index = HNSW::new_with_config(HNSWConfig::default(), LinearAlgorithm::CosineSimilarity);
        let best = EmbeddingKey::new(vec![1.0, 0.0]);
        let worse = EmbeddingKey::new(vec![0.0, 1.0]);
        let query = Node::new(best.clone());
        let best_id = *query.id();
        let worse_id = *Node::new(worse.clone()).id();
        index.insert(&[best, worse]).unwrap();
        let shortlist = vec![
            OrderedNode::new(worse_id, BatchedCosineScorer::score(&query, &query).into()),
            OrderedNode::new(
                best_id,
                BatchedCosineScorer::score(&query, &Node::new(EmbeddingKey::new(vec![0.0, 1.0])))
                    .into(),
            ),
        ];
        let result = index.rerank_int8_shortlist(&query, shortlist, NonZeroUsize::MIN);
        assert_eq!(result, vec![(best_id, 1.0)]);
    }
}
