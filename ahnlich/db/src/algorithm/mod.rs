pub mod non_linear;
mod similarity;

use std::num::NonZeroUsize;

use ahnlich_similarity::heap::BoundedMaxHeap;
use ahnlich_similarity::{
    Closeness, DistanceFn, EmbeddingKey, LinearAlgorithm, hnsw::HNSWConfig as SimilarityHnswConfig,
};
use ahnlich_types::algorithm::algorithms::DistanceMetric;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use ahnlich_types::algorithm::{algorithms::Algorithm, nonlinear::HnswConfig};
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::predicates::PredicateCondition;
use ahnlich_types::utils::StoreKeyId;

use crate::engine::predicate::PredicateEvaluator;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum AlgorithmByType {
    Linear(LinearAlgorithm),
    NonLinear(NonLinearAlgorithm),
}

impl From<Algorithm> for AlgorithmByType {
    fn from(input: Algorithm) -> Self {
        match input {
            Algorithm::CosineSimilarity => {
                AlgorithmByType::Linear(LinearAlgorithm::CosineSimilarity)
            }
            Algorithm::EuclideanDistance => {
                AlgorithmByType::Linear(LinearAlgorithm::EuclideanDistance)
            }
            Algorithm::DotProductSimilarity => {
                AlgorithmByType::Linear(LinearAlgorithm::DotProductSimilarity)
            }
            Algorithm::KdTree => AlgorithmByType::NonLinear(NonLinearAlgorithm::KdTree),
            Algorithm::Hnsw => AlgorithmByType::NonLinear(NonLinearAlgorithm::Hnsw),
        }
    }
}

/// A candidate ranked by how close it is to the search vector.
///
/// `closeness` orders it — greater is closer, for every algorithm — while `score` is
/// the algorithm's own number, which is what callers are shown. Ties break on
/// `StoreKeyId` so equally close entries come back in a stable order.
#[derive(Debug)]
pub(crate) struct SimilarityVector {
    key_id: StoreKeyId,
    closeness: Closeness,
    score: f32,
}

impl PartialEq for SimilarityVector {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for SimilarityVector {}

impl PartialOrd for SimilarityVector {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SimilarityVector {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.closeness
            .cmp(&other.closeness)
            .then_with(|| self.key_id.0.cmp(&other.key_id.0))
    }
}

pub(crate) trait FindSimilarN {
    fn find_similar_n_sequential<'a>(
        &'a self,
        search_vector: &EmbeddingKey,
        search_list: impl Iterator<Item = (&'a StoreKeyId, &'a EmbeddingKey, &'a StoreValue)>,
        predicate: Option<&PredicateCondition>,
        _used_all: bool,
        n: NonZeroUsize,
    ) -> Vec<(StoreKeyId, f32)>;
}

impl FindSimilarN for LinearAlgorithm {
    #[tracing::instrument(skip_all)]
    fn find_similar_n_sequential<'a>(
        &'a self,
        search_vector: &EmbeddingKey,
        search_list: impl Iterator<Item = (&'a StoreKeyId, &'a EmbeddingKey, &'a StoreValue)>,
        predicate: Option<&PredicateCondition>,
        _used_all: bool,
        n: NonZeroUsize,
    ) -> Vec<(StoreKeyId, f32)> {
        let mut heap = BoundedMaxHeap::new(n);

        for (key_id, vector, store_value) in search_list {
            // Check predicate inline before computing distance
            if let Some(pred) = predicate
                && !store_value.matches(pred)
            {
                continue;
            }

            // Only compute distance if predicate passed (or no predicate)
            let score = self.score(search_vector.as_slice(), vector.as_slice());
            heap.push(SimilarityVector {
                key_id: *key_id,
                closeness: score.closeness(),
                score: score.value(),
            });
        }

        heap.into_sorted_vec()
            .into_iter()
            .map(|candidate| (candidate.key_id, candidate.score))
            .collect()
    }
}

struct DbHnswConfig {
    distance_algorithm: LinearAlgorithm,
    ef_construction: usize,
    maximum_connections: usize,
    maximum_connections_zero: usize,
    extend_candidates: bool,
    keep_pruned_connections: bool,
}

impl From<HnswConfig> for DbHnswConfig {
    fn from(value: HnswConfig) -> Self {
        let defaults = SimilarityHnswConfig::default();

        let distance_algorithm = if let Some(algo) = value.distance.map(|v| {
            v.try_into().unwrap_or_else(|err| {
                tracing::error!(
                    "illegal distance algorithm selected: {}. Reverting to default.",
                    err
                );
                DistanceMetric::Euclidean
            })
        }) {
            match algo {
                DistanceMetric::Cosine => LinearAlgorithm::CosineSimilarity,
                DistanceMetric::Euclidean => LinearAlgorithm::EuclideanDistance,
                DistanceMetric::DotProduct => LinearAlgorithm::DotProductSimilarity,
            }
        } else {
            LinearAlgorithm::EuclideanDistance
        };

        Self {
            distance_algorithm,
            ef_construction: value
                .ef_construction
                .map(|val| val as usize)
                .unwrap_or(defaults.ef_construction),
            maximum_connections: value
                .maximum_connections
                .map(|val| val as usize)
                .unwrap_or(defaults.maximum_connections),
            maximum_connections_zero: value
                .maximum_connections_zero
                .map(|val| val as usize)
                .unwrap_or(defaults.maximum_connections_zero),

            extend_candidates: value
                .extend_candidates
                .unwrap_or(defaults.extend_candidates),

            keep_pruned_connections: value
                .keep_pruned_connections
                .unwrap_or(defaults.extend_candidates),
        }
    }
}

#[cfg(test)]
mod tests {
    use rayon::prelude::*;

    use super::*;
    use crate::engine::store::embedding_key_to_id;
    use crate::tests::*;
    use ahnlich_types::keyval::StoreKey;

    #[test]
    fn test_teststore_find_top_3_similar_words_using_find_nearest_n() {
        let sentences_vectors = word_to_vector();

        let first_vector = sentences_vectors.get(SEACH_TEXT).unwrap().to_owned();
        let first_embedding = EmbeddingKey::new(first_vector.key.clone());

        let mut search_list: Vec<EmbeddingKey> = vec![];

        for sentence in SENTENCES.iter() {
            let second_vector = sentences_vectors.get(*sentence).unwrap().to_owned();
            search_list.push(EmbeddingKey::new(second_vector.key));
        }

        let no_similar_values: usize = 3;

        let cosine_algorithm = LinearAlgorithm::CosineSimilarity;

        // Build a mapping from StoreKeyId to EmbeddingKey for test verification
        let key_map: std::collections::HashMap<StoreKeyId, &EmbeddingKey> = search_list
            .iter()
            .map(|key| (embedding_key_to_id(key), key))
            .collect();

        // Create dummy StoreValue for test (no metadata filtering needed)
        let dummy_store_value = StoreValue {
            value: std::collections::HashMap::new(),
        };

        // Convert to (StoreKeyId, EmbeddingKey, StoreValue) tuples for the new signature
        let search_with_ids: Vec<_> = search_list
            .iter()
            .map(|key| (embedding_key_to_id(key), key, &dummy_store_value))
            .collect();

        let similar_n_search = cosine_algorithm.find_similar_n_sequential(
            &first_embedding,
            search_with_ids
                .iter()
                .map(|(id, key, val)| (id, *key, *val)),
            None,
            false,
            NonZeroUsize::new(no_similar_values).unwrap(),
        );

        let similar_n_vecs: Vec<StoreKey> = similar_n_search
            .into_iter()
            .map(|(key_id, _)| {
                let emb = key_map.get(&key_id).unwrap();
                StoreKey {
                    key: emb.as_slice().to_vec(),
                }
            })
            .collect();

        let most_similar_sentences_vec: Vec<StoreKey> = MOST_SIMILAR
            .iter()
            .map(|sentence| sentences_vectors.get(*sentence).unwrap().to_owned())
            .collect();

        assert_eq!(most_similar_sentences_vec, similar_n_vecs);
    }
}
