use ahnlich_db::engine::store::StoreHandler;
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::{KdTreeConfig, non_linear_index};
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::predicates::{Predicate, PredicateCondition, predicate::Kind as PredicateKind};
use criterion::{Criterion, criterion_group, criterion_main};

use rayon::iter::ParallelIterator;
use rayon::slice::ParallelSlice;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn generate_storekey_store_value(size: usize, dimension: usize) -> Vec<(StoreKey, StoreValue)> {
    let mut buffer: Vec<f32> = Vec::with_capacity(size * dimension);
    buffer.extend((0..size * dimension).map(|_| fastrand::f32()));

    // Use Rayon to process the buffer in parallel
    buffer
        .par_chunks_exact(dimension)
        .map(|chunk| {
            (
                StoreKey {
                    key: chunk.to_owned(),
                },
                StoreValue {
                    value: HashMap::new(),
                },
            )
        })
        .collect()
}

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
                let random_array: Vec<f32> =
                    (0..dimension).map(|_| rand::random()).collect::<Vec<f32>>();
                (
                    StoreKey { key: random_array },
                    StoreValue {
                        value: HashMap::new(),
                    },
                )
            })
            .collect();
        no_condition_handler
            .create_store(
                StoreName {
                    value: store_name.to_string(),
                },
                NonZeroUsize::new(dimension).unwrap(),
                vec![],
                HashSet::new(),
                true,
            )
            .unwrap();
        no_condition_handler
            .set_in_store(
                &StoreName {
                    value: store_name.to_string(),
                },
                bulk_insert.clone(),
            )
            .unwrap();
        let random_input = StoreKey {
            key: (0..dimension).map(|_| rand::random()).collect::<Vec<f32>>(),
        };
        group_no_condition.sampling_mode(criterion::SamplingMode::Flat);
        group_no_condition.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                no_condition_handler
                    .get_sim_in_store(
                        &StoreName {
                            value: store_name.to_string(),
                        },
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
                let random_array: Vec<f32> =
                    (0..dimension).map(|_| rand::random()).collect::<Vec<f32>>();
                (
                    StoreKey { key: random_array },
                    StoreValue {
                        value: HashMap::new(),
                    },
                )
            })
            .collect();
        non_linear_handler
            .create_store(
                StoreName {
                    value: store_name.to_string(),
                },
                NonZeroUsize::new(dimension).unwrap(),
                vec![],
                HashSet::from_iter([non_linear_index::Index::Kdtree(KdTreeConfig {})]),
                true,
            )
            .unwrap();
        non_linear_handler
            .set_in_store(
                &StoreName {
                    value: store_name.to_string(),
                },
                bulk_insert.clone(),
            )
            .unwrap();
        let random_input = StoreKey {
            key: (0..dimension).map(|_| rand::random()).collect::<Vec<f32>>(),
        };
        group_non_linear_kdtree.sampling_mode(criterion::SamplingMode::Flat);
        group_non_linear_kdtree.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                non_linear_handler
                    .get_sim_in_store(
                        &StoreName {
                            value: store_name.to_string(),
                        },
                        random_input.clone(),
                        NonZeroUsize::new(50).unwrap(),
                        Algorithm::KdTree,
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
                StoreName {
                    value: store_name.to_string(),
                },
                NonZeroUsize::new(dimension).unwrap(),
                vec![],
                HashSet::new(),
                true,
            )
            .unwrap();
        let dimension = dimension.clone();
        let random_array = vec![(
            StoreKey {
                key: (0..dimension)
                    .map(|_| fastrand::f32())
                    .collect::<Vec<f32>>(),
            },
            StoreValue {
                value: HashMap::new(),
            },
        )];
        group.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                for _ in 0..size {
                    let handler = handler.clone();
                    handler
                        .set_in_store(
                            &StoreName {
                                value: store_name.to_string(),
                            },
                            random_array.clone(),
                        )
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

        let bulk_insert = generate_storekey_store_value(size, dimension);

        handler
            .create_store(
                StoreName {
                    value: store_name.to_string(),
                },
                NonZeroUsize::new(dimension).unwrap(),
                vec![],
                HashSet::new(),
                true,
            )
            .unwrap();
        group.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                handler
                    .set_in_store(
                        &StoreName {
                            value: store_name.to_string(),
                        },
                        bulk_insert.clone(),
                    )
                    .unwrap();
            });
        });
    }
    group.finish();
}

fn bench_predicate_queries(c: &mut Criterion) {
    let store_name = "TestPredicate";
    let sizes = [100, 1000, 10000, 100000];

    // Benchmark non-indexed predicate queries (should benefit from parallelization)
    let mut group_no_index = c.benchmark_group("predicate_query_without_index");
    for size in sizes {
        let handler = initialize_store_handler();
        let dimension = 1024;

        // Create store WITHOUT predicate index
        handler
            .create_store(
                StoreName {
                    value: store_name.to_string(),
                },
                NonZeroUsize::new(dimension).unwrap(),
                vec![], // No predicate indices
                HashSet::new(),
                true,
            )
            .unwrap();

        // Insert data with metadata
        let bulk_insert: Vec<_> = (0..size)
            .map(|i| {
                let random_array: Vec<f32> = (0..dimension).map(|_| rand::random()).collect();
                let category = if i % 3 == 0 {
                    "categoryA"
                } else if i % 3 == 1 {
                    "categoryB"
                } else {
                    "categoryC"
                };
                (
                    StoreKey { key: random_array },
                    StoreValue {
                        value: HashMap::from_iter([(
                            "category".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        category.to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )
            })
            .collect();

        handler
            .set_in_store(
                &StoreName {
                    value: store_name.to_string(),
                },
                bulk_insert,
            )
            .unwrap();

        // Create predicate condition
        let condition = PredicateCondition {
            kind: Some(ahnlich_types::predicates::predicate_condition::Kind::Value(
                Predicate {
                    kind: Some(PredicateKind::Equals(ahnlich_types::predicates::Equals {
                        key: "category".to_string(),
                        value: Some(MetadataValue {
                            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                                "categoryA".to_string(),
                            )),
                        }),
                    })),
                },
            )),
        };

        group_no_index.sampling_mode(criterion::SamplingMode::Flat);
        group_no_index.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                handler
                    .get_pred_in_store(
                        &StoreName {
                            value: store_name.to_string(),
                        },
                        &condition,
                    )
                    .unwrap();
            });
        });
    }
    group_no_index.finish();

    // Benchmark indexed predicate queries (for comparison)
    let mut group_with_index = c.benchmark_group("predicate_query_with_index");
    for size in sizes {
        let handler = initialize_store_handler();
        let dimension = 1024;

        // Create store WITH predicate index
        handler
            .create_store(
                StoreName {
                    value: store_name.to_string(),
                },
                NonZeroUsize::new(dimension).unwrap(),
                vec!["category".to_string()], // WITH predicate index
                HashSet::new(),
                true,
            )
            .unwrap();

        // Insert data with metadata
        let bulk_insert: Vec<_> = (0..size)
            .map(|i| {
                let random_array: Vec<f32> = (0..dimension).map(|_| rand::random()).collect();
                let category = if i % 3 == 0 {
                    "categoryA"
                } else if i % 3 == 1 {
                    "categoryB"
                } else {
                    "categoryC"
                };
                (
                    StoreKey { key: random_array },
                    StoreValue {
                        value: HashMap::from_iter([(
                            "category".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        category.to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )
            })
            .collect();

        handler
            .set_in_store(
                &StoreName {
                    value: store_name.to_string(),
                },
                bulk_insert,
            )
            .unwrap();

        // Create predicate condition
        let condition = PredicateCondition {
            kind: Some(ahnlich_types::predicates::predicate_condition::Kind::Value(
                Predicate {
                    kind: Some(PredicateKind::Equals(ahnlich_types::predicates::Equals {
                        key: "category".to_string(),
                        value: Some(MetadataValue {
                            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                                "categoryA".to_string(),
                            )),
                        }),
                    })),
                },
            )),
        };

        group_with_index.sampling_mode(criterion::SamplingMode::Flat);
        group_with_index.bench_function(format!("size_{size}"), |b| {
            b.iter(|| {
                handler
                    .get_pred_in_store(
                        &StoreName {
                            value: store_name.to_string(),
                        },
                        &condition,
                    )
                    .unwrap();
            });
        });
    }
    group_with_index.finish();
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

// group to measure predicate query time
criterion_group! {
    name = predicate;
    config = criterion_config(30, 10);
    targets = bench_predicate_queries
}

criterion_main!(insertion, retrieval, predicate);
