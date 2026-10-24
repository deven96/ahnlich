use super::experiments::ControlHNSW;
use super::*;
use rand::{Rng, SeedableRng, rngs::StdRng};
use std::sync::Barrier;

const DIMENSION: usize = 16;
const WRITERS: usize = 8;
const BATCHES_PER_WRITER: usize = 4;
const BATCH_SIZE: usize = 40;

fn vectors(seed: u64, count: usize) -> Vec<EmbeddingKey> {
    let mut rng = StdRng::seed_from_u64(seed);
    (0..count)
        .map(|_| EmbeddingKey::new((0..DIMENSION).map(|_| rng.gen_range(-2.0..2.0)).collect()))
        .collect()
}

fn config() -> HNSWConfig {
    HNSWConfig {
        ef_construction: 100,
        maximum_connections: 8,
        maximum_connections_zero: 16,
        extend_candidates: false,
        keep_pruned_connections: false,
    }
}

fn hash_set_index() -> ControlHNSW<LinearAlgorithm> {
    ControlHNSW::new_control_with_config_and_shared_collector_and_counter_shards(
        config(),
        LinearAlgorithm::EuclideanDistance,
        NonZeroUsize::new(1).unwrap(),
    )
}

fn compact_index() -> HNSW<LinearAlgorithm, CompactEdges> {
    HNSW::new_with_config_and_shared_collector_and_counter_shards(
        config(),
        LinearAlgorithm::EuclideanDistance,
        NonZeroUsize::new(1).unwrap(),
    )
}

/// Every edge and backlink of every stored node must resolve to a stored node.
fn dangling_references<E: EdgeStorage>(index: &HNSW<LinearAlgorithm, E>) -> Vec<(NodeId, NodeId)> {
    let nodes = index.nodes.pin();
    let mut dangling = Vec::new();
    for (id, node) in nodes.iter() {
        for edges in node.neighbours.pin().values() {
            for target in edges.snapshot().iter() {
                if !nodes.contains_key(target) {
                    dangling.push((*id, *target));
                }
            }
        }
        for target in node.back_links.pin().iter() {
            if !nodes.contains_key(target) {
                dangling.push((*id, *target));
            }
        }
    }
    dangling
}

fn check_concurrent_inserts_are_complete_and_consistent<E: EdgeStorage>(
    index: &HNSW<LinearAlgorithm, E>,
) {
    let seed = vectors(1, 300);
    index.insert(&seed).unwrap();

    let batches: Vec<Vec<EmbeddingKey>> = (0..WRITERS * BATCHES_PER_WRITER)
        .map(|batch| vectors(1_000 + batch as u64, BATCH_SIZE))
        .collect();
    let barrier = Barrier::new(WRITERS);

    let errors: Vec<String> = std::thread::scope(|scope| {
        let handles: Vec<_> = batches
            .chunks(BATCHES_PER_WRITER)
            .map(|assigned| {
                let barrier = &barrier;
                scope.spawn(move || {
                    barrier.wait();
                    assigned
                        .iter()
                        .filter_map(|batch| index.insert(batch).err().map(|e| format!("{e:?}")))
                        .collect::<Vec<_>>()
                })
            })
            .collect();
        handles
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect()
    });
    assert!(errors.is_empty(), "insert errors: {errors:?}");

    let expected: std::collections::HashSet<NodeId> = seed
        .iter()
        .chain(batches.iter().flatten())
        .map(|key| get_node_id(key.as_slice()))
        .collect();
    let stored: std::collections::HashSet<NodeId> = index.nodes.pin().keys().copied().collect();
    assert_eq!(
        stored.len(),
        expected.len(),
        "some vectors were not indexed"
    );
    assert_eq!(stored, expected);
    assert_eq!(dangling_references(index), vec![]);
}

#[test]
fn test_concurrent_inserts_are_complete_and_consistent_hash_set() {
    check_concurrent_inserts_are_complete_and_consistent(&hash_set_index());
}

#[test]
fn test_concurrent_inserts_are_complete_and_consistent_compact() {
    check_concurrent_inserts_are_complete_and_consistent(&compact_index());
}

fn check_searches_during_inserts_never_fail<E: EdgeStorage>(index: &HNSW<LinearAlgorithm, E>) {
    let seed = vectors(2, 300);
    index.insert(&seed).unwrap();
    let batches: Vec<Vec<EmbeddingKey>> = (0..WRITERS)
        .map(|batch| vectors(2_000 + batch as u64, BATCH_SIZE * BATCHES_PER_WRITER))
        .collect();
    let barrier = Barrier::new(WRITERS + 4);
    let done = std::sync::atomic::AtomicUsize::new(0);

    let failures: Vec<String> = std::thread::scope(|scope| {
        let writers: Vec<_> = batches
            .iter()
            .map(|batch| {
                let (barrier, done) = (&barrier, &done);
                scope.spawn(move || {
                    barrier.wait();
                    let result = index.insert(batch);
                    done.fetch_add(1, Ordering::SeqCst);
                    result.err().map(|e| format!("insert: {e:?}"))
                })
            })
            .collect();
        let readers: Vec<_> = (0..4)
            .map(|_| {
                let (barrier, done, seed) = (&barrier, &done, &seed);
                scope.spawn(move || {
                    barrier.wait();
                    let mut failures = Vec::new();
                    while done.load(Ordering::SeqCst) < WRITERS {
                        if let Err(e) = index.n_nearest_with_ef(
                            seed[0].as_slice(),
                            NonZeroUsize::new(10).unwrap(),
                            None,
                            NonZeroUsize::new(50),
                        ) {
                            failures.push(format!("search: {e:?}"));
                        }
                    }
                    failures.pop()
                })
            })
            .collect();
        writers
            .into_iter()
            .chain(readers)
            .filter_map(|handle| handle.join().unwrap())
            .collect()
    });
    assert!(failures.is_empty(), "failures: {failures:?}");
}

#[test]
fn test_searches_during_inserts_never_fail_hash_set() {
    check_searches_during_inserts_never_fail(&hash_set_index());
}

#[test]
fn test_searches_during_inserts_never_fail_compact() {
    check_searches_during_inserts_never_fail(&compact_index());
}

fn check_insert_survives_chosen_neighbours_being_deleted<E: EdgeStorage>(
    index: Arc<HNSW<LinearAlgorithm, E>>,
) {
    index.insert(&vectors(5, 300)).unwrap();
    let entry = index.enter_point.load().ids[0];
    let removed = std::rc::Rc::new(std::cell::Cell::new(0usize));

    // Delete every chosen neighbour (except the entry point) right after selection,
    // which is what a concurrent delete between selection and linking looks like.
    AFTER_NEIGHBOUR_SELECTION.with(|hook| {
        let (index, removed) = (Arc::clone(&index), std::rc::Rc::clone(&removed));
        *hook.borrow_mut() = Some(Box::new(move |chosen| {
            for id in chosen.iter().filter(|id| **id != entry) {
                index.delete_node(id);
                removed.set(removed.get() + 1);
            }
        }));
    });
    let new_key = vectors(6, 1).remove(0);
    let result = index.insert(std::slice::from_ref(&new_key));
    AFTER_NEIGHBOUR_SELECTION.with(|hook| *hook.borrow_mut() = None);

    result.expect("insert must tolerate its chosen neighbours being deleted");
    assert!(removed.get() > 0, "hook never removed a neighbour");

    let new_id = get_node_id(new_key.as_slice());
    let nodes = index.nodes.pin();
    let node = nodes.get(&new_id).expect("new node must be stored");
    let node_neighbours = node.neighbours.pin();
    let layer_zero = node_neighbours
        .get(&LayerIndex(0))
        .expect("new node must have layer 0 edges");
    let targets: Vec<NodeId> = layer_zero.snapshot().iter().copied().collect();
    assert!(!targets.is_empty(), "new node was left without edges");
    for target in targets {
        let target_node = nodes.get(&target).expect("edge target must be stored");
        let back = target_node.neighbours.pin();
        let back_edges = back.get(&LayerIndex(0)).expect("target has layer 0 edges");
        assert!(
            back_edges.snapshot().iter().any(|id| *id == new_id),
            "target does not link back to the new node"
        );
    }
    assert_eq!(dangling_references(&index), vec![]);
}

#[test]
fn test_insert_survives_chosen_neighbours_being_deleted_hash_set() {
    check_insert_survives_chosen_neighbours_being_deleted(Arc::new(hash_set_index()));
}

#[test]
fn test_insert_survives_chosen_neighbours_being_deleted_compact() {
    check_insert_survives_chosen_neighbours_being_deleted(Arc::new(compact_index()));
}

fn check_deleting_the_entry_point_keeps_the_index_usable<E: EdgeStorage>(
    index: &HNSW<LinearAlgorithm, E>,
) {
    let data = vectors(3, 400);
    index.insert(&data).unwrap();
    assert!(index.top_layer() >= 1, "fixture should have upper layers");

    let entry = index.enter_point.load().ids[0];
    let entry_key = data
        .iter()
        .find(|key| get_node_id(key.as_slice()) == entry)
        .unwrap()
        .clone();
    assert_eq!(index.delete(std::slice::from_ref(&entry_key)).unwrap(), 1);

    let results = index
        .n_nearest_with_ef(
            data[1].as_slice(),
            NonZeroUsize::new(10).unwrap(),
            None,
            NonZeroUsize::new(50),
        )
        .expect("search must still work after the entry point is deleted");
    assert!(!results.is_empty());
    assert!(results.iter().all(|(key, _)| *key != entry_key));

    index
        .insert(&vectors(4, 20))
        .expect("insert must still work after the entry point is deleted");
    assert_eq!(dangling_references(index), vec![]);

    assert_entry_is_highest_with_smallest_id(index);

    // Keep deleting the entry point. Once only level-0 nodes remain they all tie, which
    // is what the smallest-ID rule is for.
    for _ in 0..100 {
        let entry = index.enter_point.load().ids[0];
        index.delete_node(&entry);
        assert_entry_is_highest_with_smallest_id(index);
    }
    assert_eq!(dangling_references(index), vec![]);
}

/// After an entry point delete, the entry point must be the highest-level node, with ties
/// broken by smallest ID, and `top_layer()` must equal that level.
fn assert_entry_is_highest_with_smallest_id<E: EdgeStorage>(index: &HNSW<LinearAlgorithm, E>) {
    let nodes = index.nodes.pin();
    let level = |node: &Node<E>| node.level(index.maximum_connections);
    let highest = nodes.values().map(level).max().unwrap();
    let smallest_at_highest = nodes
        .iter()
        .filter(|(_, node)| level(node) == highest)
        .map(|(id, _)| id.0)
        .min()
        .unwrap();
    assert_eq!(index.enter_point.load().ids[0].0, smallest_at_highest);
    assert_eq!(index.top_layer(), highest);
}

#[test]
fn test_deleting_the_entry_point_keeps_the_index_usable_hash_set() {
    check_deleting_the_entry_point_keeps_the_index_usable(&hash_set_index());
}

#[test]
fn test_deleting_the_entry_point_keeps_the_index_usable_compact() {
    check_deleting_the_entry_point_keeps_the_index_usable(&compact_index());
}

fn check_delete_tolerates_a_backlink_to_a_missing_node<E: EdgeStorage>(
    index: &HNSW<LinearAlgorithm, E>,
) {
    let data = vectors(7, 100);
    index.insert(&data).unwrap();

    let nodes = index.nodes.pin();
    let (victim, backlinked) = nodes
        .iter()
        .find_map(|(id, node)| {
            node.back_links
                .pin()
                .iter()
                .next()
                .map(|backlink| (*id, *backlink))
        })
        .unwrap();
    // The backlinked node disappears (as if another thread deleted it), leaving a stale
    // backlink behind on the node we are about to delete.
    nodes.remove(&backlinked);

    index.delete_node(&victim);
    assert!(!nodes.contains_key(&victim));
}

#[test]
fn test_delete_tolerates_a_backlink_to_a_missing_node_hash_set() {
    check_delete_tolerates_a_backlink_to_a_missing_node(&hash_set_index());
}

#[test]
fn test_delete_tolerates_a_backlink_to_a_missing_node_compact() {
    check_delete_tolerates_a_backlink_to_a_missing_node(&compact_index());
}

#[test]
fn test_deleting_every_node_leaves_a_reusable_index() {
    let index = hash_set_index();
    let data = vectors(8, 60);
    index.insert(&data).unwrap();
    assert_eq!(index.delete(&data).unwrap(), data.len());
    assert!(index.nodes.pin().is_empty());
    assert!(index.enter_point.load().ids.is_empty());
    assert_eq!(index.top_layer(), 0);

    index.insert(&data[..20]).unwrap();
    let results = index
        .n_nearest_with_ef(
            data[0].as_slice(),
            NonZeroUsize::new(5).unwrap(),
            None,
            NonZeroUsize::new(20),
        )
        .unwrap();
    assert_eq!(results.len(), 5);
}

fn check_search_survives_entry_point_deleted_mid_search<E: EdgeStorage>(
    index: Arc<HNSW<LinearAlgorithm, E>>,
) {
    let data = vectors(9, 400);
    index.insert(&data).unwrap();
    assert!(index.top_layer() >= 1);
    let entry = index.enter_point.load().ids[0];

    // Delete the entry point after the search has loaded it, as a concurrent delete would.
    AFTER_SEARCH_ENTRY_LOAD.with(|hook| {
        let index = Arc::clone(&index);
        *hook.borrow_mut() = Some(Box::new(move || index.delete_node(&entry)));
    });
    let results = index.n_nearest_with_ef(
        data[1].as_slice(),
        NonZeroUsize::new(10).unwrap(),
        None,
        NonZeroUsize::new(50),
    );
    AFTER_SEARCH_ENTRY_LOAD.with(|hook| *hook.borrow_mut() = None);

    let results = results.expect("search must tolerate its entry point being deleted");
    assert_eq!(results.len(), 10);
}

#[test]
fn test_search_survives_entry_point_deleted_mid_search_hash_set() {
    check_search_survives_entry_point_deleted_mid_search(Arc::new(hash_set_index()));
}

#[test]
fn test_search_survives_entry_point_deleted_mid_search_compact() {
    check_search_survives_entry_point_deleted_mid_search(Arc::new(compact_index()));
}

fn check_stale_writer_does_not_overwrite_a_higher_entry_point<E: EdgeStorage>(
    index: Arc<HNSW<LinearAlgorithm, E>>,
) {
    index.insert(&vectors(10, 200)).unwrap();
    let level = |key: &EmbeddingKey| {
        Node::<E>::new_with_edges(key.clone()).level(index.maximum_connections)
    };
    let base = index.top_layer();

    // A node above the current top layer, and a second one that is higher still.
    let mut above = (11..)
        .map(|seed| vectors(seed, 1).remove(0))
        .filter(|key| level(key) > base);
    let slow = above.next().unwrap();
    let fast = above.find(|key| level(key) > level(&slow)).unwrap();

    // `slow` reads the entry point, then `fast` finishes inserting before `slow` writes.
    BEFORE_ENTRY_PROMOTION.with(|hook| {
        let (index, fast) = (Arc::clone(&index), fast.clone());
        *hook.borrow_mut() = Some(Box::new(move || {
            index.insert(std::slice::from_ref(&fast)).unwrap();
        }));
    });
    index.insert(std::slice::from_ref(&slow)).unwrap();

    let entry = index.enter_point.load();
    assert_eq!(entry.ids[0], get_node_id(fast.as_slice()));
    assert_eq!(entry.level, level(&fast));
    assert_eq!(index.top_layer(), level(&fast));
}

#[test]
fn test_stale_writer_does_not_overwrite_a_higher_entry_point_hash_set() {
    check_stale_writer_does_not_overwrite_a_higher_entry_point(Arc::new(hash_set_index()));
}

#[test]
fn test_stale_writer_does_not_overwrite_a_higher_entry_point_compact() {
    check_stale_writer_does_not_overwrite_a_higher_entry_point(Arc::new(compact_index()));
}

fn check_new_node_is_stored_before_it_is_linked<E: EdgeStorage>(
    index: Arc<HNSW<LinearAlgorithm, E>>,
) {
    index.insert(&vectors(11, 200)).unwrap();
    let key = vectors(12, 1).remove(0);
    let new_id = get_node_id(key.as_slice());

    // Runs between neighbour selection and linking. A node that is not stored yet cannot take
    // part in pruning the neighbours it links to.
    let stored_at_link_time = std::rc::Rc::new(std::cell::Cell::new(false));
    AFTER_NEIGHBOUR_SELECTION.with(|hook| {
        let (index, seen) = (Arc::clone(&index), std::rc::Rc::clone(&stored_at_link_time));
        *hook.borrow_mut() = Some(Box::new(move |_| {
            seen.set(index.nodes.pin().contains_key(&new_id));
        }));
    });
    let result = index.insert(std::slice::from_ref(&key));
    AFTER_NEIGHBOUR_SELECTION.with(|hook| *hook.borrow_mut() = None);

    result.unwrap();
    assert!(
        stored_at_link_time.get(),
        "node was not stored when linking started"
    );
    assert_eq!(dangling_references(&index), vec![]);
}

#[test]
fn test_new_node_is_stored_before_it_is_linked_hash_set() {
    check_new_node_is_stored_before_it_is_linked(Arc::new(hash_set_index()));
}

#[test]
fn test_new_node_is_stored_before_it_is_linked_compact() {
    check_new_node_is_stored_before_it_is_linked(Arc::new(compact_index()));
}

#[test]
fn test_inserting_the_same_node_twice_keeps_one_copy() {
    let index = compact_index();
    let key = vectors(13, 1).remove(0);
    index.insert(std::slice::from_ref(&key)).unwrap();
    index.insert(std::slice::from_ref(&key)).unwrap();
    assert_eq!(index.nodes.pin().len(), 1);
}

fn check_writers_racing_into_an_empty_index_still_get_linked<E: EdgeStorage>(
    index: Arc<HNSW<LinearAlgorithm, E>>,
) {
    let [first, second]: [EmbeddingKey; 2] = vectors(14, 2).try_into().unwrap();

    // While `first` is still linking into the empty index, `second` is inserted, as a second
    // writer would be.
    let ran = std::rc::Rc::new(std::cell::Cell::new(false));
    AFTER_NEIGHBOUR_SELECTION.with(|hook| {
        let (index, second, ran) = (Arc::clone(&index), second.clone(), std::rc::Rc::clone(&ran));
        *hook.borrow_mut() = Some(Box::new(move |_| {
            if !ran.replace(true) {
                index.insert(std::slice::from_ref(&second)).unwrap();
            }
        }));
    });
    let result = index.insert(std::slice::from_ref(&first));
    AFTER_NEIGHBOUR_SELECTION.with(|hook| *hook.borrow_mut() = None);
    result.unwrap();
    assert!(ran.get());

    let nodes = index.nodes.pin();
    assert_eq!(nodes.len(), 2);
    for (id, node) in nodes.iter() {
        let edges = node.neighbours.pin();
        let layer_zero = edges
            .get(&LayerIndex(0))
            .map_or(0, |edges| edges.snapshot().len());
        assert!(layer_zero > 0, "{id:?} was left without edges");
    }
    assert_eq!(dangling_references(&index), vec![]);
}

#[test]
fn test_writers_racing_into_an_empty_index_still_get_linked_hash_set() {
    check_writers_racing_into_an_empty_index_still_get_linked(Arc::new(hash_set_index()));
}

#[test]
fn test_writers_racing_into_an_empty_index_still_get_linked_compact() {
    check_writers_racing_into_an_empty_index_still_get_linked(Arc::new(compact_index()));
}
