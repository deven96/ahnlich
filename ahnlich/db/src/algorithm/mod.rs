mod heap;
pub mod non_linear;
mod similarity;

use std::num::NonZeroUsize;

use ahnlich_similarity::{EmbeddingKey, LinearAlgorithm};
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use heap::{BoundedMaxHeap, BoundedMinHeap, HeapOrder};
use rayon::iter::ParallelIterator;

use self::similarity::SimilarityFunc;
use crate::engine::store::StoreKeyId;

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
        }
    }
}

#[derive(Debug)]
pub(crate) struct SimilarityVector((StoreKeyId, f32));

impl From<(StoreKeyId, f32)> for SimilarityVector {
    fn from(value: (StoreKeyId, f32)) -> SimilarityVector {
        SimilarityVector((value.0, value.1))
    }
}

impl PartialEq for SimilarityVector {
    fn eq(&self, other: &Self) -> bool {
        (self.0).0 == (other.0).0
    }
}

impl Eq for SimilarityVector {}

impl PartialOrd for SimilarityVector {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SimilarityVector {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0)
            .1
            .partial_cmp(&(other.0).1)
            .unwrap_or(std::cmp::Ordering::Less)
    }
}

pub(crate) trait FindSimilarN {
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &EmbeddingKey,
        search_list: impl ParallelIterator<Item = &'a EmbeddingKey>,
        _used_all: bool,
        n: NonZeroUsize,
    ) -> Vec<(StoreKeyId, f32)>;
}

impl FindSimilarN for LinearAlgorithm {
    #[tracing::instrument(skip_all)]
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &EmbeddingKey,
        search_list: impl ParallelIterator<Item = &'a EmbeddingKey>,
        _used_all: bool,
        n: NonZeroUsize,
    ) -> Vec<(StoreKeyId, f32)> {
        let heap_order: HeapOrder = self.into();
        let similarity_function: SimilarityFunc = self.into();

        match heap_order {
            HeapOrder::Min => {
                // Use bounded min heap for minimum similarity (Euclidean distance)
                let bounded_heap = search_list
                    .fold(
                        || BoundedMinHeap::new(n),
                        |mut heap, second_vector| {
                            let similarity = similarity_function(
                                search_vector.as_slice(),
                                second_vector.as_slice(),
                            );
                            let key_id = StoreKeyId::from(second_vector);
                            let heap_value: SimilarityVector = (key_id, similarity).into();
                            heap.push(heap_value);
                            heap
                        },
                    )
                    .reduce(
                        || BoundedMinHeap::new(n),
                        |mut heap1, heap2| {
                            for item in heap2.into_sorted_vec() {
                                heap1.push(item);
                            }
                            heap1
                        },
                    );

                bounded_heap
                    .into_sorted_vec()
                    .into_iter()
                    .map(|sv| (sv.0.0, sv.0.1))
                    .collect()
            }
            HeapOrder::Max => {
                // Use bounded max heap for maximum similarity (Cosine, Dot Product)
                let bounded_heap = search_list
                    .fold(
                        || BoundedMaxHeap::new(n),
                        |mut heap, second_vector| {
                            let similarity = similarity_function(
                                search_vector.as_slice(),
                                second_vector.as_slice(),
                            );
                            let key_id = StoreKeyId::from(second_vector);
                            let heap_value: SimilarityVector = (key_id, similarity).into();
                            heap.push(heap_value);
                            heap
                        },
                    )
                    .reduce(
                        || BoundedMaxHeap::new(n),
                        |mut heap1, heap2| {
                            for item in heap2.into_sorted_vec() {
                                heap1.push(item);
                            }
                            heap1
                        },
                    );

                bounded_heap
                    .into_sorted_vec()
                    .into_iter()
                    .map(|sv| (sv.0.0, sv.0.1))
                    .collect()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rayon::iter::IntoParallelRefIterator;

    use super::*;
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
            .map(|key| (StoreKeyId::from(key), key))
            .collect();

        let similar_n_search = cosine_algorithm.find_similar_n(
            &first_embedding,
            search_list.par_iter(),
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
