use pulp::{Arch, Simd, WithSimd};

use crate::{DistanceFn, LinearAlgorithm};

/// A value where **smaller means closer**.
///
/// Deliberately not orderable. Ordering goes through [`Closeness`], the only type here
/// that knows which direction is closer.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Distance(f32);

/// A value where **larger means closer**.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Similarity(f32);

impl Distance {
    #[inline]
    pub fn value(self) -> f32 {
        self.0
    }
}

impl Similarity {
    #[inline]
    pub fn value(self) -> f32 {
        self.0
    }
}

/// Ordering key: **greater is always closer**, whatever the metric.
///
/// The `From` impls below are the only way to build one, so a raw similarity cannot
/// reach a comparison without its direction applied first.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(transparent)]
pub struct Closeness(f32);

impl Eq for Closeness {}

impl PartialOrd for Closeness {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Closeness {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl From<Distance> for Closeness {
    /// Negated rather than `1.0 - d`: negation is exact in floating point and works
    /// for unbounded values.
    #[inline]
    fn from(distance: Distance) -> Self {
        Closeness(-distance.0)
    }
}

impl From<Similarity> for Closeness {
    #[inline]
    fn from(similarity: Similarity) -> Self {
        Closeness(similarity.0)
    }
}

/// What a metric computed, carrying which direction means closer.
///
/// The variant is fixed by the return type of the function that produced it, so a
/// metric cannot disagree with itself about its own direction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Score {
    Distance(Distance),
    Similarity(Similarity),
}

impl Score {
    /// The metric's own number, as reported to callers. Cosine stays a similarity.
    #[inline]
    pub fn value(self) -> f32 {
        match self {
            Score::Distance(distance) => distance.value(),
            Score::Similarity(similarity) => similarity.value(),
        }
    }

    #[inline]
    pub fn closeness(self) -> Closeness {
        match self {
            Score::Distance(distance) => distance.into(),
            Score::Similarity(similarity) => similarity.into(),
        }
    }
}

impl From<Distance> for Score {
    #[inline]
    fn from(distance: Distance) -> Self {
        Score::Distance(distance)
    }
}

impl From<Similarity> for Score {
    #[inline]
    fn from(similarity: Similarity) -> Self {
        Score::Similarity(similarity)
    }
}

impl DistanceFn for LinearAlgorithm {
    #[inline]
    fn score(&self, a: &[f32], b: &[f32]) -> Score {
        match self {
            Self::EuclideanDistance => euclidean_distance(a, b).into(),
            Self::CosineSimilarity => cosine_similarity(a, b).into(),
            Self::DotProductSimilarity => dot_product(a, b).into(),
        }
    }
}

/// COSINE SIMILARITY
///
/// Cosine similarity is the cosine of the angle between two vectors.
/// It measures how similar two vector points are.
/// It is scale-invariant, meaning it is unaffected by the length of the vectors.
///
/// The cosine of the angle shows whether vectors are similar,
/// dissimilar, or orthogonal (independent).
///
/// The range of cosine similarity is from -1 to 1:
/// - 1  → identical direction (most similar)
/// - -1 → opposite direction
/// - 0  → orthogonal (independent)
///
/// To calculate cosine similarity between two vectors:
/// - Calculate the dot product of both vectors.
/// - Compute the magnitude of each vector.
///   The magnitude of a vector is calculated using the Pythagorean theorem:
///   sqrt(A^2 + B^2)
///   where A and B are vector components.
/// - Divide the dot product by the product of the magnitudes:
///   cos(θ) = A · B / (||A|| * ||B||)
///
/// Where the magnitude is:
///   ||A|| = sqrt(a1^2 + a2^2 + ... + an^2)
///
/// An implementation for retrieving the most similar items can use a MaxHeap,
/// since we are looking for values closest to 1.
/// Smaller angular distance between vectors denotes higher similarity.
///
/// Fused SIMD kernel: computes the dot product and both magnitudes
/// in a single pass. This avoids iterating over the vectors twice
/// (once for dot product and once for magnitude),
/// improving cache locality and reducing SIMD dispatch overhead.
struct CosineSimilarityKernel<'a> {
    first: &'a [f32],
    second: &'a [f32],
}

impl WithSimd for CosineSimilarityKernel<'_> {
    type Output = f32;

    #[inline(always)]
    fn with_simd<S: Simd>(self, simd: S) -> Self::Output {
        let (first_head, first_tail) = S::as_simd_f32s(self.first);
        let (second_head, second_tail) = S::as_simd_f32s(self.second);

        let mut dot = simd.splat_f32s(0.0);
        let mut mag_first = simd.splat_f32s(0.0);
        let mut mag_second = simd.splat_f32s(0.0);

        for (&a, &b) in first_head.iter().zip(second_head) {
            dot = simd.mul_add_f32s(a, b, dot);
            mag_first = simd.mul_add_f32s(a, a, mag_first);
            mag_second = simd.mul_add_f32s(b, b, mag_second);
        }

        let mut dot_sum = simd.reduce_sum_f32s(dot);
        let mut mag_first_sum = simd.reduce_sum_f32s(mag_first);
        let mut mag_second_sum = simd.reduce_sum_f32s(mag_second);

        for (&x, &y) in first_tail.iter().zip(second_tail) {
            dot_sum += x * y;
            mag_first_sum += x * x;
            mag_second_sum += y * y;
        }

        dot_sum / (mag_first_sum.sqrt() * mag_second_sum.sqrt())
    }
}

#[tracing::instrument(skip_all)]
pub fn cosine_similarity(first: &[f32], second: &[f32]) -> Similarity {
    assert_eq!(
        first.len(),
        second.len(),
        "Vectors must have the same length!"
    );

    let arch = Arch::new();
    Similarity(arch.dispatch(CosineSimilarityKernel { first, second }))
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
pub fn dot_product(first: &[f32], second: &[f32]) -> Similarity {
    assert_eq!(
        first.len(),
        second.len(),
        "Vectors must have the same length!"
    );

    let arch = Arch::new();
    Similarity(arch.dispatch(DotProduct { first, second }))
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
pub fn euclidean_distance(first: &[f32], second: &[f32]) -> Distance {
    // Calculate the sum of squared differences for each dimension
    assert_eq!(
        first.len(),
        second.len(),
        "Vectors must have the same length!"
    );

    let arch = Arch::new();

    Distance(arch.dispatch(EuclideanDistance { first, second }))
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
