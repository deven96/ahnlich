use std::{hint::black_box, num::NonZeroUsize};

use ahnlich_similarity::hnsw::{HNSWConfig, index::HNSW};
use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};

#[path = "../support/hnsw_collector_sharing.rs"]
mod support;

fn bench(c: &mut Criterion) {
    let Some(index) = HNSW::new_with_int8_cosine(HNSWConfig::default(), NonZeroUsize::MIN) else {
        eprintln!("Skipping INT8 traversal comparison: AVX2 kernel is unavailable");
        return;
    };
    let data = support::vectors(0, 10_000, 768);
    let queries = support::vectors(1_000_000, 32, 768);
    index.insert(&data).unwrap();

    let k = NonZeroUsize::new(100).unwrap();
    let mut group = c.benchmark_group("hnsw_int8_traversal");
    group.throughput(Throughput::Elements(queries.len() as u64));
    for (variant, search) in [
        (
            "control",
            HNSW::n_nearest_ids_with_ef as fn(_, _, _, _, _) -> _,
        ),
        ("candidate", HNSW::n_nearest_ids_with_int8_cosine),
    ] {
        group.bench_function(BenchmarkId::new(variant, "10k_768d_k100"), |b| {
            b.iter(|| {
                for query in &queries {
                    black_box(search(&index, black_box(query.as_slice()), k, None, None).unwrap());
                }
            });
        });
    }
    group.finish();
}

criterion_group!(benches, bench);
criterion_main!(benches);
