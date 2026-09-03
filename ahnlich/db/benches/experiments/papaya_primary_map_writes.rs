use std::collections::HashMap as StdHashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use ahnlich_similarity::EmbeddingKey;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::{MetadataValue, metadata_value::Value as MetadataValueKind};
use ahnlich_types::utils::StoreKeyId;
use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use papaya::HashMap as ConcurrentHashMap;
use rayon::prelude::*;

const EMBEDDING_DIMENSIONS: [usize; 3] = [128, 768, 1_536];
const ENTRY_COUNTS: [usize; 9] = [
    100, 500, 1_000, 2_000, 5_000, 10_000, 25_000, 50_000, 100_000,
];

type PrimaryEntry = (StoreKeyId, EmbeddingKey, Arc<StoreValue>);
type PrimaryMap = ConcurrentHashMap<StoreKeyId, (EmbeddingKey, Arc<StoreValue>)>;

#[derive(Clone, Copy, Debug)]
enum Workload {
    InsertOnly,
    UpdateOnly,
    Mixed,
}

impl Workload {
    const ALL: [Self; 3] = [Self::InsertOnly, Self::UpdateOnly, Self::Mixed];

    const fn name(self) -> &'static str {
        match self {
            Self::InsertOnly => "insert_only",
            Self::UpdateOnly => "update_only",
            Self::Mixed => "mixed",
        }
    }

    const fn starts_present(self, index: usize) -> bool {
        match self {
            Self::InsertOnly => false,
            Self::UpdateOnly => true,
            Self::Mixed => index.is_multiple_of(2),
        }
    }

    const fn expected_counts(self, entry_count: usize) -> (usize, usize) {
        match self {
            Self::InsertOnly => (entry_count, 0),
            Self::UpdateOnly => (0, entry_count),
            Self::Mixed => (entry_count / 2, entry_count.div_ceil(2)),
        }
    }
}

#[derive(Debug)]
struct WriteResult {
    inserted: usize,
    updated: usize,
    inserted_keys: Vec<EmbeddingKey>,
    map_len: usize,
}

fn metadata_value(value: impl Into<String>) -> MetadataValue {
    MetadataValue {
        value: Some(MetadataValueKind::RawString(value.into())),
    }
}

fn entry(index: usize, generation: usize, dimensions: usize) -> PrimaryEntry {
    let mut metadata = StdHashMap::new();
    metadata.insert(
        "category".to_owned(),
        metadata_value(format!("group-{}", index % 10)),
    );
    metadata.insert(
        "price_range".to_owned(),
        metadata_value(format!("range-{}", index % 5)),
    );
    metadata.insert(
        "generation".to_owned(),
        metadata_value(generation.to_string()),
    );

    let mut embedding = vec![index as f32; dimensions];
    embedding[dimensions - 1] = generation as f32;

    (
        StoreKeyId(index as u64),
        EmbeddingKey::new(embedding),
        Arc::new(StoreValue { value: metadata }),
    )
}

fn fixture(entry_count: usize, dimensions: usize) -> Vec<PrimaryEntry> {
    (0..entry_count)
        .map(|index| entry(index, 1, dimensions))
        .collect()
}

fn setup(entries: &[PrimaryEntry], workload: Workload) -> (PrimaryMap, Vec<PrimaryEntry>) {
    // Keep reservation and fixture ownership outside the timed region so the
    // benchmark isolates the primary-map write implementation.
    let map = PrimaryMap::with_capacity(entries.len().max(1));
    let pinned = map.pin();
    for (index, (key_id, _, _)) in entries.iter().enumerate() {
        if workload.starts_present(index) {
            let (_, existing_key, existing_value) = entry(index, 0, entries[index].1.0.len());
            pinned.insert(*key_id, (existing_key, existing_value));
        }
    }
    drop(pinned);

    (map, entries.to_vec())
}

fn finish(
    map: &PrimaryMap,
    inserted: usize,
    updated: usize,
    inserted_keys: Vec<EmbeddingKey>,
) -> WriteResult {
    WriteResult {
        inserted,
        updated,
        inserted_keys,
        map_len: map.pin().len(),
    }
}

// This is the primary-map write implementation before this change.
fn current_per_entry_pin(map: &PrimaryMap, entries: Vec<PrimaryEntry>) -> WriteResult {
    let inserted = AtomicUsize::new(0);
    let updated = AtomicUsize::new(0);

    let inserted_keys: Vec<_> = entries
        .into_par_iter()
        .filter_map(|(key_id, embedding_key, value)| {
            let pinned = map.pin();
            if pinned
                .insert(key_id, (embedding_key.clone(), Arc::clone(&value)))
                .is_some()
            {
                updated.fetch_add(1, Ordering::SeqCst);
                None
            } else {
                inserted.fetch_add(1, Ordering::SeqCst);
                Some(embedding_key)
            }
        })
        .collect();

    finish(
        map,
        inserted.into_inner(),
        updated.into_inner(),
        inserted_keys,
    )
}

// This is the primary-map write implementation proposed by this change.
fn sequential_one_guard(map: &PrimaryMap, entries: Vec<PrimaryEntry>) -> WriteResult {
    let pinned = map.pin();
    let mut inserted = 0;
    let mut updated = 0;
    let mut inserted_keys = Vec::new();

    for (key_id, embedding_key, value) in entries {
        if pinned
            .insert(key_id, (embedding_key.clone(), Arc::clone(&value)))
            .is_some()
        {
            updated += 1;
        } else {
            inserted += 1;
            inserted_keys.push(embedding_key);
        }
    }

    drop(pinned);
    finish(map, inserted, updated, inserted_keys)
}

fn assert_result(result: &WriteResult, workload: Workload, entry_count: usize) {
    let (expected_inserted, expected_updated) = workload.expected_counts(entry_count);
    assert_eq!(result.inserted, expected_inserted);
    assert_eq!(result.updated, expected_updated);
    assert_eq!(result.inserted_keys.len(), expected_inserted);
    assert_eq!(result.map_len, entry_count);
}

fn assert_map_contains_input(map: &PrimaryMap, entries: &[PrimaryEntry]) {
    let pinned = map.pin();
    for (key_id, expected_key, expected_value) in entries {
        let (stored_key, stored_value) = pinned.get(key_id).expect("input entry missing from map");
        assert!(Arc::ptr_eq(&stored_key.0, &expected_key.0));
        assert!(Arc::ptr_eq(stored_value, expected_value));
    }
}

fn validate_strategies(entries: &[PrimaryEntry], workload: Workload) {
    let (map, input) = setup(entries, workload);
    assert_result(&current_per_entry_pin(&map, input), workload, entries.len());
    assert_map_contains_input(&map, entries);

    let (map, input) = setup(entries, workload);
    assert_result(&sequential_one_guard(&map, input), workload, entries.len());
    assert_map_contains_input(&map, entries);
}

fn papaya_primary_map_writes(c: &mut Criterion) {
    for dimensions in EMBEDDING_DIMENSIONS {
        for workload in Workload::ALL {
            let mut group = c.benchmark_group(format!(
                "papaya_primary_map_writes/{}/{}_dimensions/{}_rayon_threads",
                workload.name(),
                dimensions,
                rayon::current_num_threads(),
            ));
            group
                .sample_size(20)
                .warm_up_time(Duration::from_secs(1))
                .measurement_time(Duration::from_secs(3));

            for entry_count in ENTRY_COUNTS {
                let entries = fixture(entry_count, dimensions);
                validate_strategies(&entries, workload);
                group.throughput(Throughput::Elements(entry_count as u64));

                group.bench_with_input(
                    BenchmarkId::new("control", entry_count),
                    &entries,
                    |b, entries| {
                        b.iter_batched(
                            || setup(entries, workload),
                            |(map, entries)| black_box(current_per_entry_pin(&map, entries)),
                            BatchSize::LargeInput,
                        )
                    },
                );

                group.bench_with_input(
                    BenchmarkId::new("candidate", entry_count),
                    &entries,
                    |b, entries| {
                        b.iter_batched(
                            || setup(entries, workload),
                            |(map, entries)| black_box(sequential_one_guard(&map, entries)),
                            BatchSize::LargeInput,
                        )
                    },
                );
            }

            group.finish();
        }
    }
}

criterion_group!(benches, papaya_primary_map_writes);
criterion_main!(benches);
