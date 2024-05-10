mod heap;
mod similarity;

use ndarray::prelude::*;

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
        (value.0 .0, value.0 .1)
    }
}

impl<'a> PartialEq for SimilarityVectorF64<'a> {
    fn eq(&self, other: &Self) -> bool {
        *(self.0 .0) == *(other.0 .0)
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
        (self.0).1.partial_cmp(&(other.0).1).unwrap()
    }
}

trait KNearestN {}
