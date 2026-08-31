use std::collections::HashMap;
use std::sync::Arc;

use ahnlich_types::metadata::{MetadataValue, metadata_value::Value};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use tikv_jemalloc_ctl::{epoch, stats};

const KEY_CARDINALITY: usize = 4;

#[derive(Clone, Copy)]
enum Representation {
    Owned,
    KeyInterned,
    Interned,
}

impl Representation {
    fn from_env() -> Self {
        match std::env::var("AHNLICH_METADATA_REPRESENTATION")
            .as_deref()
            .unwrap_or("owned")
        {
            "owned" => Self::Owned,
            "key_interned" => Self::KeyInterned,
            "interned" => Self::Interned,
            value => {
                panic!(
                    "invalid metadata representation '{value}': expected owned, key_interned, or interned"
                )
            }
        }
    }
}

type SourceRows = Vec<HashMap<String, MetadataValue>>;

struct OwnedMetadata {
    rows: SourceRows,
}

#[derive(Default)]
struct StringInterner {
    ids: HashMap<Arc<str>, u32>,
    values: Vec<Arc<str>>,
}

impl StringInterner {
    fn intern(&mut self, value: &str) -> u32 {
        if let Some(id) = self.ids.get(value) {
            return *id;
        }

        let id = u32::try_from(self.values.len()).expect("benchmark interner exhausted u32 IDs");
        let value: Arc<str> = Arc::from(value);
        self.values.push(Arc::clone(&value));
        self.ids.insert(value, id);
        id
    }

    fn id(&self, value: &str) -> Option<u32> {
        self.ids.get(value).copied()
    }

    fn value(&self, id: u32) -> &str {
        &self.values[id as usize]
    }
}

struct InternedMetadata {
    interner: StringInterner,
    rows: Vec<HashMap<u32, u32>>,
}

struct KeyInternedMetadata {
    key_interner: StringInterner,
    rows: Vec<HashMap<u32, MetadataValue>>,
}

enum MetadataStore {
    Owned(OwnedMetadata),
    KeyInterned(KeyInternedMetadata),
    Interned(InternedMetadata),
}

impl MetadataStore {
    fn build(representation: Representation, source: SourceRows) -> Self {
        match representation {
            Representation::Owned => Self::Owned(OwnedMetadata { rows: source }),
            Representation::KeyInterned => {
                let mut key_interner = StringInterner::default();
                let rows = source
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|(key, value)| (key_interner.intern(&key), value))
                            .collect()
                    })
                    .collect();
                Self::KeyInterned(KeyInternedMetadata { key_interner, rows })
            }
            Representation::Interned => {
                let mut interner = StringInterner::default();
                let rows = source
                    .into_iter()
                    .map(|row| {
                        row.into_iter()
                            .map(|(key, value)| {
                                (
                                    interner.intern(&key),
                                    interner.intern(raw_string_ref(&value)),
                                )
                            })
                            .collect()
                    })
                    .collect();
                Self::Interned(InternedMetadata { interner, rows })
            }
        }
    }

    fn count_equal(&self, key: &str, value: &str) -> usize {
        match self {
            Self::Owned(metadata) => metadata
                .rows
                .iter()
                .filter(|row| {
                    row.get(key)
                        .is_some_and(|candidate| raw_string_ref(candidate) == value)
                })
                .count(),
            Self::KeyInterned(metadata) => {
                let Some(key) = metadata.key_interner.id(key) else {
                    return 0;
                };
                metadata
                    .rows
                    .iter()
                    .filter(|row| {
                        row.get(&key)
                            .is_some_and(|candidate| raw_string_ref(candidate) == value)
                    })
                    .count()
            }
            Self::Interned(metadata) => {
                let (Some(key), Some(value)) =
                    (metadata.interner.id(key), metadata.interner.id(value))
                else {
                    return 0;
                };
                metadata
                    .rows
                    .iter()
                    .filter(|row| row.get(&key) == Some(&value))
                    .count()
            }
        }
    }

    fn materialize(&self) -> SourceRows {
        match self {
            Self::Owned(metadata) => metadata.rows.clone(),
            Self::KeyInterned(metadata) => metadata
                .rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|(key, value)| {
                            (metadata.key_interner.value(*key).to_owned(), value.clone())
                        })
                        .collect()
                })
                .collect(),
            Self::Interned(metadata) => metadata
                .rows
                .iter()
                .map(|row| {
                    row.iter()
                        .map(|(key, value)| {
                            (
                                metadata.interner.value(*key).to_owned(),
                                raw_string(metadata.interner.value(*value)),
                            )
                        })
                        .collect()
                })
                .collect(),
        }
    }
}

fn raw_string(value: impl Into<String>) -> MetadataValue {
    MetadataValue {
        value: Some(Value::RawString(value.into())),
    }
}

fn raw_string_ref(value: &MetadataValue) -> &str {
    match &value.value {
        Some(Value::RawString(value)) => value,
        _ => panic!("metadata representation benchmark expects RawString values"),
    }
}

fn source_rows(size: usize) -> SourceRows {
    const KEYS: [&str; KEY_CARDINALITY] = ["category", "brand", "region", "availability"];
    const VALUE_PREFIXES: [&str; KEY_CARDINALITY] = ["category", "brand", "region", "state"];

    (0..size)
        .map(|row| {
            KEYS.into_iter()
                .zip(VALUE_PREFIXES)
                .enumerate()
                .map(|(column, (key, prefix))| {
                    (
                        key.to_owned(),
                        raw_string(format!("{prefix}-{}", (row + column) % 8)),
                    )
                })
                .collect()
        })
        .collect()
}

fn allocated_bytes(build: impl FnOnce() -> MetadataStore) -> (MetadataStore, usize) {
    let epoch_mib = epoch::mib().expect("jemalloc epoch MIB");
    let allocated = stats::allocated::mib().expect("jemalloc allocated MIB");
    epoch_mib.advance().expect("advance jemalloc epoch");
    let before = allocated.read().expect("read allocated bytes");
    let store = build();
    epoch_mib.advance().expect("advance jemalloc epoch");
    let after = allocated.read().expect("read allocated bytes");
    (store, after.saturating_sub(before))
}

fn validate_and_report(size: usize, source: &SourceRows) {
    let owned = MetadataStore::build(Representation::Owned, source.clone());
    let key_interned = MetadataStore::build(Representation::KeyInterned, source.clone());
    let interned = MetadataStore::build(Representation::Interned, source.clone());

    assert_eq!(
        owned.count_equal("category", "category-3"),
        key_interned.count_equal("category", "category-3")
    );
    assert_eq!(
        owned.count_equal("category", "category-3"),
        interned.count_equal("category", "category-3")
    );
    assert_eq!(owned.materialize(), key_interned.materialize());
    assert_eq!(owned.materialize(), interned.materialize());

    drop((owned, key_interned, interned));
    let (_owned, owned_bytes) =
        allocated_bytes(|| MetadataStore::build(Representation::Owned, source_rows(size)));
    let (_key_interned, key_interned_bytes) =
        allocated_bytes(|| MetadataStore::build(Representation::KeyInterned, source_rows(size)));
    let (_interned, interned_bytes) =
        allocated_bytes(|| MetadataStore::build(Representation::Interned, source_rows(size)));

    println!(
        "metadata representation size={size}: owned={owned_bytes} B, key_interned={key_interned_bytes} B ({:+.1}%), interned={interned_bytes} B ({:+.1}%)",
        (key_interned_bytes as f64 / owned_bytes as f64 - 1.0) * 100.0,
        (interned_bytes as f64 / owned_bytes as f64 - 1.0) * 100.0,
    );
}

fn bench_build(c: &mut Criterion) {
    let representation = Representation::from_env();
    let mut group = c.benchmark_group("metadata_representation_build");
    group.sample_size(10);

    for size in [1_000, 10_000, 100_000] {
        let source = source_rows(size);
        validate_and_report(size, &source);
        group.bench_function(format!("size_{size}"), |b| {
            b.iter_batched(
                || source.clone(),
                |source| MetadataStore::build(representation, source),
                BatchSize::LargeInput,
            );
        });
    }
    group.finish();
}

fn bench_filter(c: &mut Criterion) {
    let representation = Representation::from_env();
    let mut group = c.benchmark_group("metadata_representation_filter");

    for size in [1_000, 10_000, 100_000] {
        let store = MetadataStore::build(representation, source_rows(size));
        group.bench_function(format!("size_{size}"), |b| {
            b.iter(|| store.count_equal("category", "category-3"));
        });
    }
    group.finish();
}

fn bench_materialize(c: &mut Criterion) {
    let representation = Representation::from_env();
    let mut group = c.benchmark_group("metadata_representation_materialize");
    group.sample_size(10);

    for size in [1_000, 10_000, 100_000] {
        let store = MetadataStore::build(representation, source_rows(size));
        group.bench_function(format!("size_{size}"), |b| b.iter(|| store.materialize()));
    }
    group.finish();
}

fn criterion_config() -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::from_secs(10))
        .sample_size(10)
}

criterion_group! {
    name = metadata_representation;
    config = criterion_config();
    targets = bench_build, bench_filter, bench_materialize
}
criterion_main!(metadata_representation);
