use std::ops::Deref;
use types::{keyval::StoreKey, similarity::Algorithm};

type SimFuncSig = fn(&StoreKey, &StoreKey) -> f64;

pub(crate) struct SimilarityFunc(SimFuncSig);

impl Deref for SimilarityFunc {
    type Target = SimFuncSig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&Algorithm> for SimilarityFunc {
    fn from(value: &Algorithm) -> SimilarityFunc {
        match value {
            Algorithm::CosineSimilarity => SimilarityFunc(cosine_similarity),

            Algorithm::EuclideanDistance => SimilarityFunc(euclidean_distance),

            Algorithm::DotProductSimilarity => SimilarityFunc(dot_product),
        }
    }
}

///
/// ## COSINE SIMILARITY
/// Cosine similiarity is the cosine of the angles between vectors.
/// It tries to find how close or similar two vector points are.
/// It is scalent invarient meaning it is unaffected by the length of the vectors.
///
///
/// Cosine of the angles between vectors shows how similar, dissimilar or orthogonal(independent).
///
///
/// The range of similarity for cosine similarity ranges from -1 to 1:
/// where:
///    -  1 means similar
///    -  -1 different.
///    -  0 means Orthogonal
///
///
/// To calculate the cosine similarity for two vectors, we need to:
///   - Calculate the dot product of both vectors
///   - Find the product of the magnitude of both vectors.
///          A magnitude of a vector can be calculated using pythagoras theorem:
///          sqrt( A^2 + B^2)
///             where A and B are two vectors.
///
///   - divide the dot product by the product of the magnitude of both vectors.
///        ```
///           cos(0) = A.B / ||A||.||B||
///        ```
///
///     where Magnitude(A or B):
///     ||A|| = sqrt[(1)^2 + (2)^2]
///     
///
///  An Implementation for most similar items would be a MaxHeap.
///  We are looking the closest number to one meaning Max
///  The smaller the distance between two points, denotes higher
///                                  similarity
///

fn cosine_similarity(first: &StoreKey, second: &StoreKey) -> f64 {
    // formular = dot product of vectors / product of the magnitude of the vectors
    // maginiture of a vector can be calcuated using pythagoras theorem.
    // sqrt of sum of vector values
    //
    //

    let dot_product = dot_product(&first, &second);

    // the magnitude can be calculated using the arr.norm method.
    let mag_first = &first.0.iter().map(|x| x * x).sum::<f64>().sqrt();

    let mag_second = &second
        .0
        .iter()
        .map(|x| x * x)
        .map(|x| x)
        .sum::<f64>()
        .sqrt();

    let cosine_sim = dot_product / (mag_first * mag_second);

    cosine_sim
}

///
/// ## DOT PRODUCT
/// The dot product or scalar product is an algebraic operation that takes two equal-length
/// sequences of numbers (usually coordinate vectors), and returns a single number.
///
/// An Implementation for most similar items would be a MaxHeap.
/// The larger the dot product between two vectors, the more similar

fn dot_product(first: &StoreKey, second: &StoreKey) -> f64 {
    let dot_product = second.0.dot(&first.0.t());
    dot_product
}

///  
///  ## Euclidean Distance
///     - d(p,q)= sqrt { (p-q)^2 }
///  Euclidean distance is the square root of the sum of squared differences between corresponding
///  elements of the two vectors.
///
///  Note that the formula treats the values of P and Q seriously: no adjustment is made for
///  differences in scale. Euclidean distance is only appropriate for data measured on the same
///  scale(meaning it is  scale invarient).
///  The distance derived is purely based on the difference between both vectors and is therefore,
///  prone to skewness if the units of the vectors are vastly different.
///
///  Hence, it is important to ensure that the data is normalised before applying the Euclidean
///  distance function.
///
///  An Implementation for most similar items would be a MinHeap, The smaller the distance between
///  two points, denotes higher similarity
///

fn euclidean_distance(first: &StoreKey, second: &StoreKey) -> f64 {
    // Calculate the sum of squared differences for each dimension
    let mut sum_of_squared_differences = 0.0;
    for (&coord1, &coord2) in first.0.iter().zip(second.0.iter()) {
        let diff = coord1 - coord2;
        sum_of_squared_differences += diff * diff;
    }

    // Calculate the square root of the sum of squared differences
    let distance = f64::sqrt(sum_of_squared_differences);
    distance
}

#[cfg(test)]
pub(super) mod tests {
    use std::collections::HashMap;

    use super::*;

    pub(crate) fn key_to_words(
        key: &StoreKey,
        vector_to_sentences: &Vec<(StoreKey, String)>,
    ) -> Option<String> {
        for (vector, word) in vector_to_sentences {
            if key == vector {
                return Some(word.to_string());
            }
        }
        None
    }

    pub(crate) fn word_to_vector() -> HashMap<String, StoreKey> {
        let words = std::fs::read_to_string("tests/mock_data.json").unwrap();

        let words_to_vec: HashMap<String, Vec<f64>> = serde_json::from_str(&words).unwrap();

        HashMap::from_iter(
            words_to_vec
                .into_iter()
                .map(|(key, value)| (key, StoreKey(ndarray::Array1::<f64>::from_vec(value)))),
        )
    }

    pub(crate) const SEACH_TEXT: &'static str =
        "Football fans enjoy gathering to watch matches at sports bars.";

    pub(crate) const MOST_SIMILAR: [&'static str; 3] = [
        "Attending football games at the stadium is an exciting experience.",
        "On sunny days, people often gather outdoors for a friendly game of football.",
        "Rainy weather can sometimes lead to canceled outdoor events like football matches.",
    ];
    pub(crate) const SENTENCES: [&'static str; 5] = [
        "On sunny days, people often gather outdoors for a friendly game of football.",
        "Attending football games at the stadium is an exciting experience.",
        "Grilling burgers and hot dogs is a popular activity during summer barbecues.",
        "Rainy weather can sometimes lead to canceled outdoor events like football matches.",
        "Sunny weather is ideal for outdoor activities like playing football.",
    ];

    #[test]
    fn test_find_top_3_similar_words_using_cosine_similarity() {
        let sentences_vectors = word_to_vector();

        let mut most_similar_result: Vec<(&'static str, f64)> = vec![];

        let first_vector = sentences_vectors.get(SEACH_TEXT).unwrap().to_owned();

        for sentence in SENTENCES.iter() {
            let second_vector = sentences_vectors.get(*sentence).unwrap().to_owned();

            let similarity = cosine_similarity(&first_vector, &second_vector);

            most_similar_result.push((*sentence, similarity))
        }

        // sort by largest first, the closer to one the more similar it is
        most_similar_result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let final_result: Vec<&'static str> = most_similar_result.iter().map(|s| s.0).collect();

        assert_eq!(MOST_SIMILAR.to_vec(), final_result[..3]);
    }
    #[test]
    fn test_find_top_3_similar_words_using_euclidean_distance() {
        let sentences_vectors = word_to_vector();

        let mut most_similar_result: Vec<(&'static str, f64)> = vec![];

        let first_vector = sentences_vectors.get(SEACH_TEXT).unwrap().to_owned();

        for sentence in SENTENCES.iter() {
            let second_vector = sentences_vectors.get(*sentence).unwrap().to_owned();

            let similarity = euclidean_distance(&first_vector, &second_vector);

            most_similar_result.push((*sentence, similarity))
        }

        // sort by smallest first, the closer to zero the more similar it is
        most_similar_result.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let final_result: Vec<&'static str> = most_similar_result.iter().map(|s| s.0).collect();

        assert_eq!(MOST_SIMILAR.to_vec(), final_result[..3]);
    }

    #[test]
    fn test_find_top_3_similar_words_using_dot_product() {
        let sentences_vectors = word_to_vector();

        let mut most_similar_result: Vec<(&'static str, f64)> = vec![];

        let first_vector = sentences_vectors.get(SEACH_TEXT).unwrap().to_owned();

        for sentence in SENTENCES.iter() {
            let second_vector = sentences_vectors.get(*sentence).unwrap().to_owned();

            let similarity = dot_product(&first_vector, &second_vector);

            most_similar_result.push((*sentence, similarity))
        }

        // sort by largest first, the larger the value the more similar it is
        most_similar_result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let final_result: Vec<&'static str> = most_similar_result.iter().map(|s| s.0).collect();

        assert_eq!(MOST_SIMILAR.to_vec(), final_result[..3]);
    }
}
