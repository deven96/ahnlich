mod heap;
pub mod non_linear;
mod similarity;

use std::num::NonZeroUsize;

use ahnlich_types::keyval::StoreKey;
use ahnlich_types::similarity::Algorithm;
use ahnlich_types::similarity::NonLinearAlgorithm;

use self::{heap::AlgorithmHeapType, similarity::SimilarityFunc};

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
            Algorithm::KDTree => AlgorithmByType::NonLinear(NonLinearAlgorithm::KDTree),
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
impl<'a> From<SimilarityVector<'a>> for (&'a StoreKey, f32) {
    fn from(value: SimilarityVector<'a>) -> (&'a StoreKey, f32) {
        ((value.0).0, (value.0).1)
    }
}

impl<'a> PartialEq for SimilarityVector<'a> {
    fn eq(&self, other: &Self) -> bool {
        *((self.0).0) == *((other.0).0)
    }
}

impl<'a> Eq for SimilarityVector<'a> {}

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

pub(crate) trait FindSimilarN {
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &StoreKey,
        search_list: impl Iterator<Item = &'a StoreKey>,
        _used_all: bool,
        n: NonZeroUsize,
    ) -> Vec<(StoreKey, f32)>;
}

impl FindSimilarN for LinearAlgorithm {
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &StoreKey,
        search_list: impl Iterator<Item = &'a StoreKey>,
        _used_all: bool,
        n: NonZeroUsize,
    ) -> Vec<(StoreKey, f32)> {
        let mut heap: AlgorithmHeapType = (self, n).into();

        let similarity_function: SimilarityFunc = self.into();

        for second_vector in search_list {
            let similarity = similarity_function(search_vector, second_vector);

            let heap_value: SimilarityVector = (second_vector, similarity).into();
            heap.push(heap_value)
        }
        heap.output()
    }
}

#[cfg(test)]
mod tests {
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
            search_list.iter(),
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
