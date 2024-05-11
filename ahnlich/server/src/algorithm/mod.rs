mod heap;
mod similarity;

use std::num::NonZeroUsize;

use ndarray::prelude::*;
use types::similarity::Algorithm;

use self::{heap::AlgorithmHeapType, similarity::SimilarityFunc};

#[derive(Debug)]
pub(crate) struct SimilarityVectorF64<'a>(
    (&'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>, f64),
);

impl<'a> From<(&'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>, f64)>
    for SimilarityVectorF64<'a>
{
    fn from(
        value: (&'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>, f64),
    ) -> SimilarityVectorF64<'a> {
        SimilarityVectorF64((value.0, value.1))
    }
}
impl<'a> From<SimilarityVectorF64<'a>>
    for (&'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>, f64)
{
    fn from(
        value: SimilarityVectorF64<'a>,
    ) -> (&'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>, f64) {
        ((value.0).0, (value.0).1)
    }
}

impl<'a> PartialEq for SimilarityVectorF64<'a> {
    fn eq(&self, other: &Self) -> bool {
        *((self.0).0) == *((other.0).0)
    }
}

impl<'a> Eq for SimilarityVectorF64<'a> {}

impl PartialOrd for SimilarityVectorF64<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        (self.0).1.partial_cmp(&(other.0).1)
    }
}

impl Ord for SimilarityVectorF64<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        (self.0)
            .1
            .partial_cmp(&(other.0).1)
            .unwrap_or(std::cmp::Ordering::Less)
    }
}

trait KNearestN {
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>,
        search_list: impl Iterator<Item = &'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>>,
        algorithm: &'a Algorithm,
        n: NonZeroUsize,
    ) -> Vec<(&'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>, f64)> {
        let mut heap: AlgorithmHeapType = (algorithm, n).into();

        let similarity_function: SimilarityFunc = algorithm.into();

        for second_vector in search_list {
            let similarity = similarity_function(search_vector, second_vector);

            let heap_value: SimilarityVectorF64 = (second_vector, similarity).into();
            heap.push(heap_value)
        }
        heap.output()
    }
}

#[cfg(test)]
struct TestStore;

#[cfg(test)]
impl TestStore {
    fn new() -> Self {
        Self
    }
}

#[cfg(test)]
impl KNearestN for TestStore {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::algorithm::similarity::tests::*;

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

        let test_store = TestStore::new();

        let similar_n_search = test_store.find_similar_n(
            &first_vector,
            search_list.iter(),
            &Algorithm::CosineSimilarity,
            NonZeroUsize::new(no_similar_values).unwrap(),
        );

        let similar_n_vecs: Vec<ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>> =
            similar_n_search
                .into_iter()
                .map(|(vector, _)| vector.to_owned())
                .collect();

        let most_similar_sentences_vec: Vec<ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>> =
            MOST_SIMILAR
                .iter()
                .map(|sentence| sentences_vectors.get(*sentence).unwrap().to_owned())
                .collect();

        assert_eq!(most_similar_sentences_vec, similar_n_vecs);
    }
}
