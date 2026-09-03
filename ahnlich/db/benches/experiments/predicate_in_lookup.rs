use std::time::Duration;

use ahnlich_db::engine::predicate::benchmark::PredicateInLookupBenchmark;
use ahnlich_types::metadata::{MetadataValue, metadata_value};
use ahnlich_types::predicates::{In, Predicate, predicate::Kind as PredicateKind};
use ahnlich_types::utils::StoreKeyId;
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

const INDEX_CARDINALITIES: [usize; 3] = [100, 1_000, 100_000];
const REQUESTED_VALUE_COUNTS: [usize; 3] = [1, 4, 32];
const IDS_PER_VALUE: [usize; 2] = [1, 32];
const DUPLICATE_VALUE_COUNTS: [usize; 2] = [4, 32];

#[derive(Clone, Copy)]
enum MatchPattern {
    Hits,
    Misses,
    Mixed,
}

impl MatchPattern {
    const fn label(self) -> &'static str {
        match self {
            Self::Hits => "hits",
            Self::Misses => "misses",
            Self::Mixed => "mixed",
        }
    }
}

fn metadata_value(index: usize) -> MetadataValue {
    MetadataValue {
        value: Some(metadata_value::Value::RawString(format!("value-{index}"))),
    }
}

fn missing_metadata_value(index: usize) -> MetadataValue {
    MetadataValue {
        value: Some(metadata_value::Value::RawString(format!(
            "missing-value-{index}"
        ))),
    }
}

fn fixture(cardinality: usize, ids_per_value: usize) -> PredicateInLookupBenchmark {
    let entries = (0..cardinality)
        .flat_map(|value_index| {
            let value = metadata_value(value_index);
            (0..ids_per_value).map(move |id_index| {
                let store_key_id = value_index * ids_per_value + id_index;
                (value.clone(), StoreKeyId(store_key_id as u64))
            })
        })
        .collect();

    PredicateInLookupBenchmark::new(entries)
}

fn requested_values(
    cardinality: usize,
    requested_count: usize,
    pattern: MatchPattern,
) -> Vec<MetadataValue> {
    (0..requested_count)
        .map(|index| match pattern {
            MatchPattern::Hits => metadata_value(index * cardinality / requested_count),
            MatchPattern::Misses => missing_metadata_value(index),
            MatchPattern::Mixed if index % 2 == 0 => {
                metadata_value(index * cardinality / requested_count)
            }
            MatchPattern::Mixed => missing_metadata_value(index),
        })
        .collect()
}

fn in_predicate(values: Vec<MetadataValue>) -> Predicate {
    Predicate {
        kind: Some(PredicateKind::In(In {
            key: "indexed-field".to_string(),
            values,
        })),
    }
}

fn benchmark_scenario(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    fixture: &PredicateInLookupBenchmark,
    scenario: &str,
    predicate: &Predicate,
) {
    assert_eq!(
        fixture.matches_control(predicate),
        fixture.matches_candidate(predicate),
        "control and candidate differed for {scenario}"
    );

    group.bench_with_input(
        BenchmarkId::new("control", scenario),
        predicate,
        |b, predicate| {
            b.iter(|| black_box(fixture.matches_control(black_box(predicate))));
        },
    );

    group.bench_with_input(
        BenchmarkId::new("candidate", scenario),
        predicate,
        |b, predicate| {
            b.iter(|| black_box(fixture.matches_candidate(black_box(predicate))));
        },
    );
}

fn predicate_in_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("predicate_in_lookup");
    group.sampling_mode(criterion::SamplingMode::Flat);

    for cardinality in INDEX_CARDINALITIES {
        for ids_per_value in IDS_PER_VALUE {
            let fixture = fixture(cardinality, ids_per_value);

            for requested_count in REQUESTED_VALUE_COUNTS {
                for pattern in [
                    MatchPattern::Hits,
                    MatchPattern::Misses,
                    MatchPattern::Mixed,
                ] {
                    if requested_count == 1 && matches!(pattern, MatchPattern::Mixed) {
                        continue;
                    }

                    let scenario = format!(
                        "{}_cardinality_{}_requested_{}_{}_ids_per_value",
                        cardinality,
                        requested_count,
                        pattern.label(),
                        ids_per_value
                    );
                    let predicate =
                        in_predicate(requested_values(cardinality, requested_count, pattern));
                    benchmark_scenario(&mut group, &fixture, &scenario, &predicate);
                }
            }

            for duplicate_count in DUPLICATE_VALUE_COUNTS {
                let scenario = format!(
                    "{}_cardinality_{}_duplicate_hits_{}_ids_per_value",
                    cardinality, duplicate_count, ids_per_value
                );
                let predicate = in_predicate(vec![metadata_value(0); duplicate_count]);
                benchmark_scenario(&mut group, &fixture, &scenario, &predicate);
            }
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = predicate_in_lookup
}
criterion_main!(benches);
