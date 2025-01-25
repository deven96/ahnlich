#![allow(dead_code)]
use super::LinearAlgorithm;
use super::SimilarityVector;
use ahnlich_types::keyval::StoreKey;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::num::NonZeroUsize;

pub(crate) struct MinHeap<'a> {
    max_capacity: NonZeroUsize,
    heap: BinaryHeap<Reverse<SimilarityVector<'a>>>,
}

impl<'a> MinHeap<'a> {
    pub(crate) fn new(capacity: NonZeroUsize) -> Self {
        Self {
            heap: BinaryHeap::new(),
            max_capacity: capacity,
        }
    }
    #[tracing::instrument(skip_all)]
    pub(crate) fn len(&self) -> usize {
        self.heap.len()
    }
    #[tracing::instrument(skip_all)]
    pub(crate) fn push(&mut self, item: SimilarityVector<'a>) {
        self.heap.push(Reverse(item));
    }
    #[tracing::instrument(skip_all)]
    pub(crate) fn pop(&mut self) -> Option<SimilarityVector<'a>> {
        self.heap.pop().map(|popped_item| popped_item.0)
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn output(&mut self) -> Vec<(StoreKey, f32)> {
        let mut result: Vec<_> = Vec::with_capacity(self.max_capacity.get());

        loop {
            match self.pop() {
                Some(value) if result.len() < self.max_capacity.get() => {
                    let vector_sim = value.0;
                    result.push((vector_sim.0.clone(), vector_sim.1));
                }
                _ => break,
            }
        }
        result
    }
}

pub(crate) struct MaxHeap<'a> {
    max_capacity: NonZeroUsize,
    heap: BinaryHeap<SimilarityVector<'a>>,
}

impl<'a> MaxHeap<'a> {
    pub(crate) fn new(capacity: NonZeroUsize) -> Self {
        Self {
            heap: BinaryHeap::new(),
            max_capacity: capacity,
        }
    }
    #[tracing::instrument(skip_all)]
    fn push(&mut self, item: SimilarityVector<'a>) {
        self.heap.push(item);
    }
    #[tracing::instrument(skip_all)]
    pub(crate) fn pop(&mut self) -> Option<SimilarityVector<'a>> {
        self.heap.pop()
    }
    #[tracing::instrument(skip_all)]
    pub(crate) fn len(&self) -> usize {
        self.heap.len()
    }

    #[tracing::instrument(skip_all)]
    fn output(&mut self) -> Vec<(StoreKey, f32)> {
        let mut result: Vec<_> = Vec::with_capacity(self.max_capacity.get());

        loop {
            match self.heap.pop() {
                Some(value) if result.len() < self.max_capacity.get() => {
                    let vector_sim = value.0;
                    result.push((vector_sim.0.clone(), vector_sim.1));
                }
                _ => break,
            }
        }
        result
    }
}

pub(crate) enum AlgorithmHeapType<'a> {
    Min(MinHeap<'a>),
    Max(MaxHeap<'a>),
}

impl<'a> AlgorithmHeapType<'a> {
    #[tracing::instrument(skip_all)]
    pub(crate) fn push(&mut self, item: SimilarityVector<'a>) {
        match self {
            Self::Max(h) => h.push(item),
            Self::Min(h) => h.push(item),
        }
    }
    #[tracing::instrument(skip_all)]
    pub(crate) fn pop(&mut self) -> Option<SimilarityVector<'a>> {
        match self {
            Self::Max(h) => h.pop(),
            Self::Min(h) => h.pop(),
        }
    }

    #[tracing::instrument(skip_all)]
    pub(crate) fn output(&mut self) -> Vec<(StoreKey, f32)> {
        match self {
            Self::Min(h) => h.output(),
            Self::Max(h) => h.output(),
        }
    }
}

impl From<(&LinearAlgorithm, NonZeroUsize)> for AlgorithmHeapType<'_> {
    fn from((value, capacity): (&LinearAlgorithm, NonZeroUsize)) -> Self {
        match value {
            LinearAlgorithm::EuclideanDistance => AlgorithmHeapType::Min(MinHeap::new(capacity)),
            LinearAlgorithm::CosineSimilarity | LinearAlgorithm::DotProductSimilarity => {
                AlgorithmHeapType::Max(MaxHeap::new(capacity))
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_min_heap_ordering_works() {
        let mut heap = MinHeap::new(NonZeroUsize::new(3).unwrap());
        let mut count = 0.0;
        let first_vector = StoreKey(vec![2.0, 2.0]);

        // If we pop these scores now, they should come back in the reverse order.
        while count < 5.0 {
            let similarity: f32 = 1.0 + count;

            let item: SimilarityVector = (&first_vector, similarity).into();

            heap.push(item);

            count += 1.0;
        }

        assert_eq!(heap.pop(), Some((&first_vector, 1.0).into()));
        assert_eq!(heap.pop(), Some((&first_vector, 2.0).into()));
        assert_eq!(heap.pop(), Some((&first_vector, 3.0).into()));
    }

    #[test]
    fn test_max_heap_ordering_works() {
        let mut heap = MaxHeap::new(NonZeroUsize::new(3).unwrap());
        let mut count = 0.0;
        let first_vector = StoreKey(vec![2.0, 2.0]);

        // If we pop these scores now, they should come back  the right order(max first).
        while count < 5.0 {
            let similarity: f32 = 1.0 + count;
            let item: SimilarityVector = (&first_vector, similarity).into();

            heap.push(item);

            count += 1.0;
        }

        assert_eq!(heap.pop(), Some((&first_vector, 5.0).into()));
        assert_eq!(heap.pop(), Some((&first_vector, 4.0).into()));
        assert_eq!(heap.pop(), Some((&first_vector, 3.0).into()));
    }
}
