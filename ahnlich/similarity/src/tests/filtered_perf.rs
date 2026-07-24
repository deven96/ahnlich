use crate::EmbeddingKey;
use crate::hnsw::index::HNSW;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::time::Instant;

/// Loose predicate: half the store matches. Asking for 10 results should be cheap —
/// almost anything near the query satisfies the predicate.
///
/// Run with:
///   cargo test -p ahnlich_similarity --release --lib filtered_perf -- --nocapture --ignored
#[test]
#[ignore]
fn perf_loose_filter() {
    const N: usize = 20_000;
    const DIM: usize = 128;

    let hnsw = HNSW::default();
    let keys: Vec<EmbeddingKey> = (0..N)
        .map(|i| {
            let base = i as f32 * 0.0001;
            EmbeddingKey::new((0..DIM).map(|d| base + d as f32 * 0.01).collect())
        })
        .collect();
    hnsw.insert(&keys).expect("insert");

    // accept half the store -> a loose, weakly selective predicate
    let accept: HashSet<u64> = keys
        .iter()
        .step_by(2)
        .map(|k| ahnlich_types::utils::hash_f32_vec(&k.0))
        .collect();

    let query: Vec<f32> = (0..DIM).map(|d| d as f32 * 0.01).collect();
    let n = NonZeroUsize::new(10).unwrap();

    // warm
    let _ = hnsw.n_nearest(&query, n, Some(accept.clone())).unwrap();

    let start = Instant::now();
    let iters = 20;
    for _ in 0..iters {
        let res = hnsw.n_nearest(&query, n, Some(accept.clone())).unwrap();
        assert_eq!(res.len(), 10, "loose filter must still return 10 results");
    }
    let elapsed = start.elapsed() / iters;

    println!(
        "LOOSE FILTER (accept {} of {}): {:?} per query",
        accept.len(),
        N,
        elapsed
    );
}
