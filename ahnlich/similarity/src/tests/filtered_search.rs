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
