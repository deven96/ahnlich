use ahnlich_db::engine::store::StoreHandler;
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::predicates::{Predicate, PredicateCondition, predicate::Kind as PredicateKind};
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

fn main() {
    #[cfg(feature = "dhat-heap")]
    let _profiler = dhat::Profiler::new_heap();

    profile_batch_insertion(10000);
    profile_similarity_queries(10000, 100);
    profile_predicate_queries(10000, 100, false);
    profile_predicate_queries(10000, 100, true);
}

fn initialize_store_handler() -> Arc<StoreHandler> {
    let write_flag = Arc::new(AtomicBool::new(false));
    Arc::new(StoreHandler::new(write_flag))
}

fn profile_batch_insertion(size: usize) {
    let handler = initialize_store_handler();
    let store_name = "BatchInsert";
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

    let bulk_insert: Vec<_> = (0..size)
        .map(|_| {
            let random_array: Vec<f32> = (0..dimension).map(|_| rand::random()).collect();
            (
                StoreKey { key: random_array },
                StoreValue {
                    value: HashMap::new(),
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
}

fn profile_similarity_queries(size: usize, num_queries: usize) {
    let handler = initialize_store_handler();
    let store_name = "SimilarityQuery";
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

    // Insert data
    let bulk_insert: Vec<_> = (0..size)
        .map(|_| {
            let random_array: Vec<f32> = (0..dimension).map(|_| rand::random()).collect();
            (
                StoreKey { key: random_array },
                StoreValue {
                    value: HashMap::new(),
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

    // Run queries
    for _ in 0..num_queries {
        let random_input = StoreKey {
            key: (0..dimension).map(|_| rand::random()).collect(),
        };

        handler
            .get_sim_in_store(
                &StoreName {
                    value: store_name.to_string(),
                },
                random_input,
                NonZeroUsize::new(50).unwrap(),
                Algorithm::CosineSimilarity,
                None,
            )
            .unwrap();
    }
}

fn profile_predicate_queries(size: usize, num_queries: usize, with_index: bool) {
    let handler = initialize_store_handler();
    let store_name = "PredicateQuery";
    let dimension = 1024;

    // Create store with or without predicate index
    let predicate_indices = if with_index {
        vec!["category".to_string()]
    } else {
        vec![]
    };

    handler
        .create_store(
            StoreName {
                value: store_name.to_string(),
            },
            NonZeroUsize::new(dimension).unwrap(),
            predicate_indices,
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
                            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                                category.to_string(),
                            )),
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

    // Run queries
    for _ in 0..num_queries {
        handler
            .get_pred_in_store(
                &StoreName {
                    value: store_name.to_string(),
                },
                &condition,
            )
            .unwrap();
    }
}
