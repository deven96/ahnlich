use super::LinearAlgorithm;

// Re-export bounded heaps from similarity crate
pub use ahnlich_similarity::heap::{BoundedMaxHeap, BoundedMinHeap};

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
