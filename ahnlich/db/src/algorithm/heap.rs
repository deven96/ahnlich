#![allow(dead_code)]
use super::LinearAlgorithm;

pub enum HeapOrder {
    Min,
    Max,
}

impl From<&LinearAlgorithm> for HeapOrder {
    fn from(value: &LinearAlgorithm) -> Self {
        match value {
            LinearAlgorithm::EuclideanDistance => HeapOrder::Min,
            LinearAlgorithm::CosineSimilarity | LinearAlgorithm::DotProductSimilarity => {
                HeapOrder::Max
            }
        }
    }
}
