use super::LinearAlgorithm;
use grpc_types::keyval::StoreKey;
use pulp::{Arch, Simd, WithSimd};
use std::ops::Deref;

type SimFuncSig = fn(&StoreKey, &StoreKey) -> f32;

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

struct Magnitude<'a> {
    first: &'a [f32],
    second: &'a [f32],
}

impl WithSimd for Magnitude<'_> {
    type Output = f32;

    #[inline(always)]
    fn with_simd<S: Simd>(self, simd: S) -> Self::Output {
        let (first_head, first_tail) = S::as_simd_f32s(self.first);
        let (second_head, second_tail) = S::as_simd_f32s(self.second);

        let mut mag_first = simd.splat_f32s(0.0);
        let mut mag_second = simd.splat_f32s(0.0);

        // Process the SIMD-aligned portion
        for (&chunk_first, &chunk_second) in first_head.iter().zip(second_head) {
            mag_first = simd.mul_add_f32s(chunk_first, chunk_first, mag_first);
            mag_second = simd.mul_add_f32s(chunk_second, chunk_second, mag_second);
        }

        // Reduce the SIMD results (no sqrt yet)
        let mag_first = simd.reduce_sum_f32s(mag_first);
        let mag_second = simd.reduce_sum_f32s(mag_second);

        // Process the remaining scalar portion, adding to mag_first and mag_second
        let mut scalar_mag_first = 0.0;
        let mut scalar_mag_second = 0.0;

        for (&x, &y) in first_tail.iter().zip(second_tail) {
            scalar_mag_first += x * x;
            scalar_mag_second += y * y;
        }
        let mag_first = mag_first + scalar_mag_first;
        let mag_second = mag_second + scalar_mag_second;

        // Apply sqrt to the total sum of squares
        mag_first.sqrt() * mag_second.sqrt()
    }
}

///
/// COSINE SIMILARITY
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
///        ```#
///            cos(0) = A.B / ||A||.||B||
///        ```
///
///     where Magnitude(A or B):
///     ||A|| = sqrt[(1)^2 + (2)^2]
///     
///
///  An Implementation for most similar items would be a MaxHeap.
///  We are looking the closest number to one meaning Max
///  The smaller the distance between two points, denotes higher similarity.
#[tracing::instrument(skip_all)]
fn cosine_similarity(first: &StoreKey, second: &StoreKey) -> f32 {
    assert_eq!(
        first.key.len(),
        second.key.len(),
        "Vectors must have the same length!"
    );

    let dot_product = dot_product(first, second);

    let arch = Arch::new();
    let magnitude = arch.dispatch(Magnitude {
        first: first.key.as_slice(),
        second: second.key.as_slice(),
    });

    dot_product / magnitude
}

///
/// DOT PRODUCT
/// The dot product or scalar product is an algebraic operation that takes two equal-length
/// sequences of numbers (usually coordinate vectors), and returns a single number.
///
/// An Implementation for most similar items would be a MaxHeap.
/// The larger the dot product between two vectors, the more similar
struct DotProduct<'a> {
    first: &'a [f32],
    second: &'a [f32],
}

impl WithSimd for DotProduct<'_> {
    type Output = f32;

    #[inline(always)]
    fn with_simd<S: Simd>(self, simd: S) -> Self::Output {
        let (first_head, first_tail) = S::as_simd_f32s(self.first);
        let (second_head, second_tail) = S::as_simd_f32s(self.second);

        let mut sum_of_points = simd.splat_f32s(0.0);

        for (&chunk_first, &chunk_second) in first_head.iter().zip(second_head) {
            sum_of_points = simd.mul_add_f32s(chunk_first, chunk_second, sum_of_points);
        }

        let mut dot_product = simd.reduce_sum_f32s(sum_of_points);

        dot_product += first_tail
            .iter()
            .zip(second_tail)
            .map(|(&x, &y)| x * y)
            .sum::<f32>();
        dot_product
    }
}

#[tracing::instrument(skip_all)]
fn dot_product(first: &StoreKey, second: &StoreKey) -> f32 {
    assert_eq!(
        first.key.len(),
        second.key.len(),
        "Vectors must have the same length!"
    );

    let arch = Arch::new();
    arch.dispatch(DotProduct {
        first: first.key.as_slice(),
        second: second.key.as_slice(),
    })
}

///  
///  EUCLIDEAN DISTANCE
///     - d(p,q)= sqrt { (p-q)^2 }
///
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
struct EuclideanDistance<'a> {
    first: &'a [f32],
    second: &'a [f32],
}

impl WithSimd for EuclideanDistance<'_> {
    type Output = f32;

    #[inline(always)]
    fn with_simd<S: Simd>(self, simd: S) -> Self::Output {
        let (first_head, first_tail) = S::as_simd_f32s(self.first);
        let (second_head, second_tail) = S::as_simd_f32s(self.second);

        let mut sum_of_squares = simd.splat_f32s(0.0);

        for (&cord_first, &cord_second) in first_head.iter().zip(second_head) {
            let diff = simd.sub_f32s(cord_first, cord_second);
            sum_of_squares = simd.mul_add_f32s(diff, diff, sum_of_squares);
        }

        let mut total = simd.reduce_sum_f32s(sum_of_squares);

        total += first_tail
            .iter()
            .zip(second_tail)
            .map(|(&x, &y)| {
                let diff = x - y;
                diff * diff
            })
            .sum::<f32>();

        total.sqrt()
    }
}

#[tracing::instrument(skip_all)]
fn euclidean_distance(first: &StoreKey, second: &StoreKey) -> f32 {
    // Calculate the sum of squared differences for each dimension
    assert_eq!(
        first.key.len(),
        second.key.len(),
        "Vectors must have the same length!"
    );

    let arch = Arch::new();

    arch.dispatch(EuclideanDistance {
        first: first.key.as_slice(),
        second: second.key.as_slice(),
    })
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::*;
    use crate::tests::*;

    use rayon::{iter::ParallelIterator, slice::ParallelSlice};

    fn euclidean_distance_comp(first: &StoreKey, second: &StoreKey) -> f32 {
        // Calculate the sum of squared differences for each dimension
        let mut sum_of_squared_differences = 0.0;
        for (&coord1, &coord2) in first.key.iter().zip(second.key.iter()) {
            let diff = coord1 - coord2;
            sum_of_squared_differences += diff * diff;
        }

        // Calculate the square root of the sum of squared differences
        f32::sqrt(sum_of_squared_differences)
    }

    fn dot_product_comp(first: &StoreKey, second: &StoreKey) -> f32 {
        first
            .key
            .iter()
            .zip(&second.key)
            .map(|(&x, &y)| x * y)
            .sum::<f32>()
    }

    // simple cosine similarity for correctness comparison against simd variant
    fn cosine_similarity_comp(first: &StoreKey, second: &StoreKey) -> f32 {
        // formular = dot product of vectors / product of the magnitude of the vectors
        // maginiture of a vector can be calcuated using pythagoras theorem.
        // sqrt of sum of vector values
        //
        //

        let dot_product = dot_product(first, second);

        // the magnitude can be calculated using the arr.norm method.
        let mag_first = &first.key.iter().map(|x| x * x).sum::<f32>(); //.sqrt();

        let mag_second = &second.key.iter().map(|x| x * x).sum::<f32>(); //.sqrt();

        dot_product / (mag_first.sqrt() * mag_second.sqrt())
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
                cosine_similarity_comp(&comp_array, val);
            }
        });

        let with_simd_duration = time_execution(|| {
            for val in &store_values {
                cosine_similarity(&comp_array, val);
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
        let array_one = StoreKey {
            key: vec![1.0, 1.1, 1.2, 1.3, 2.0, 3.1, 3.2, 4.1, 5.1],
        };
        let array_two = StoreKey {
            key: vec![2.0, 3.1, 1.2, 1.3, 2.0, 3.0, 3.2, 4.1, 5.1],
        };

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
                dot_product_comp(&comp_array, val);
            }
        });

        let with_simd_duration = time_execution(|| {
            for val in &store_values {
                dot_product(&comp_array, val);
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
        let array_one = StoreKey {
            key: vec![1.0, 1.1, 1.2, 1.3, 2.0, 3.1, 3.2, 4.1, 5.1],
        };
        let array_two = StoreKey {
            key: vec![2.0, 3.1, 1.2, 1.3, 2.0, 3.0, 3.2, 4.1, 5.1],
        };

        let scalar_cos_sim = cosine_similarity_comp(&array_one, &array_two);
        let simd_cos_sim = cosine_similarity(&array_one, &array_two);

        assert_eq!(scalar_cos_sim, simd_cos_sim);
    }

    #[test]
    fn test_verify_euclidean_distance_simd() {
        let array_one = StoreKey {
            key: vec![1.0, 1.1, 1.2, 1.3, 2.0, 3.1, 3.2, 4.1, 5.1],
        };
        let array_two = StoreKey {
            key: vec![2.0, 3.1, 1.2, 1.3, 2.0, 3.0, 3.2, 4.1, 5.1],
        };

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
                euclidean_distance_comp(&comp_array, val);
            }
        });

        let with_simd_duration = time_execution(|| {
            for val in &store_values {
                euclidean_distance(&comp_array, val);
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

        let mut most_similar_result: Vec<(&'static str, f32)> = vec![];

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

        let mut most_similar_result: Vec<(&'static str, f32)> = vec![];

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
