use ndarray::prelude::*;
use std::ops::Deref;
use types::similarity::Algorithm;

type SimFuncSig = fn(
    &ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>,
    &ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>,
) -> f64;

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

fn cosine_similarity(
    first: &ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>,
    second: &ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>,
) -> f64 {
    // formular = dot product of vectors / product of the magnitude of the vectors
    // maginiture of a vector can be calcuated using pythagoras theorem.
    // sqrt of sum of vector values
    // for magnitutude of A( ||A||) =[1,2]
    // sqrt[(1)^2 + (2)^2]
    //
    //

    let dot_product = dot_product(&first, &second);

    // the magnitude can be calculated using the arr.norm method.
    //

    let mag_first = &first
        .iter()
        .map(|x| x * x)
        .map(|x| x as f64)
        .sum::<f64>()
        .sqrt();

    let mag_second = &second
        .iter()
        .map(|x| x * x)
        .map(|x| x as f64)
        .sum::<f64>()
        .sqrt();

    let cosine_sim = dot_product / (mag_first * mag_second);

    cosine_sim
}
// for mulitplication of a matrix, the number of columns in the first matrix == the number of rows
// in the second
// A.B != B.A

fn dot_product(
    first: &ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>,
    second: &ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>,
) -> f64 {
    // transpose if the shapes are equal that is
    // check that for a A = m*n and B = p * q
    // m = q and n = q
    // else check that it's already transposed

    if first.shape() != second.shape() {
        panic!("Incorrect dimensions, both vectors must match");
    }

    let dot_product = second.dot(&first.t());

    dot_product
}

fn euclidean_distance(
    first: &ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>,
    second: &ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>,
) -> f64 {
    // Calculate the sum of squared differences for each dimension
    let mut sum_of_squared_differences = 0.0;
    for (&coord1, &coord2) in first.iter().zip(second.iter()) {
        let diff = coord1 - coord2;
        sum_of_squared_differences += diff * diff;
    }

    // Calculate the square root of the sum of squared differences
    let distance = f64::sqrt(sum_of_squared_differences);
    distance
}
