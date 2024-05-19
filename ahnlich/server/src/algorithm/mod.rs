mod heap;
mod similarity;

use std::num::NonZeroUsize;

use types::keyval::{SearchInput, StoreKey};
use types::similarity::Algorithm;

use self::{heap::AlgorithmHeapType, similarity::SimilarityFunc};

#[derive(Debug)]
pub(crate) struct SimilarityVector<'a>((&'a StoreKey, f64));

impl<'a> From<(&'a StoreKey, f64)> for SimilarityVector<'a> {
    fn from(value: (&'a StoreKey, f64)) -> SimilarityVector<'a> {
        SimilarityVector((value.0, value.1))
    }
}
impl<'a> From<SimilarityVector<'a>> for (&'a StoreKey, f64) {
    fn from(value: SimilarityVector<'a>) -> (&'a StoreKey, f64) {
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
        search_vector: &SearchInput,
        search_list: impl Iterator<Item = &'a StoreKey>,
        n: NonZeroUsize,
    ) -> Vec<(&'a StoreKey, f64)>;
}

impl FindSimilarN for Algorithm {
    fn find_similar_n<'a>(
        &'a self,
        search_vector: &SearchInput,
        search_list: impl Iterator<Item = &'a StoreKey>,
        n: NonZeroUsize,
    ) -> Vec<(&'a StoreKey, f64)> {
        let mut heap: AlgorithmHeapType = (self, n).into();

        let similarity_function: SimilarityFunc = self.into();
        let search_vector = StoreKey(search_vector.clone());

        for second_vector in search_list {
            let similarity = similarity_function(&search_vector, second_vector);

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

        let cosine_algorithm = Algorithm::CosineSimilarity;

        let similar_n_search = cosine_algorithm.find_similar_n(
            &first_vector.0,
            search_list.iter(),
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
