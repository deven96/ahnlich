/// K Dimensional Tree algorithm is a binary search tree that extends to multiple dimensions,
/// making it an efficient datastructure for applying nearest neighbour searches and range searches
use crate::error::Error;
use crossbeam::epoch::{self, Atomic, Guard, Owned, Shared};
use ndarray::Array1;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::cmp::Ordering as CmpOrdering;
use std::cmp::Reverse;
use std::collections::BinaryHeap;
use std::num::NonZeroUsize;
use std::sync::atomic::Ordering;

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KDNode {
    point: Array1<f32>,
    left: Atomic<KDNode>,
    right: Atomic<KDNode>,
}

impl KDNode {
    pub fn new(point: Array1<f32>) -> Self {
        Self {
            point,
            left: Atomic::null(),
            right: Atomic::null(),
        }
    }
}

// Internal structure to sort array by second field which is similarity score
#[derive(Debug)]
struct OrderedArray(Array1<f32>, f32);

impl PartialEq for OrderedArray {
    fn eq(&self, other: &Self) -> bool {
        self.1 == other.1
    }
}

impl Eq for OrderedArray {}

impl PartialOrd for OrderedArray {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderedArray {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.1
            .partial_cmp(&other.1)
            .unwrap_or(std::cmp::Ordering::Less)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct KDTree {
    root: Atomic<KDNode>,
    dimension: NonZeroUsize,
}

impl KDTree {
    /// initialize KDTree with a specified nonzero dimension
    /// Every point within the tree is expected to then match the specified dimension
    pub fn new(dimension: NonZeroUsize) -> Self {
        Self {
            root: Atomic::null(),
            dimension,
        }
    }

    fn assert_shape(&self, input: &Array1<f32>) -> Result<(), Error> {
        let dim = self.dimension.get();
        if [dim] != input.shape() {
            return Err(Error::DimensionMisMatch {
                expected: dim,
                found: input.shape()[0],
            });
        }
        Ok(())
    }

    /// insert new point into the KDTree by recursively walking through each dimension of the
    /// point. This asserts that the one-dimensional array being passed in here conforms to the
    /// shape specified by dimension else a dimension mismatch error is returned
    pub fn insert(&self, point: Array1<f32>) -> Result<(), Error> {
        self.assert_shape(&point)?;
        let guard = epoch::pin();
        self.insert_recursive(&self.root, point, 0, &guard);
        Ok(())
    }

    fn insert_recursive(
        &self,
        node: &Atomic<KDNode>,
        point: Array1<f32>,
        depth: usize,
        guard: &Guard,
    ) {
        let dim = depth % self.dimension.get();
        loop {
            match node.compare_exchange(
                Shared::null(),
                Shared::null(),
                Ordering::AcqRel,
                Ordering::Acquire,
                guard,
            ) {
                // node is null i.e does not exist so we create it
                Ok(shared) => {
                    let new_node = Box::new(KDNode::new(point.clone()));
                    let new_node_ptr = Owned::from(new_node);
                    // successfully created new node else keep spinning
                    if node
                        .compare_exchange(
                            shared,
                            new_node_ptr,
                            Ordering::AcqRel,
                            Ordering::Acquire,
                            guard,
                        )
                        .is_ok()
                    {
                        break;
                    }
                }
                // node already exists so compare dimensions and insert left or right
                Err(shared) => {
                    let current = unsafe { shared.current.deref() };
                    // if they are exactly the same then no need to append to tree
                    if point == current.point {
                        break;
                    }
                    match point[dim]
                        .partial_cmp(&current.point[dim])
                        .expect("Partial cmp does not exist")
                    {
                        CmpOrdering::Less => {
                            self.insert_recursive(&current.left, point, depth + 1, guard);
                            break;
                        }
                        _ => {
                            self.insert_recursive(&current.right, point, depth + 1, guard);
                            break;
                        }
                    }
                }
            }
        }
    }

    /// Delete an entry matching delete_point from KD tree
    pub fn delete(&self, delete_point: &Array1<f32>) -> Result<Option<Array1<f32>>, Error> {
        self.assert_shape(delete_point)?;
        let guard = epoch::pin();
        Ok(self.delete_recursive(&self.root, delete_point, 0, &guard))
    }

    fn delete_recursive(
        &self,
        node: &Atomic<KDNode>,
        delete_point: &Array1<f32>,
        depth: usize,
        guard: &Guard,
    ) -> Option<Array1<f32>> {
        let dim = depth % self.dimension.get();

        match node.load(Ordering::Acquire, guard) {
            empty if empty == Shared::null() => None,
            shared => {
                let current = unsafe { shared.deref() };
                // found node to delete
                if current.point == delete_point {
                    if current.right.load(Ordering::Acquire, guard).is_null() {
                        // Replace current with left node
                        let left_child = current.left.swap(Shared::null(), Ordering::AcqRel, guard);
                        node.store(left_child, Ordering::Release);
                        Some(current.point.clone())
                    } else if current.left.load(Ordering::Acquire, guard).is_null() {
                        // Replace current with right node
                        let right_child =
                            current.right.swap(Shared::null(), Ordering::AcqRel, guard);
                        node.store(right_child, Ordering::Release);
                        return Some(current.point.clone());
                    } else {
                        // Node has both children not null, so we need to find minimum successor to
                        // replace current
                        let successor = Self::find_min(&current.right, guard);
                        let successor_point = unsafe { successor.deref().point.clone() };
                        let new_right = self.delete_recursive(
                            &current.right,
                            &successor_point,
                            depth + 1,
                            guard,
                        );
                        let new_point = Owned::new(KDNode {
                            point: successor_point.clone(),
                            left: current.left.clone(),
                            right: Atomic::null(),
                        });
                        let new_right = new_right
                            .map(|right| Owned::new(KDNode::new(right)).into_shared(guard))
                            .unwrap_or(Shared::null());
                        new_point.right.store(new_right, Ordering::Release);
                        return Some(successor_point);
                    }
                } else if delete_point[dim] < current.point[dim] {
                    let left_child =
                        self.delete_recursive(&current.left, delete_point, depth + 1, guard);
                    if let Some(left_child) = left_child {
                        // find minimum of the left child to replace current left
                        let successor = Self::find_min(&current.left, guard);
                        current.left.store(successor, Ordering::Release);
                        return Some(left_child);
                    }
                    return None;
                } else {
                    let right_child =
                        self.delete_recursive(&current.right, delete_point, depth + 1, guard);
                    if let Some(right_child) = right_child {
                        // find minimum of the right child to replace current right
                        let successor = Self::find_min(&current.right, guard);
                        current.right.store(successor, Ordering::Release);
                        return Some(right_child);
                    }
                    return None;
                }
            }
        }
    }

    fn find_min<'a>(node: &Atomic<KDNode>, guard: &'a Guard) -> Shared<'a, KDNode> {
        match node.load(Ordering::Acquire, guard) {
            empty if empty == Shared::null() => Shared::null(),
            shared => {
                let current = unsafe { shared.deref() };
                if current.left.load(Ordering::Acquire, guard).is_null() {
                    // there is no lesser left to go
                    shared
                } else {
                    Self::find_min(&current.left, guard)
                }
            }
        }
    }

    /// Returns the N nearest arrays to the reference point
    pub fn n_nearest(
        &self,
        reference_point: &Array1<f32>,
        n: NonZeroUsize,
    ) -> Result<Vec<(Array1<f32>, f32)>, Error> {
        self.assert_shape(reference_point)?;
        let guard = epoch::pin();
        let mut heap = BinaryHeap::new();
        self.n_nearest_recursive(&self.root, reference_point, 0, n, &guard, &mut heap);
        let mut results = Vec::with_capacity(n.get());
        while let Some(Reverse(OrderedArray(val, distance))) = heap.pop() {
            results.push((val, distance));
            if results.len() == n.get() {
                break;
            }
        }
        Ok(results)
    }

    fn n_nearest_recursive(
        &self,
        node: &Atomic<KDNode>,
        reference_point: &Array1<f32>,
        depth: usize,
        n: NonZeroUsize,
        guard: &Guard,
        heap: &mut BinaryHeap<Reverse<OrderedArray>>,
    ) {
        if let Some(shared) = unsafe { node.load(Ordering::Acquire, guard).as_ref() } {
            let distance = self.squared_distance(reference_point, &shared.point);
            if heap.len() < n.get() {
                heap.push(Reverse(OrderedArray(shared.point.clone(), distance)));
            } else if let Some(Reverse(OrderedArray(_, max_distance))) = heap.peek() {
                if distance < *max_distance {
                    heap.pop();
                    heap.push(Reverse(OrderedArray(shared.point.clone(), distance)));
                }
            }

            let dim = depth % self.dimension.get();
            let go_left_first = reference_point[dim] < shared.point[dim];
            if go_left_first {
                self.n_nearest_recursive(&shared.left, reference_point, depth + 1, n, guard, heap);
                if heap.len() < n.get()
                    || (reference_point[dim] - shared.point[dim]).abs()
                        < heap.peek().map_or(f32::INFINITY, |x| x.0 .1)
                {
                    self.n_nearest_recursive(
                        &shared.right,
                        reference_point,
                        depth + 1,
                        n,
                        guard,
                        heap,
                    );
                }
            } else {
                self.n_nearest_recursive(&shared.right, reference_point, depth + 1, n, guard, heap);
                if heap.len() < n.get()
                    || (reference_point[dim] - shared.point[dim]).abs()
                        < heap.peek().map_or(f32::INFINITY, |x| x.0 .1)
                {
                    self.n_nearest_recursive(
                        &shared.left,
                        reference_point,
                        depth + 1,
                        n,
                        guard,
                        heap,
                    );
                }
            }
        }
    }

    fn squared_distance(&self, a: &Array1<f32>, b: &Array1<f32>) -> f32 {
        a.iter()
            .zip(b.iter())
            .map(|(ai, bi)| (ai - bi).powi(2))
            .sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;
    use ndarray::Array;
    use pretty_assertions::assert_eq;
    use std::sync::Arc;

    #[test]
    fn test_simple_insert_multithread() {
        let dimension = 5;
        let kdtree = Arc::new(KDTree::new(NonZeroUsize::new(dimension).unwrap()));
        let handlers = (0..3).map(|_| {
            let tree = kdtree.clone();
            let dimension = dimension.clone();
            std::thread::spawn(move || {
                let random =
                    Array::from((0..dimension).map(|_| rand::random()).collect::<Vec<f32>>());
                tree.insert(random)
            })
        });

        for handler in handlers {
            let _ = handler.join().unwrap();
        }

        let res = kdtree.n_nearest(&array![1.0, 2.0], NonZeroUsize::new(5).unwrap());
        // should error as dimension does not match
        assert!(res.is_err());
        let res = kdtree
            .n_nearest(
                &array![1.0, 2.0, 3.0, 4.0, 5.0],
                NonZeroUsize::new(5).unwrap(),
            )
            .unwrap();
        // only 3 things were inserted so we expect 3
        assert_eq!(res.len(), 3);
    }

    #[test]
    fn test_results_are_accurate() {
        let dimension = NonZeroUsize::new(3).unwrap();
        let closest_n = NonZeroUsize::new(1).unwrap();
        let kdtree = Arc::new(KDTree::new(dimension));
        kdtree.insert(array![1.0, 2.0, 3.0]).unwrap();
        kdtree.insert(array![1.1, 2.2, 3.3]).unwrap();
        kdtree.insert(array![1.2, 2.3, 3.1]).unwrap();
        kdtree.insert(array![1.3, 2.1, 3.2]).unwrap();
        // should not insert twice
        kdtree.insert(array![1.3, 2.1, 3.2]).unwrap();

        // Exact matches
        let res = kdtree.n_nearest(&array![1.0, 2.0, 3.0], closest_n).unwrap();
        assert_eq!(res, vec![(array![1.0, 2.0, 3.0], 0.0)]);
        let res = kdtree.n_nearest(&array![1.3, 2.1, 3.2], closest_n).unwrap();
        assert_eq!(res, vec![(array![1.3, 2.1, 3.2], 0.0)]);

        // Close matches
        let res = kdtree.n_nearest(&array![1.3, 2.1, 3.0], closest_n).unwrap();
        assert_eq!(res, vec![(array![1.3, 2.1, 3.2], 0.040000018)]);

        // check insertion length remained 4 despite 4 inserts
        let res = kdtree
            .n_nearest(&array![1.3, 2.1, 3.2], NonZeroUsize::new(5).unwrap())
            .unwrap();
        assert_eq!(res.len(), 4);
    }

    // drawback of KDTree is that it compares across axes and potentially misses closer matches in
    // euclidean distance that are located in another axis
    //    #[test]
    //    fn test_branch_works_to_check_combinations() {
    //        let dimension = NonZeroUsize::new(3).unwrap();
    //        let closest_n = NonZeroUsize::new(2).unwrap();
    //        let kdtree = Arc::new(KDTree::new(dimension));
    //        kdtree.insert(array![1.0, 2.0, 3.0]).unwrap();
    //        kdtree.insert(array![1.1, 2.0, 3.0]).unwrap();
    //        kdtree.insert(array![0.6, 2.0, 3.0]).unwrap();
    //        let res = kdtree
    //            .n_nearest(&array![0.9, 2.0, 3.0], closest_n)
    //            .unwrap();
    //        assert_eq!(
    //            res,
    //            vec![
    //                (array![1.0, 2.0, 3.0], 0.0),
    //                (array![1.1, 2.0, 3.0], 0.010000004),
    //            ]
    //        );
    //    }

    #[test]
    fn test_delete_sequence() {
        let dimension = NonZeroUsize::new(3).unwrap();
        let closest_n = NonZeroUsize::new(1).unwrap();
        let kdtree = Arc::new(KDTree::new(dimension));
        kdtree.insert(array![1.0, 2.0, 3.0]).unwrap();
        kdtree.insert(array![0.9, 2.0, 3.0]).unwrap();
        kdtree.insert(array![1.1, 2.0, 3.0]).unwrap();
        kdtree.insert(array![0.95, 2.0, 3.2]).unwrap();

        // Exact matches
        let res = kdtree.n_nearest(&array![0.9, 2.0, 3.0], closest_n).unwrap();
        assert_eq!(res, vec![(array![0.9, 2.0, 3.0], 0.0)]);
        let res = kdtree
            .n_nearest(&array![0.9, 2.0, 3.0], NonZeroUsize::new(4).unwrap())
            .unwrap();
        assert_eq!(res.len(), 4);

        // Delete returns nothing as exact does not exist
        let res = kdtree.delete(&array![0.05, 1.0, 2.4]).unwrap();
        assert!(res.is_none());
        let res = kdtree
            .n_nearest(&array![1.0, 2.0, 3.0], NonZeroUsize::new(4).unwrap())
            .unwrap();
        // ensure size remains the same as nothing was deleted
        assert_eq!(res.len(), 4);

        // Delete a non-leaf/non-root node
        let res = kdtree.delete(&array![0.9, 2.0, 3.0]).unwrap().unwrap();
        assert_eq!(res, array![0.9, 2.0, 3.0]);
        let res = kdtree
            .n_nearest(&array![1.0, 2.0, 3.0], NonZeroUsize::new(4).unwrap())
            .unwrap();
        // ensure size changes but only one node got removed
        assert_eq!(
            res,
            vec![
                (array![1.0, 2.0, 3.0], 0.0),
                (array![1.1, 2.0, 3.0], 0.010000004),
                (array![0.95, 2.0, 3.2], 0.04250002),
            ]
        );
        // Delete a leaf node
        let res = kdtree.delete(&array![0.95, 2.0, 3.2]).unwrap().unwrap();
        assert_eq!(res, array![0.95, 2.0, 3.2]);
        let res = kdtree
            .n_nearest(&array![1.0, 2.0, 3.0], NonZeroUsize::new(4).unwrap())
            .unwrap();
        // ensure size changes but only one node got removed
        assert_eq!(
            res,
            vec![
                (array![1.0, 2.0, 3.0], 0.0),
                (array![1.1, 2.0, 3.0], 0.010000004),
            ]
        );
        // Delete root node
        let res = kdtree.delete(&array![1.0, 2.0, 3.0]).unwrap().unwrap();
        assert_eq!(res, array![1.0, 2.0, 3.0]);
        let res = kdtree
            .n_nearest(&array![1.0, 2.0, 3.0], NonZeroUsize::new(4).unwrap())
            .unwrap();
        // ensure size changes but only one node got removed
        assert_eq!(res, vec![(array![1.1, 2.0, 3.0], 0.010000004),]);
    }
}
