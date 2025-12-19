use pulp::{Arch, Simd, WithSimd};

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
///            cos(0) = A.B / ||A||.||B||
///
///     where Magnitude(A or B):
///     ||A|| = sqrt[(1)^2 + (2)^2]
///     
///
///  An Implementation for most similar items would be a MaxHeap.
///  We are looking the closest number to one meaning Max
///  The smaller the distance between two points, denotes higher similarity.
#[tracing::instrument(skip_all)]
pub fn cosine_similarity(first: &[f32], second: &[f32]) -> f32 {
    assert_eq!(
        first.len(),
        second.len(),
        "Vectors must have the same length!"
    );

    let dot = dot_product(first, second);

    let arch = Arch::new();
    let magnitude = arch.dispatch(Magnitude { first, second });

    dot / magnitude
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
pub fn dot_product(first: &[f32], second: &[f32]) -> f32 {
    assert_eq!(
        first.len(),
        second.len(),
        "Vectors must have the same length!"
    );

    let arch = Arch::new();
    arch.dispatch(DotProduct { first, second })
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
pub fn euclidean_distance(first: &[f32], second: &[f32]) -> f32 {
    // Calculate the sum of squared differences for each dimension
    assert_eq!(
        first.len(),
        second.len(),
        "Vectors must have the same length!"
    );

    let arch = Arch::new();

    arch.dispatch(EuclideanDistance { first, second })
}

/// Squared Euclidean distance (without the sqrt) - useful for KD-trees and other
/// algorithms where relative ordering is more important than absolute distance
#[tracing::instrument(skip_all)]
pub fn squared_euclidean_distance(first: &[f32], second: &[f32]) -> f32 {
    assert_eq!(
        first.len(),
        second.len(),
        "Vectors must have the same length!"
    );

    first
        .iter()
        .zip(second.iter())
        .map(|(a, b)| {
            let diff = a - b;
            diff * diff
        })
        .sum()
}
