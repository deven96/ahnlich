use ahnlich_db::engine::store::StoreHandler;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use criterion::{criterion_group, criterion_main, Criterion};
use ndarray::Array;
use ndarray::Array1;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn initialize_store_handler() -> Arc<StoreHandler> {
    let write_flag = Arc::new(AtomicBool::new(false));
    let handler = Arc::new(StoreHandler::new(write_flag));
    handler
}

fn bench_insertion(c: &mut Criterion) {
    let store_name = "TestInsertion";
    let sizes = [100, 1000, 10000, 100000];

    let mut group = c.benchmark_group("store_sequential_insertion_without_predicates");

    for size in sizes {
        let handler = initialize_store_handler();
        let dimension = 1024;
        handler
            .create_store(
                StoreName(store_name.to_string()),
                NonZeroUsize::new(dimension).unwrap(),
                vec![],
                true,
            )
            .unwrap();
        let dimension = dimension.clone();
        let random_array = vec![(
            StoreKey(Array::from(
                (0..dimension).map(|_| rand::random()).collect::<Vec<f32>>(),
            )),
            HashMap::new(),
        )];
        group.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                for _ in 0..size {
                    let handler = handler.clone();
                    handler
                        .set_in_store(&StoreName(store_name.to_string()), random_array.clone())
                        .unwrap();
                }
            });
        });
    }
    group.finish();

    let mut group = c.benchmark_group("store_batch_insertion_without_predicates");
    for size in sizes {
        let handler = initialize_store_handler();
        let dimension = 1024;
        let bulk_insert: Vec<_> = (0..size)
            .map(|_| {
                let random_array: Array1<f32> =
                    Array::from((0..dimension).map(|_| rand::random()).collect::<Vec<f32>>());
                (StoreKey(random_array), HashMap::new())
            })
            .collect();
        handler
            .create_store(
                StoreName(store_name.to_string()),
                NonZeroUsize::new(dimension).unwrap(),
                vec![],
                true,
            )
            .unwrap();
        group.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                handler
                    .set_in_store(&StoreName(store_name.to_string()), bulk_insert.clone())
                    .unwrap();
            });
        });
    }
    group.finish();
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::new(30, 0))
        .sample_size(10)
}

// group to measure insertion time of 100, 1k, 10k and 100k
criterion_group! {
    name = insertion;
    config = criterion_config();
    targets = bench_insertion
}
criterion_main!(insertion);
