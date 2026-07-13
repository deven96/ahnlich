use ahnlich_db::engine::store::StoreHandler;
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::predicates::{Predicate, PredicateCondition, predicate::Kind as PredicateKind};
use ahnlich_types::schema::Schema;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

// Memory profiling benchmark
// Uses jemalloc's stats API to measure memory usage patterns under different workloads
//
// Usage:
//   Basic memory stats: cargo bench --bench memory_profile
//   Heap profiling with dumps: MALLOC_CONF=prof:true,prof_prefix:jeprof.out cargo bench --bench memory_profile
//   Analyze dumps: jeprof --pdf target/release/deps/memory_profile-* jeprof.out.*.heap

fn main() {
    use tikv_jemalloc_ctl::{epoch, stats};

    println!("=== Memory Usage Profiling ===");
    println!("Measuring memory patterns under different workloads\n");

    // Helper to print current memory stats
    let print_stats = |label: &str| {
        epoch::mib().unwrap().advance().unwrap();
        let allocated = stats::allocated::mib().unwrap().read().unwrap();
        let resident = stats::resident::mib().unwrap().read().unwrap();
        let active = stats::active::mib().unwrap().read().unwrap();
        println!("[{}]", label);
        println!("  Allocated: {} MB", allocated / 1_048_576);
        println!("  Resident:  {} MB", resident / 1_048_576);
        println!("  Active:    {} MB\n", active / 1_048_576);
    };

    print_stats("Baseline");

    println!("\n>>> Testing memory usage with ONE persistent store...\n");

    // Create ONE handler that stays alive throughout all tests
    let handler = initialize_store_handler();
    print_stats("After creating StoreHandler");

    // Test 1: Batch insertion with NO index
    let store1 = "store_no_index";
    println!(">>> Inserting 10k vectors (1024-dim) WITHOUT HNSW index...");
    insert_vectors(&handler, store1, 10000, 1024, vec![], false);
    print_stats("After 10k vectors (NO index)");

    // Test 2: Batch insertion WITH HNSW index
    let store2 = "store_with_hnsw";
    println!(">>> Inserting 10k vectors (1024-dim) WITH HNSW index...");
    insert_vectors(&handler, store2, 10000, 1024, vec![], true);
    print_stats("After 10k vectors (WITH HNSW index)");

    // Test 3: Store with predicate index
    let store3 = "store_with_predicate";
    println!(">>> Inserting 10k vectors WITH predicate index...");
    insert_vectors(
        &handler,
        store3,
        10000,
        1024,
        vec!["category".to_string()],
        true,
    );
    print_stats("After 10k vectors (WITH predicate index)");

    // Test 4: Run similarity queries
    println!(">>> Running 1000 similarity queries...");
    run_similarity_queries(&handler, store2, 1000, 1024);
    print_stats("After 1000 similarity queries");

    // Test 5: Run predicate queries
    println!(">>> Running 1000 predicate queries...");
    run_predicate_queries(&handler, store3, 1000);
    print_stats("After 1000 predicate queries");

    // Keep handler alive to see final memory
    std::mem::forget(handler);

    println!("\n>>> Expected memory breakdown:");
    println!("  10k vectors (1024-dim): ~40 MB (theoretical minimum)");
    println!("  Per store overhead: ~5-10 MB");
    println!("  HNSW index: ~2-3x vector memory (~80-120 MB)");
    println!("  Predicate index: ~1-5 MB per indexed field");
    println!("\nTotal expected for 3 stores: ~200-300 MB");
}

fn initialize_store_handler() -> Arc<StoreHandler> {
    let write_flag = Arc::new(AtomicBool::new(false));
    Arc::new(StoreHandler::new(write_flag))
}

fn insert_vectors(
    handler: &Arc<StoreHandler>,
    store_name: &str,
    count: usize,
    dimension: usize,
    predicate_indices: Vec<String>,
    create_non_linear_index: bool,
) {
    handler
        .create_store(
            StoreName {
                value: store_name.to_string(),
            },
            &Schema::default(),
            NonZeroUsize::new(dimension).unwrap(),
            predicate_indices,
            HashSet::new(),
            create_non_linear_index,
        )
        .unwrap();

    let bulk_insert: Vec<_> = (0..count)
        .map(|i| {
            let random_array: Vec<f32> = (0..dimension).map(|_| rand::random()).collect();
            let mut metadata = HashMap::new();

            // Add metadata for predicate testing
            if i % 3 == 0 {
                metadata.insert(
                    "category".to_string(),
                    MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "categoryA".to_string(),
                        )),
                    },
                );
            }

            (
                StoreKey { key: random_array },
                StoreValue { value: metadata },
            )
        })
        .collect();

    handler
        .set_in_store(
            &StoreName {
                value: store_name.to_string(),
            },
            &Schema::default(),
            bulk_insert,
        )
        .unwrap();
}

fn run_similarity_queries(
    handler: &Arc<StoreHandler>,
    store_name: &str,
    num_queries: usize,
    dimension: usize,
) {
    for _ in 0..num_queries {
        let random_input = StoreKey {
            key: (0..dimension).map(|_| rand::random()).collect(),
        };

        handler
            .get_sim_in_store(
                &StoreName {
                    value: store_name.to_string(),
                },
                &Schema::default(),
                random_input,
                NonZeroUsize::new(50).unwrap(),
                Algorithm::CosineSimilarity,
                None,
            )
            .unwrap();
    }
}

fn run_predicate_queries(handler: &Arc<StoreHandler>, store_name: &str, num_queries: usize) {
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

    for _ in 0..num_queries {
        handler
            .get_pred_in_store(
                &StoreName {
                    value: store_name.to_string(),
                },
                &Schema::default(),
                &condition,
            )
            .unwrap();
    }
}
