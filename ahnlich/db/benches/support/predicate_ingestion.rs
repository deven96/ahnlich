use std::collections::HashMap;
use std::sync::Arc;

use ahnlich_db::engine::predicate_experiments::{
    Entries, Memberships, PredicateIngestionFixture, Variant,
};
use ahnlich_db::engine::store::ParallelismConfig;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::{MetadataValue, metadata_value::Value};
use ahnlich_types::utils::StoreKeyId;
use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, black_box};

pub struct Scenario {
    pub label: String,
    pub batch: usize,
    pub indexed_fields: usize,
    pub cardinality: usize,
    pub existing_fields: usize,
    pub parallel: bool,
}

fn entries(s: &Scenario, start: usize, count: usize, indexed_fields: usize) -> Entries {
    (start..start + count)
        .map(|id| {
            let mut value = HashMap::new();
            for field in 0..indexed_fields {
                value.insert(
                    format!("indexed-{field}"),
                    MetadataValue {
                        value: Some(Value::RawString(format!("value-{}", id % s.cardinality))),
                    },
                );
            }
            (StoreKeyId(id as u64), Arc::new(StoreValue { value }))
        })
        .collect()
}

pub fn benchmark(c: &mut Criterion, name: &str, candidate: Variant, scenarios: Vec<Scenario>) {
    let mut group = c.benchmark_group(name);
    group.sampling_mode(criterion::SamplingMode::Flat);

    for s in scenarios {
        let allowed: Vec<_> = (0..s.indexed_fields)
            .map(|field| format!("indexed-{field}"))
            .collect();
        // Populate existing buckets, then insert disjoint IDs into each fresh fixture.
        // Repeated iterations must not silently turn insertion into duplicate-ID updates.
        let initial = if s.existing_fields == 0 {
            Vec::new()
        } else {
            entries(&s, 0, s.cardinality, s.existing_fields)
        };
        let incoming = entries(&s, s.cardinality, s.batch, s.indexed_fields);
        let config = ParallelismConfig::from_cli(
            rayon::current_num_threads(),
            None,
            if s.parallel { 1_000 } else { usize::MAX },
        );
        assert_eq!(config.should_use_parallel(s.batch, 1), s.parallel);

        // Independent expected state, including pre-existing memberships.
        let mut expected = Memberships::new();
        for (id, entry) in initial.iter().chain(&incoming) {
            for (key, value) in &entry.value {
                if allowed.contains(key) {
                    expected
                        .entry(key.clone())
                        .or_default()
                        .entry(value.clone())
                        .or_default()
                        .insert(*id);
                }
            }
        }
        let setup = || {
            (
                PredicateIngestionFixture::new(allowed.clone(), initial.clone(), config.clone(), 1),
                incoming.clone(),
            )
        };
        for variant in [Variant::Control, candidate] {
            let (fixture, input) = setup();
            fixture.run(variant, input.clone());
            assert_eq!(fixture.memberships(), expected, "{}", s.label);
            // Also check duplicate-ID insertion, outside the measured routine.
            fixture.run(variant, input);
            assert_eq!(fixture.memberships(), expected, "{} repeated", s.label);
        }

        group.throughput(Throughput::Elements(s.batch as u64));
        for (label, variant) in [("control", Variant::Control), ("candidate", candidate)] {
            group.bench_function(BenchmarkId::new(label, &s.label), |b| {
                // PerIteration bounds memory even for large metadata. Fixture setup,
                // input cloning, inspection, and final index teardown are not timed.
                b.iter_batched_ref(
                    setup,
                    |(fixture, input)| {
                        fixture.run(variant, black_box(std::mem::take(input)));
                        black_box(&*fixture);
                    },
                    BatchSize::PerIteration,
                );
            });
        }
    }
    group.finish();
}
