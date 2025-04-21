mod heap;
pub mod non_linear;
mod similarity;

use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::num::NonZeroUsize;

use grpc_types::algorithm::algorithms::Algorithm;
use grpc_types::algorithm::nonlinear::NonLinearAlgorithm;
use grpc_types::keyval::StoreKey;
use heap::HeapOrder;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;

use self::similarity::SimilarityFunc;

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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub(crate) enum LinearAlgorithm {
    EuclideanDistance,
    CosineSimilarity,
    DotProductSimilarity,
}

#[derive(Debug)]
pub(crate) struct SimilarityVector<'a>((&'a StoreKey, f32));

impl<'a> From<(&'a StoreKey, f32)> for SimilarityVector<'a> {
    fn from(value: (&'a StoreKey, f32)) -> SimilarityVector<'a> {
        SimilarityVector((value.0, value.1))
    }
}

impl PartialEq for SimilarityVector<'_> {
    fn eq(&self, other: &Self) -> bool {
        *((self.0).0) == *((other.0).0)
    }
}

impl Eq for SimilarityVector<'_> {}

impl PartialOrd for SimilarityVector<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SimilarityVector<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0)
            .1
            .partial_cmp(&(other.0).1)
            .unwrap_or(std::cmp::Ordering::Less)
    }
}

// Pop the topmost N from a generic binary tree
fn pop_n<T: Ord>(heap: &mut BinaryHeap<T>, n: NonZeroUsize) -> Vec<T> {
    (0..n.get()).filter_map(|_| heap.pop()).collect()
}

pub(crate) trait FindSimilarN {
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &StoreKey,
        search_list: impl ParallelIterator<Item = &'a StoreKey>,
        _used_all: bool,
        n: NonZeroUsize,
    ) -> Vec<(StoreKey, f32)>;
}

impl FindSimilarN for LinearAlgorithm {
    #[tracing::instrument(skip_all)]
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &StoreKey,
        search_list: impl ParallelIterator<Item = &'a StoreKey>,
        _used_all: bool,
        n: NonZeroUsize,
    ) -> Vec<(StoreKey, f32)> {
        let heap_order: HeapOrder = self.into();
        let similarity_function: SimilarityFunc = self.into();

        match heap_order {
            HeapOrder::Min => pop_n(
                &mut search_list
                    .map(|second_vector| {
                        let similarity = similarity_function(search_vector, second_vector);
                        let heap_value: SimilarityVector = (second_vector, similarity).into();
                        Reverse(heap_value)
                    })
                    .collect::<BinaryHeap<_>>(),
                n,
            )
            .into_par_iter()
            .map(|a| (a.0 .0 .0.clone(), a.0 .0 .1))
            .collect(),
            HeapOrder::Max => pop_n(
                &mut search_list
                    .map(|second_vector| {
                        let similarity = similarity_function(search_vector, second_vector);
                        let heap_value: SimilarityVector = (second_vector, similarity).into();
                        heap_value
                    })
                    .collect::<BinaryHeap<_>>(),
                n,
            )
            .into_par_iter()
            .map(|a| (a.0 .0.clone(), a.0 .1))
            .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rayon::iter::IntoParallelRefIterator;

    use super::*;
    use crate::tests::*;

    #[test]
    fn test_teststore_find_top_3_similar_words_using_find_nearest_n() {
        let sentences_vectors = word_to_vector();

        let first_vector = sentences_vectors.get(SEACH_TEXT).unwrap().to_owned();

        let mut search_list = vec![];

        for sentence in SENTENCES.iter() {
            let second_vector = sentences_vectors.get(*sentence).unwrap().to_owned();

            search_list.push(second_vector)
        }

        let no_similar_values: usize = 3;

        let cosine_algorithm = LinearAlgorithm::CosineSimilarity;

        let similar_n_search = cosine_algorithm.find_similar_n(
            &first_vector,
            search_list.par_iter(),
            false,
            NonZeroUsize::new(no_similar_values).unwrap(),
        );

        let similar_n_vecs: Vec<StoreKey> = similar_n_search
            .into_iter()
            .map(|(vector, _)| vector.to_owned())
            .collect();

        let most_similar_sentences_vec: Vec<StoreKey> = MOST_SIMILAR
            .iter()
            .map(|sentence| sentences_vectors.get(*sentence).unwrap().to_owned())
            .collect();

        assert_eq!(most_similar_sentences_vec, similar_n_vecs);
    }
}
