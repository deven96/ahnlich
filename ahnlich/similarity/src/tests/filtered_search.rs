use crate::EmbeddingKey;
use crate::hnsw::index::HNSW;
use std::collections::HashSet;
use std::num::NonZeroUsize;

const DIM: usize = 8;
const NEAR_COUNT: usize = 900;
const FAR_COUNT: usize = 100;

/// Build a store with two well-separated clusters:
/// - NEAR: 900 vectors clustered tightly around the origin
/// - FAR:  100 vectors clustered far away (around 100.0)
///
/// The query is the origin, so every NEAR vector beats every FAR vector.
fn two_cluster_dataset() -> (Vec<EmbeddingKey>, Vec<EmbeddingKey>) {
    let near = (0..NEAR_COUNT)
        .map(|i| EmbeddingKey::new(vec![i as f32 * 0.001; DIM]))
        .collect();
    let far = (0..FAR_COUNT)
        .map(|i| EmbeddingKey::new(vec![100.0 + i as f32 * 0.001; DIM]))
        .collect();
    (near, far)
}

/// A predicate that matches ONLY the far cluster.
///
/// There are 100 acceptable vectors and we ask for 10, so a correct filtered
/// search MUST return 10 results: the 10 nearest members of the far cluster.
///
/// The current implementation sets `search_k = max(n, accept_list.len())` = 100,
/// which drives `ef` = 100, so it retrieves the global top-100 (all of which are
/// NEAR vectors), then post-filters them against the accept list and finds
/// nothing acceptable -> returns 0 results.
#[test]
fn filtered_search_returns_n_results_when_enough_candidates_match() {
    let hnsw = HNSW::default();
    let (near, far) = two_cluster_dataset();

    hnsw.insert(&near).expect("insert near cluster");
    hnsw.insert(&far).expect("insert far cluster");

    // accept list = the far cluster only
    let accept_list: HashSet<u64> = far
        .iter()
        .map(|k| ahnlich_types::utils::hash_f32_vec(&k.0))
        .collect();

    let query = vec![0.0f32; DIM];
    let n = NonZeroUsize::new(10).unwrap();

    let results = hnsw
        .n_nearest(&query, n, Some(accept_list.clone()))
        .expect("filtered search should succeed");

    assert_eq!(
        results.len(),
        10,
        "expected 10 results (100 vectors match the predicate), got {}. \
         Filtered search is dropping results that satisfy the predicate.",
        results.len()
    );

    // every returned vector must be in the accept list
    for (key, _) in &results {
        let id = ahnlich_types::utils::hash_f32_vec(&key.0);
        assert!(
            accept_list.contains(&id),
            "returned a vector that does not satisfy the predicate"
        );
    }
}

/// Force the GRAPH in-filtering path, not the brute-force one.
///
/// With more than `BRUTE_FORCE_ACCEPT_LIST_THRESHOLD` (4096) accepted vectors, the search
/// walks the graph and filters during traversal. This is the subtle path: results must
/// still be the nearest ACCEPTED vectors, the count must be right, and the excluded nearest
/// must never appear.
#[test]
fn filtered_search_on_the_graph_path_returns_nearest_accepted() {
    const N: usize = 8_000; // > 4096 so the accept list clears the brute-force threshold
    const EXCLUDE_NEAREST: usize = 200;

    let hnsw = HNSW::default();
    // Vectors on a ray from the origin: distance grows with index, so the true ranking is
    // just the index order.
    let keys: Vec<EmbeddingKey> = (0..N)
        .map(|i| EmbeddingKey::new(vec![i as f32 * 0.01; DIM]))
        .collect();
    hnsw.insert(&keys).expect("insert");

    // Accept everything except the 200 nearest, so the filter actually changes the answer
    // and the accepted set (7800) is far above the brute-force threshold.
    let accept_list: HashSet<u64> = keys
        .iter()
        .skip(EXCLUDE_NEAREST)
        .map(|k| ahnlich_types::utils::hash_f32_vec(&k.0))
        .collect();
    assert!(accept_list.len() > 4096, "must exercise the graph path");

    let excluded: HashSet<u64> = keys
        .iter()
        .take(EXCLUDE_NEAREST)
        .map(|k| ahnlich_types::utils::hash_f32_vec(&k.0))
        .collect();

    let query = vec![0.0f32; DIM];
    let n = NonZeroUsize::new(10).unwrap();
    let results = hnsw
        .n_nearest(&query, n, Some(accept_list.clone()))
        .unwrap();

    assert_eq!(
        results.len(),
        10,
        "graph path must still return the full count"
    );

    for (key, _) in &results {
        let id = ahnlich_types::utils::hash_f32_vec(&key.0);
        assert!(accept_list.contains(&id), "returned a non-accepted vector");
        assert!(
            !excluded.contains(&id),
            "returned one of the excluded nearest"
        );
    }

    // The true nearest accepted are indices EXCLUDE_NEAREST..EXCLUDE_NEAREST+10. HNSW is
    // approximate, so require high recall rather than an exact match.
    let truth: HashSet<u64> = keys
        .iter()
        .skip(EXCLUDE_NEAREST)
        .take(10)
        .map(|k| ahnlich_types::utils::hash_f32_vec(&k.0))
        .collect();
    let hits = results
        .iter()
        .filter(|(key, _)| truth.contains(&ahnlich_types::utils::hash_f32_vec(&key.0)))
        .count();
    assert!(
        hits >= 8,
        "expected high recall of the nearest accepted, got {hits}/10"
    );
}
