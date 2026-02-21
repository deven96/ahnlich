use super::LinearAlgorithm;
use ahnlich_similarity::distance::{cosine_similarity, dot_product, euclidean_distance};
use std::ops::Deref;

type SimFuncSig = fn(&[f32], &[f32]) -> f32;

pub(crate) struct SimilarityFunc(SimFuncSig);

impl Deref for SimilarityFunc {
    type Target = SimFuncSig;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<&LinearAlgorithm> for SimilarityFunc {
    fn from(value: &LinearAlgorithm) -> SimilarityFunc {
        match value {
            LinearAlgorithm::CosineSimilarity => SimilarityFunc(cosine_similarity),

            LinearAlgorithm::EuclideanDistance => SimilarityFunc(euclidean_distance),

            LinearAlgorithm::DotProductSimilarity => SimilarityFunc(dot_product),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::tests::*;
    use ahnlich_types::keyval::StoreKey;

    use rayon::{iter::ParallelIterator, slice::ParallelSlice};

    fn euclidean_distance_comp(first: &[f32], second: &[f32]) -> f32 {
        // Calculate the sum of squared differences for each dimension
        let mut sum_of_squared_differences = 0.0;
        for (&coord1, &coord2) in first.iter().zip(second.iter()) {
            let diff = coord1 - coord2;
            sum_of_squared_differences += diff * diff;
        }

        // Calculate the square root of the sum of squared differences
        f32::sqrt(sum_of_squared_differences)
    }

    fn dot_product_comp(first: &[f32], second: &[f32]) -> f32 {
        first.iter().zip(second).map(|(&x, &y)| x * y).sum::<f32>()
    }

    // simple cosine similarity for correctness comparison against simd variant
    fn cosine_similarity_comp(first: &[f32], second: &[f32]) -> f32 {
        // formular = dot product of vectors / product of the magnitude of the vectors
        // maginiture of a vector can be calcuated using pythagoras theorem.
        // sqrt of sum of vector values
        //
        //

        let dot = dot_product(first, second);

        // the magnitude can be calculated using the arr.norm method.
        let mag_first = &first.iter().map(|x| x * x).sum::<f32>(); //.sqrt();

        let mag_second = &second.iter().map(|x| x * x).sum::<f32>(); //.sqrt();

        dot / (mag_first.sqrt() * mag_second.sqrt())
    }

    fn time_execution<F>(mut func: F) -> Duration
    where
        F: FnMut(),
    {
        let start = Instant::now();
        func();
        start.elapsed()
    }

    // NOTE: Test runs a bit slow due to the number gen here
    // Tried using fast number generation plus fixed buffers and parallel iteration but
    // the test alone still clocks ~7-8 seconds locally
    fn generate_test_array(size: usize, dimension: usize) -> Vec<StoreKey> {
        let mut buffer: Vec<f32> = Vec::with_capacity(size * dimension);
        buffer.extend((0..size * dimension).map(|_| fastrand::f32()));

        // Use Rayon to process the buffer in parallel
        buffer
            .par_chunks_exact(dimension)
            .map(|chunk| StoreKey {
                key: chunk.to_owned(),
            })
            .collect()
    }

    #[test]
    fn test_cosine_simd_vs_cosine_100k_entries_of_1k_size_embeddings() {
        // pick high dimensionality of 1024 to hopefully show difference between SIMD and non-SIMD
        // variants of implementation
        let dimension = 1024;
        // simulate against 100k store inputs
        let size = 100_000;
        // a single array of comparison
        let comp_array: StoreKey = generate_test_array(1, dimension)
            .pop()
            .expect("Could not get comp array");
        let store_values: Vec<StoreKey> = generate_test_array(size, dimension);

        let without_simd_duration = time_execution(|| {
            for val in &store_values {
                cosine_similarity_comp(&comp_array.key, &val.key);
            }
        });

        let with_simd_duration = time_execution(|| {
            for val in &store_values {
                cosine_similarity(&comp_array.key, &val.key);
            }
        });

        println!(
            "Without SIMD duration {}",
            without_simd_duration.as_millis()
        );
        println!("With SIMD duration {}", with_simd_duration.as_millis());
        assert!(with_simd_duration < without_simd_duration)
    }

    #[test]
    fn test_verify_dot_product_simd() {
        let array_one = vec![1.0f32, 1.1, 1.2, 1.3, 2.0, 3.1, 3.2, 4.1, 5.1];
        let array_two = vec![2.0f32, 3.1, 1.2, 1.3, 2.0, 3.0, 3.2, 4.1, 5.1];

        let scalar_dot_product = dot_product_comp(&array_one, &array_two);
        let simd_dot_product = dot_product(&array_one, &array_two);

        assert_eq!(scalar_dot_product, simd_dot_product);
    }

    #[test]
    fn test_simd_dot_product_vs_scalar_dot_product_100k_entries_of_1k_size_embeddings() {
        // pick high dimensionality of 1024 to hopefully show difference between SIMD and non-SIMD
        // variants of implementation
        let dimension = 1024;
        // simulate against 100k store inputs
        let size = 100_000;
        // a single array of comparison
        let comp_array: StoreKey = generate_test_array(1, dimension)
            .pop()
            .expect("Could not get comp array");
        let store_values: Vec<StoreKey> = generate_test_array(size, dimension);

        let without_simd_duration = time_execution(|| {
            for val in &store_values {
                dot_product_comp(&comp_array.key, &val.key);
            }
        });

        let with_simd_duration = time_execution(|| {
            for val in &store_values {
                dot_product(&comp_array.key, &val.key);
            }
        });

        println!(
            "Without SIMD duration {}",
            without_simd_duration.as_millis()
        );
        println!("With SIMD duration {}", with_simd_duration.as_millis());
        assert!(with_simd_duration < without_simd_duration)
    }

    #[test]
    fn test_verify_simd_cosine_sim() {
        let array_one = vec![1.0f32, 1.1, 1.2, 1.3, 2.0, 3.1, 3.2, 4.1, 5.1];
        let array_two = vec![2.0f32, 3.1, 1.2, 1.3, 2.0, 3.0, 3.2, 4.1, 5.1];

        let scalar_cos_sim = cosine_similarity_comp(&array_one, &array_two);
        let simd_cos_sim = cosine_similarity(&array_one, &array_two);

        assert_eq!(scalar_cos_sim, simd_cos_sim);
    }

    #[test]
    fn test_verify_euclidean_distance_simd() {
        let array_one = vec![1.0f32, 1.1, 1.2, 1.3, 2.0, 3.1, 3.2, 4.1, 5.1];
        let array_two = vec![2.0f32, 3.1, 1.2, 1.3, 2.0, 3.0, 3.2, 4.1, 5.1];

        let scalar_euclid = euclidean_distance_comp(&array_one, &array_two);
        let simd_euclid = euclidean_distance(&array_one, &array_two);

        assert_eq!(scalar_euclid, simd_euclid);
    }

    #[test]
    fn test_euclidean_simd_vs_scalar_euclidean_100k_entries_of_1k_size_embeddings() {
        // pick high dimensionality of 1024 to hopefully show difference between SIMD and non-SIMD
        // variants of implementation
        let dimension = 1024;
        // simulate against 100k store inputs
        let size = 100_000;
        // a single array of comparison
        let comp_array: StoreKey = generate_test_array(1, dimension)
            .pop()
            .expect("Could not get comp array");
        let store_values: Vec<StoreKey> = generate_test_array(size, dimension);

        let without_simd_duration = time_execution(|| {
            for val in &store_values {
                euclidean_distance_comp(&comp_array.key, &val.key);
            }
        });

        let with_simd_duration = time_execution(|| {
            for val in &store_values {
                euclidean_distance(&comp_array.key, &val.key);
            }
        });

        println!(
            "Without SIMD duration {}",
            without_simd_duration.as_millis()
        );
        println!("With SIMD duration {}", with_simd_duration.as_millis());
        assert!(with_simd_duration < without_simd_duration)
    }

    #[test]
    fn test_find_top_3_similar_words_using_cosine_similarity() {
        let sentences_vectors = word_to_vector();

        let mut most_similar_result: Vec<(&'static str, f32)> = vec![];

        let first_vector = sentences_vectors.get(SEACH_TEXT).unwrap().to_owned();

        for sentence in SENTENCES.iter() {
            let second_vector = sentences_vectors.get(*sentence).unwrap().to_owned();

            let similarity = cosine_similarity(&first_vector.key, &second_vector.key);

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

        let mut most_similar_result: Vec<(&'static str, f32)> = vec![];

        let first_vector = sentences_vectors.get(SEACH_TEXT).unwrap().to_owned();

        for sentence in SENTENCES.iter() {
            let second_vector = sentences_vectors.get(*sentence).unwrap().to_owned();

            let similarity = euclidean_distance(&first_vector.key, &second_vector.key);

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

        let mut most_similar_result: Vec<(&'static str, f32)> = vec![];

        let first_vector = sentences_vectors.get(SEACH_TEXT).unwrap().to_owned();

        for sentence in SENTENCES.iter() {
            let second_vector = sentences_vectors.get(*sentence).unwrap().to_owned();

            let similarity = dot_product(&first_vector.key, &second_vector.key);

            most_similar_result.push((*sentence, similarity))
        }

        // sort by largest first, the larger the value the more similar it is
        most_similar_result.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let final_result: Vec<&'static str> = most_similar_result.iter().map(|s| s.0).collect();

        assert_eq!(MOST_SIMILAR.to_vec(), final_result[..3]);
    }
}
