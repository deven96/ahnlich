//! Isolates the existing-index lookup fast path; metadata grouping is unchanged.
#[path = "../support/predicate_ingestion.rs"]
mod support;

use ahnlich_db::engine::predicate_experiments::Variant;
use criterion::{Criterion, criterion_group, criterion_main};
use std::time::Duration;
use support::Scenario;

fn predicate_existing_index_ingestion(c: &mut Criterion) {
    let mut scenarios = Vec::new();
    for (batch, parallel) in [(100, false), (1_000, false), (1_000, true)] {
        for (indexed_fields, cardinality) in [(1, 1), (1, 100), (4, 100)] {
            scenarios.push(Scenario {
                label: format!("existing_batch{batch}_fields{indexed_fields}_values{cardinality}_parallel{parallel}"),
                batch, indexed_fields, cardinality, parallel,
                existing_fields: indexed_fields,
            });
        }
    }
    for (label, existing_fields) in [("missing", 0), ("mixed", 2)] {
        scenarios.push(Scenario {
            label: format!("{label}_batch1000_fields4_values100_parallelfalse"),
            batch: 1_000,
            indexed_fields: 4,
            cardinality: 100,
            parallel: false,
            existing_fields,
        });
    }
    support::benchmark(
        c,
        "predicate_existing_index_ingestion",
        Variant::ExistingIndex,
        scenarios,
    );
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = predicate_existing_index_ingestion
}
criterion_main!(benches);
