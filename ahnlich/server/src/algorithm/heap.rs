use super::SimilarityVectorF64;
use ndarray::prelude::*;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::num::NonZeroUsize;
use types::similarity::Algorithm;

pub(crate) struct MinHeap<'a> {
    max_capacity: NonZeroUsize,
    heap: BinaryHeap<Reverse<SimilarityVectorF64<'a>>>,
}

impl<'a> MinHeap<'a> {
    pub(crate) fn new(capacity: NonZeroUsize) -> Self {
        Self {
            heap: BinaryHeap::new(),
            max_capacity: capacity,
        }
    }
    pub(crate) fn len(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.heap.len()).unwrap()
    }
    pub(crate) fn push(&mut self, item: SimilarityVectorF64<'a>) {
        self.heap.push(Reverse(item));
    }
    pub(crate) fn pop(&mut self) -> Option<SimilarityVectorF64<'a>> {
        self.heap.pop().map(|popped_item| popped_item.0)
    }

    pub(crate) fn output(
        &mut self,
    ) -> Vec<(&'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>, f64)> {
        let mut result: Vec<_> = vec![];

        loop {
            match self.pop() {
                Some(value) if NonZeroUsize::new(result.len()).unwrap() < self.max_capacity => {
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
    heap: BinaryHeap<SimilarityVectorF64<'a>>,
}

impl<'a> MaxHeap<'a> {
    pub(crate) fn new(capacity: NonZeroUsize) -> Self {
        Self {
            heap: BinaryHeap::new(),
            max_capacity: capacity,
        }
    }
    fn push(&mut self, item: SimilarityVectorF64<'a>) {
        self.heap.push(item);
    }
    pub(crate) fn pop(&mut self) -> Option<SimilarityVectorF64<'a>> {
        self.heap.pop()
    }
    pub(crate) fn len(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.heap.len()).unwrap()
    }

    fn output(&mut self) -> Vec<(&'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>, f64)> {
        let mut result: Vec<_> = vec![];

        loop {
            match self.heap.pop() {
                Some(value) if NonZeroUsize::new(result.len()).unwrap() < self.max_capacity => {
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
    MIN(MinHeap<'a>),
    MAX(MaxHeap<'a>),
}

impl<'a> AlgorithmHeapType<'a> {
    pub(crate) fn push(&mut self, item: SimilarityVectorF64<'a>) {
        match self {
            Self::MAX(h) => h.push(item),
            Self::MIN(h) => h.push(item),
        }
    }
    pub(crate) fn pop(&mut self) -> Option<SimilarityVectorF64<'a>> {
        match self {
            Self::MAX(h) => h.pop(),
            Self::MIN(h) => h.pop(),
        }
    }

    pub(crate) fn output(
        &mut self,
    ) -> Vec<(&'a ndarray::ArrayBase<ndarray::OwnedRepr<f64>, Ix1>, f64)> {
        match self {
            Self::MIN(h) => h.output(),
            Self::MAX(h) => h.output(),
        }
    }
}

impl From<(&Algorithm, NonZeroUsize)> for AlgorithmHeapType<'_> {
    fn from((value, capacity): (&Algorithm, NonZeroUsize)) -> Self {
        match value {
            Algorithm::EuclideanDistance => AlgorithmHeapType::MIN(MinHeap::new(capacity)),
            Algorithm::CosineSimilarity | Algorithm::DotProductSimilarity => {
                AlgorithmHeapType::MAX(MaxHeap::new(capacity))
            }
        }
    }
}
