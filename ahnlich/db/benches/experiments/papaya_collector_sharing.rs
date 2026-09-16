use ahnlich_db::engine::predicate_experiments::{
    CollectorSharingVariant, PredicateCollectorFixture,
};
use ahnlich_db::engine::store::ParallelismConfig;
use ahnlich_types::metadata::{MetadataValue, metadata_value::Value};
use ahnlich_types::utils::StoreKeyId;
use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    initial_entries: usize,
    incoming_entries: usize,
    cardinality: usize,
    parallel: bool,
}

const SCENARIOS: [Scenario; 8] = [
    Scenario {
        name: "first_low_cardinality_10k_sequential",
        initial_entries: 0,
        incoming_entries: 10_000,
        cardinality: 10,
        parallel: false,
    },
    Scenario {
        name: "first_unique_10k_sequential",
        initial_entries: 0,
        incoming_entries: 10_000,
        cardinality: 10_000,
        parallel: false,
    },
    Scenario {
        name: "first_low_cardinality_10k_parallel",
        initial_entries: 0,
        incoming_entries: 10_000,
        cardinality: 10,
        parallel: true,
    },
    Scenario {
        name: "first_unique_10k_parallel",
        initial_entries: 0,
        incoming_entries: 10_000,
        cardinality: 10_000,
        parallel: true,
    },
    Scenario {
        name: "existing_low_cardinality_10k_sequential",
        initial_entries: 10,
        incoming_entries: 10_000,
        cardinality: 10,
        parallel: false,
    },
    Scenario {
        name: "existing_low_cardinality_10k_parallel",
        initial_entries: 10,
        incoming_entries: 10_000,
        cardinality: 10,
        parallel: true,
    },
    Scenario {
        name: "first_unique_100k_sequential",
        initial_entries: 0,
        incoming_entries: 100_000,
        cardinality: 100_000,
        parallel: false,
    },
    Scenario {
        name: "first_unique_100k_parallel",
        initial_entries: 0,
        incoming_entries: 100_000,
        cardinality: 100_000,
        parallel: true,
    },
];

fn entries(start: usize, count: usize, cardinality: usize) -> Vec<(MetadataValue, StoreKeyId)> {
    (start..start + count)
        .map(|id| {
            (
                MetadataValue {
                    value: Some(Value::RawString(format!("value-{}", id % cardinality))),
                },
                StoreKeyId(id as u64),
            )
        })
        .collect()
}

fn expected_memberships(
    initial: &[(MetadataValue, StoreKeyId)],
    incoming: &[(MetadataValue, StoreKeyId)],
) -> BTreeMap<MetadataValue, BTreeSet<StoreKeyId>> {
    initial
        .iter()
        .chain(incoming)
        .fold(BTreeMap::new(), |mut memberships, (value, id)| {
            memberships.entry(value.clone()).or_default().insert(*id);
            memberships
        })
}

fn collector_sharing(c: &mut Criterion) {
    let config = ParallelismConfig::from_cli(rayon::current_num_threads(), None, usize::MAX);
    let mut construction = c.benchmark_group("papaya_collector_sharing/construction");
    for (label, variant) in [
        ("control", CollectorSharingVariant::Control),
        ("candidate", CollectorSharingVariant::Candidate),
    ] {
        construction.bench_function(BenchmarkId::new(label, "1"), |b| {
            b.iter(|| {
                black_box(PredicateCollectorFixture::new(
                    variant,
                    Vec::new(),
                    config.clone(),
                    1,
                ))
            });
        });
    }
    construction.finish();

    let mut group = c.benchmark_group("papaya_collector_sharing/ingestion");
    group.sampling_mode(criterion::SamplingMode::Flat);

    for scenario in SCENARIOS {
        let initial = entries(0, scenario.initial_entries, scenario.cardinality);
        let incoming = entries(
            scenario.initial_entries,
            scenario.incoming_entries,
            scenario.cardinality,
        );
        let config = ParallelismConfig::from_cli(
            rayon::current_num_threads(),
            None,
            if scenario.parallel { 1 } else { usize::MAX },
        );
        assert_eq!(
            config.should_use_parallel(scenario.incoming_entries, 1),
            scenario.parallel
        );
        let expected = expected_memberships(&initial, &incoming);

        for variant in [
            CollectorSharingVariant::Control,
            CollectorSharingVariant::Candidate,
        ] {
            let fixture =
                PredicateCollectorFixture::new(variant, initial.clone(), config.clone(), 1);
            fixture.add(incoming.clone());
            assert_eq!(fixture.memberships(), expected, "{}", scenario.name);
        }

        group.throughput(Throughput::Elements(scenario.incoming_entries as u64));
        for (label, variant) in [
            ("control", CollectorSharingVariant::Control),
            ("candidate", CollectorSharingVariant::Candidate),
        ] {
            group.bench_function(BenchmarkId::new(label, scenario.name), |b| {
                b.iter_batched(
                    || {
                        (
                            PredicateCollectorFixture::new(
                                variant,
                                initial.clone(),
                                config.clone(),
                                1,
                            ),
                            incoming.clone(),
                        )
                    },
                    |(fixture, incoming)| {
                        fixture.add(black_box(incoming));
                        black_box(fixture)
                    },
                    BatchSize::LargeInput,
                );
            });
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
    targets = collector_sharing
}
criterion_main!(benches);
