use ahnlich_db::engine::store::StoreHandler;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::similarity::Algorithm;
use ahnlich_types::similarity::NonLinearAlgorithm;
use criterion::{criterion_group, criterion_main, Criterion};
use ndarray::Array;
use ndarray::Array1;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

fn initialize_store_handler() -> Arc<StoreHandler> {
    let write_flag = Arc::new(AtomicBool::new(false));
    let handler = Arc::new(StoreHandler::new(write_flag));
    handler
}

fn bench_retrieval(c: &mut Criterion) {
    let store_name = "TestRetrieval";
    let sizes = [100, 1000, 10000, 100000];

    let mut group_no_condition = c.benchmark_group("store_retrieval_no_condition");
    for size in sizes {
        let no_condition_handler = initialize_store_handler();
        let dimension = 1024;
        let bulk_insert: Vec<_> = (0..size)
            .map(|_| {
                let random_array: Array1<f32> =
                    Array::from((0..dimension).map(|_| rand::random()).collect::<Vec<f32>>());
                (StoreKey(random_array), HashMap::new())
            })
            .collect();
        no_condition_handler
            .create_store(
                StoreName(store_name.to_string()),
                NonZeroUsize::new(dimension).unwrap(),
                vec![],
                HashSet::new(),
                true,
            )
            .unwrap();
        no_condition_handler
            .set_in_store(&StoreName(store_name.to_string()), bulk_insert.clone())
            .unwrap();
        let random_input = StoreKey(Array::from(
            (0..dimension).map(|_| rand::random()).collect::<Vec<f32>>(),
        ));
        group_no_condition.sampling_mode(criterion::SamplingMode::Flat);
        group_no_condition.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                no_condition_handler
                    .get_sim_in_store(
                        &StoreName(store_name.to_string()),
                        random_input.clone(),
                        NonZeroUsize::new(50).unwrap(),
                        Algorithm::CosineSimilarity,
                        None,
                    )
                    .unwrap();
            });
        });
    }
    group_no_condition.finish();

    let mut group_non_linear_kdtree = c.benchmark_group("store_retrieval_non_linear_kdtree");
    for size in sizes {
        let non_linear_handler = initialize_store_handler();
        let dimension = 1024;
        let bulk_insert: Vec<_> = (0..size)
            .map(|_| {
                let random_array: Array1<f32> =
                    Array::from((0..dimension).map(|_| rand::random()).collect::<Vec<f32>>());
                (StoreKey(random_array), HashMap::new())
            })
            .collect();
        non_linear_handler
            .create_store(
                StoreName(store_name.to_string()),
                NonZeroUsize::new(dimension).unwrap(),
                vec![],
                HashSet::from_iter([NonLinearAlgorithm::KDTree]),
                true,
            )
            .unwrap();
        non_linear_handler
            .set_in_store(&StoreName(store_name.to_string()), bulk_insert.clone())
            .unwrap();
        let random_input = StoreKey(Array::from(
            (0..dimension).map(|_| rand::random()).collect::<Vec<f32>>(),
        ));
        group_non_linear_kdtree.sampling_mode(criterion::SamplingMode::Flat);
        group_non_linear_kdtree.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                non_linear_handler
                    .get_sim_in_store(
                        &StoreName(store_name.to_string()),
                        random_input.clone(),
                        NonZeroUsize::new(50).unwrap(),
                        Algorithm::KDTree,
                        None,
                    )
                    .unwrap();
            });
        });
    }
    group_non_linear_kdtree.finish();
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
                HashSet::new(),
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
                HashSet::new(),
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

fn criterion_config(seconds: u64, sample_size: usize) -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::new(seconds, 0))
        .sample_size(sample_size)
}

// group to measure insertion time of 100, 1k, 10k and 100k
criterion_group! {
    name = insertion;
    config = criterion_config(30, 10);
    targets = bench_insertion
}

// group to measure retrieval time of 100, 1k, 10k and 100k
criterion_group! {
    name = retrieval;
    config = criterion_config(30, 10);
    targets = bench_retrieval
}
criterion_main!(insertion, retrieval);
