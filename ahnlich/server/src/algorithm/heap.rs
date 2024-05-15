use super::SimilarityVector;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::num::NonZeroUsize;
use types::keyval::StoreKey;
use types::similarity::Algorithm;

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
    pub(crate) fn len(&self) -> usize {
        self.heap.len()
    }
    pub(crate) fn push(&mut self, item: SimilarityVector<'a>) {
        self.heap.push(Reverse(item));
    }
    pub(crate) fn pop(&mut self) -> Option<SimilarityVector<'a>> {
        self.heap.pop().map(|popped_item| popped_item.0)
    }

    pub(crate) fn output(&mut self) -> Vec<(&'a StoreKey, f64)> {
        let mut result: Vec<_> = vec![];

        loop {
            match self.pop() {
                Some(value) if result.len() < self.max_capacity.into() => {
                    let vector_sim = value.0;
                    result.push((vector_sim.0, vector_sim.1));
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
    fn push(&mut self, item: SimilarityVector<'a>) {
        self.heap.push(item);
    }
    pub(crate) fn pop(&mut self) -> Option<SimilarityVector<'a>> {
        self.heap.pop()
    }
    pub(crate) fn len(&self) -> usize {
        self.heap.len()
    }

    fn output(&mut self) -> Vec<(&'a StoreKey, f64)> {
        let mut result: Vec<_> = vec![];

        loop {
            match self.heap.pop() {
                Some(value) if result.len() < self.max_capacity.into() => {
                    let vector_sim = value.0;
                    result.push((vector_sim.0, vector_sim.1));
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
    pub(crate) fn push(&mut self, item: SimilarityVector<'a>) {
        match self {
            Self::Max(h) => h.push(item),
            Self::Min(h) => h.push(item),
        }
    }
    pub(crate) fn pop(&mut self) -> Option<SimilarityVector<'a>> {
        match self {
            Self::Max(h) => h.pop(),
            Self::Min(h) => h.pop(),
        }
    }

    pub(crate) fn output(&mut self) -> Vec<(&'a StoreKey, f64)> {
        match self {
            Self::Min(h) => h.output(),
            Self::Max(h) => h.output(),
        }
    }
}

impl From<(&Algorithm, NonZeroUsize)> for AlgorithmHeapType<'_> {
    fn from((value, capacity): (&Algorithm, NonZeroUsize)) -> Self {
        match value {
            Algorithm::EuclideanDistance => AlgorithmHeapType::Min(MinHeap::new(capacity)),
            Algorithm::CosineSimilarity | Algorithm::DotProductSimilarity => {
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
        let first_vector = StoreKey(ndarray::Array1::<f64>::zeros(2).map(|x| x + 2.0));

        // If we pop these scores now, they should come back in the reverse order.
        while count < 5.0 {
            let similarity: f64 = 1.0 + count;

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
        let first_vector = StoreKey(ndarray::Array1::<f64>::zeros(2).map(|x| x + 2.0));

        // If we pop these scores now, they should come back  the right order(max first).
        while count < 5.0 {
            let similarity: f64 = 1.0 + count;
            let item: SimilarityVector = (&first_vector, similarity).into();

            heap.push(item);

            count += 1.0;
        }

        assert_eq!(heap.pop(), Some((&first_vector, 5.0).into()));
        assert_eq!(heap.pop(), Some((&first_vector, 4.0).into()));
        assert_eq!(heap.pop(), Some((&first_vector, 3.0).into()));
    }
}
