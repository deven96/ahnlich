#![allow(dead_code)]
/// Heirarchical Navigable Small Worlds establishes a localised list of closest nodes based on a
/// similarity function. It then navigates between these localised lists in DFS manner until it
/// gets the values it needs to
use std::collections::BinaryHeap;

struct HeirarchicalNavigable {
    heap: BinaryHeap<f32>,
}
