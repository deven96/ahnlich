use ahnlich_db::engine::store::StoreHandler;
use ahnlich_types::algorithm::nonlinear::{HnswConfig, non_linear_index};
use ahnlich_types::keyval::{StoreKey, StoreName, StoreValue};
use ahnlich_types::metadata::MetadataValue;
use chrono;
use criterion::{Criterion, criterion_group, criterion_main};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tempfile::TempDir;
use utils::persistence::AhnlichPersistenceUtils;

fn criterion_config(seconds: u64, sample_size: usize) -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::new(seconds, 0))
        .sample_size(sample_size)
}

fn generate_random_metadata() -> HashMap<String, MetadataValue> {
    let categories = ["doc", "image", "video", "audio"];
    let mut metadata = HashMap::new();

    // Text field - simulated snippet
    let text = format!("Sample text content {}", fastrand::u32(..));
    metadata.insert(
        "text".to_string(),
        MetadataValue {
            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                text,
            )),
        },
    );

    // Timestamp field
    let timestamp = chrono::Utc::now().to_rfc3339();
    metadata.insert(
        "timestamp".to_string(),
        MetadataValue {
            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                timestamp,
            )),
        },
    );

    // Category field
    let category = categories[fastrand::usize(..categories.len())];
    metadata.insert(
        "category".to_string(),
        MetadataValue {
            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                category.to_string(),
            )),
        },
    );

    metadata
}

/// Creates a test store populated with random embeddings and optional metadata.
///
/// # Arguments
/// * `size` - Number of entries to generate
/// * `dimension` - Embedding vector dimension (typically 1024)
/// * `with_hnsw` - If true, creates HNSW index with production config
/// * `with_metadata` - If true, adds 3 metadata fields per entry
fn create_test_store_with_data(
    size: usize,
    dimension: usize,
    with_hnsw: bool,
    with_metadata: bool,
) -> (Arc<StoreHandler>, StoreName) {
    let write_flag = Arc::new(AtomicBool::new(false));
    let handler = Arc::new(StoreHandler::new(write_flag));
    let store_name = StoreName {
        value: "benchmark_store".to_string(),
    };

    // Create store with optional HNSW index
    let mut non_linear_indices = HashSet::new();
    if with_hnsw {
        non_linear_indices.insert(non_linear_index::Index::Hnsw(HnswConfig {
            distance: None,
            ef_construction: Some(400),
            maximum_connections: Some(64),
            maximum_connections_zero: None,
            extend_candidates: None,
            keep_pruned_connections: None,
        }));
    }

    handler
        .create_store(
            store_name.clone(),
            NonZeroUsize::new(dimension).unwrap(),
            vec![],
            non_linear_indices,
            true,
        )
        .expect("Failed to create store");

    // Generate entries with embeddings and optional metadata
    let entries: Vec<(StoreKey, StoreValue)> = (0..size)
        .map(|_| {
            let key = StoreKey {
                key: (0..dimension).map(|_| fastrand::f32()).collect(),
            };
            let value = StoreValue {
                value: if with_metadata {
                    generate_random_metadata()
                } else {
                    HashMap::new()
                },
            };
            (key, value)
        })
        .collect();

    handler
        .set_in_store(&store_name, entries)
        .expect("Failed to insert entries");

    (handler, store_name)
}

fn serialize_with_json(
    stores: &ahnlich_db::engine::store::Stores,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, stores)?;
    Ok(())
}

fn deserialize_with_json(path: &Path) -> Result<ahnlich_db::engine::store::Stores, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let stores = serde_json::from_reader(reader)?;
    Ok(stores)
}

fn serialize_with_bitcode(
    stores: &ahnlich_db::engine::store::Stores,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    let encoded = bitcode::serialize(stores)?;
    std::io::Write::write_all(&mut std::io::BufWriter::new(writer), &encoded)?;
    Ok(())
}

fn deserialize_with_bitcode(
    path: &Path,
) -> Result<ahnlich_db::engine::store::Stores, Box<dyn Error>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();
    std::io::Read::read_to_end(&mut reader, &mut buffer)?;
    let stores = bitcode::deserialize(&buffer)?;
    Ok(stores)
}

fn measure_file_size(path: &Path) -> Result<u64, Box<dyn Error>> {
    let metadata = std::fs::metadata(path)?;
    Ok(metadata.len())
}

fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.2} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Benchmark store persistence (serialization and deserialization) across multiple
/// scenarios:
/// - Store sizes: 100, 1K, 10K, 100K entries
/// - Index configs: no-index, HNSW
/// - Formats: JSON (serde_json), bitcode (with serde feature)
///
/// Each benchmark measures:
/// - Serialize: time to write stores to disk + file size
/// - Deserialize: time to read stores from disk + validation
fn bench_persistence(c: &mut Criterion) {
    let sizes = [100, 1000, 10000, 100000];
    let dimension = 1024;

    let mut serialize_group = c.benchmark_group("persistence_serialize");
    serialize_group.sampling_mode(criterion::SamplingMode::Flat);

    for size in sizes {
        // Scenario 1: No index, JSON
        {
            let (handler, _store_name) = create_test_store_with_data(size, dimension, false, true);
            let bench_name = format!("{}_entries_no_index_json", size);

            serialize_group.bench_function(&bench_name, |b| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let file_path = temp_dir.path().join("stores.json");
                    let stores = handler.get_snapshot();
                    serialize_with_json(&stores, &file_path)
                        .expect("Failed to serialize with JSON");

                    // Measure file size
                    let size_bytes = measure_file_size(&file_path).unwrap();
                    println!("File size for {}: {}", bench_name, format_size(size_bytes));
                });
            });
        }

        // Scenario 2: No index, bitcode
        {
            let (handler, _store_name) = create_test_store_with_data(size, dimension, false, true);
            let bench_name = format!("{}_entries_no_index_bitcode", size);

            serialize_group.bench_function(&bench_name, |b| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let file_path = temp_dir.path().join("stores.bitcode");
                    let stores = handler.get_snapshot();
                    serialize_with_bitcode(&stores, &file_path)
                        .expect("Failed to serialize with bitcode");

                    // Measure file size
                    let size_bytes = measure_file_size(&file_path).unwrap();
                    println!("File size for {}: {}", bench_name, format_size(size_bytes));
                });
            });
        }

        // Scenario 3: HNSW index, JSON
        {
            let (handler, _store_name) = create_test_store_with_data(size, dimension, true, true);
            let bench_name = format!("{}_entries_hnsw_json", size);

            serialize_group.bench_function(&bench_name, |b| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let file_path = temp_dir.path().join("stores.json");
                    let stores = handler.get_snapshot();
                    serialize_with_json(&stores, &file_path)
                        .expect("Failed to serialize with JSON");

                    let size_bytes = measure_file_size(&file_path).unwrap();
                    println!("File size for {}: {}", bench_name, format_size(size_bytes));
                });
            });
        }

        // Scenario 4: HNSW index, bitcode
        {
            let (handler, _store_name) = create_test_store_with_data(size, dimension, true, true);
            let bench_name = format!("{}_entries_hnsw_bitcode", size);

            serialize_group.bench_function(&bench_name, |b| {
                b.iter(|| {
                    let temp_dir = TempDir::new().unwrap();
                    let file_path = temp_dir.path().join("stores.bitcode");
                    let stores = handler.get_snapshot();
                    serialize_with_bitcode(&stores, &file_path)
                        .expect("Failed to serialize with bitcode");

                    let size_bytes = measure_file_size(&file_path).unwrap();
                    println!("File size for {}: {}", bench_name, format_size(size_bytes));
                });
            });
        }
    }

    serialize_group.finish();

    let mut deserialize_group = c.benchmark_group("persistence_deserialize");
    deserialize_group.sampling_mode(criterion::SamplingMode::Flat);

    for size in sizes {
        // Scenario 1: No index, JSON
        {
            let (handler, _store_name) = create_test_store_with_data(size, dimension, false, true);
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("stores.json");
            let stores = handler.get_snapshot();
            serialize_with_json(&stores, &file_path).expect("Setup failed");

            let bench_name = format!("{}_entries_no_index_json", size);
            deserialize_group.bench_function(&bench_name, |b| {
                b.iter(|| {
                    let loaded =
                        deserialize_with_json(&file_path).expect("Failed to deserialize with JSON");

                    // Validate roundtrip
                    assert!(!loaded.is_empty(), "Loaded stores should not be empty");
                });
            });
        }

        // Scenario 2: No index, bitcode
        {
            let (handler, _store_name) = create_test_store_with_data(size, dimension, false, true);
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("stores.bitcode");
            let stores = handler.get_snapshot();
            serialize_with_bitcode(&stores, &file_path).expect("Setup failed");

            let bench_name = format!("{}_entries_no_index_bitcode", size);
            deserialize_group.bench_function(&bench_name, |b| {
                b.iter(|| {
                    let loaded = deserialize_with_bitcode(&file_path)
                        .expect("Failed to deserialize with bitcode");

                    assert!(!loaded.is_empty(), "Loaded stores should not be empty");
                });
            });
        }

        // Scenario 3: HNSW index, JSON
        {
            let (handler, _store_name) = create_test_store_with_data(size, dimension, true, true);
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("stores.json");
            let stores = handler.get_snapshot();
            serialize_with_json(&stores, &file_path).expect("Setup failed");

            let bench_name = format!("{}_entries_hnsw_json", size);
            deserialize_group.bench_function(&bench_name, |b| {
                b.iter(|| {
                    let loaded =
                        deserialize_with_json(&file_path).expect("Failed to deserialize with JSON");

                    assert!(!loaded.is_empty(), "Loaded stores should not be empty");
                });
            });
        }

        // Scenario 4: HNSW index, bitcode
        {
            let (handler, _store_name) = create_test_store_with_data(size, dimension, true, true);
            let temp_dir = TempDir::new().unwrap();
            let file_path = temp_dir.path().join("stores.bitcode");
            let stores = handler.get_snapshot();
            serialize_with_bitcode(&stores, &file_path).expect("Setup failed");

            let bench_name = format!("{}_entries_hnsw_bitcode", size);
            deserialize_group.bench_function(&bench_name, |b| {
                b.iter(|| {
                    let loaded = deserialize_with_bitcode(&file_path)
                        .expect("Failed to deserialize with bitcode");

                    assert!(!loaded.is_empty(), "Loaded stores should not be empty");
                });
            });
        }
    }

    deserialize_group.finish();
}

criterion_group! {
    name = persistence;
    config = criterion_config(30, 10);
    targets = bench_persistence
}

criterion_main!(persistence);
