use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::num::NonZeroUsize;

/// A bounded max heap that maintains at most `capacity` elements.
/// When the heap is full, incoming elements replace the smallest element if they are larger.
pub struct BoundedMaxHeap<T: Ord> {
    heap: BinaryHeap<Reverse<T>>,
    capacity: usize,
}

impl<T: Ord> BoundedMaxHeap<T> {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity.get()),
            capacity: capacity.get(),
        }
    }

    /// Insert an element into the bounded heap.
    /// If the heap is at capacity and the new element is larger than the smallest,
    /// the smallest element is removed and the new element is added.
    pub fn push(&mut self, item: T) {
        if self.heap.len() < self.capacity {
            self.heap.push(Reverse(item));
        } else if let Some(mut smallest) = self.heap.peek_mut() {
            // Only insert if the new item is better (larger) than the worst (smallest) item
            if item > smallest.0 {
                *smallest = Reverse(item);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn peek(&self) -> Option<&T> {
        self.heap.peek().map(|r| &r.0)
    }

    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop().map(|r| r.0)
    }

    /// Convert the bounded heap into a Vec, consuming the heap.
    /// Elements are returned in descending order (largest first).
    pub fn into_sorted_vec(self) -> Vec<T> {
        let mut vec: Vec<_> = self.heap.into_iter().map(|r| r.0).collect();
        vec.sort_by(|a, b| b.cmp(a)); // Sort descending
        vec
    }

    /// Get an iterator over the heap elements (unordered)
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.heap.iter().map(|r| &r.0)
    }
}

/// A bounded min heap that maintains at most `capacity` elements.
/// When the heap is full, incoming elements replace the largest element if they are smaller.
pub struct BoundedMinHeap<T: Ord> {
    heap: BinaryHeap<T>,
    capacity: usize,
}

impl<T: Ord> BoundedMinHeap<T> {
    pub fn new(capacity: NonZeroUsize) -> Self {
        Self {
            heap: BinaryHeap::with_capacity(capacity.get()),
            capacity: capacity.get(),
        }
    }

    /// Insert an element into the bounded heap.
    /// If the heap is at capacity and the new element is smaller than the largest,
    /// the largest element is removed and the new element is added.
    pub fn push(&mut self, item: T) {
        if self.heap.len() < self.capacity {
            self.heap.push(item);
        } else if let Some(mut largest) = self.heap.peek_mut() {
            // Only insert if the new item is better (smaller) than the worst (largest) item
            if item < *largest {
                *largest = item;
            }
        }
    }

    pub fn len(&self) -> usize {
        self.heap.len()
    }

    pub fn is_empty(&self) -> bool {
        self.heap.is_empty()
    }

    pub fn peek(&self) -> Option<&T> {
        self.heap.peek()
    }

    pub fn pop(&mut self) -> Option<T> {
        self.heap.pop()
    }

    /// Convert the bounded heap into a Vec, consuming the heap.
    /// Elements are returned in ascending order (smallest first).
    pub fn into_sorted_vec(self) -> Vec<T> {
        let mut vec: Vec<_> = self.heap.into_iter().collect();
        vec.sort(); // Sort ascending
        vec
    }

    /// Get an iterator over the heap elements (unordered)
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.heap.iter()
    }
}
