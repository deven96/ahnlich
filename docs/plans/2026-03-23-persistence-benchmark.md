# Persistence Benchmark Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add serialization/deserialization benchmarks comparing serde_json vs bitcode for ahnlich store persistence with realistic data and index configurations.

**Architecture:** Create a new benchmark file `ahnlich/db/benches/persistence.rs` with test data generators, serialization helpers for both formats, and Criterion benchmarks that measure serialize time, file size, and deserialize time across multiple store sizes and index configurations.

**Tech Stack:** Rust, Criterion (benchmarking), serde_json, bitcode (serde feature), tempfile

---

## File Structure

**New Files:**
- `ahnlich/db/benches/persistence.rs` - Main benchmark with all test scenarios

**Modified Files:**
- `ahnlich/Cargo.toml` - Remove bincode workspace dependency
- `ahnlich/db/Cargo.toml` - Add bitcode dev-dependency

---

### Task 1: Remove Unused bincode Dependency

**Files:**
- Modify: `ahnlich/Cargo.toml`

- [ ] **Step 1: Verify bincode is unused**

Run: `rg "use bincode" --type rust`
Expected: No matches (confirming it's safe to remove)

- [ ] **Step 2: Remove bincode from workspace dependencies**

Edit `ahnlich/Cargo.toml`, remove the line:
```toml
bincode = "2.0.1"
```

- [ ] **Step 3: Verify build still works**

Run: `cargo build`
Expected: Success

- [ ] **Step 4: Commit**

```bash
git add ahnlich/Cargo.toml
git commit -m "chore: remove unused bincode dependency"
```

---

### Task 2: Add bitcode Dependency

**Files:**
- Modify: `ahnlich/db/Cargo.toml`

- [ ] **Step 1: Add bitcode to dev-dependencies**

Edit `ahnlich/db/Cargo.toml`, add to `[dev-dependencies]`:
```toml
bitcode = { version = "0.6", features = ["serde"] }
```

- [ ] **Step 2: Verify bitcode builds**

Run: `cargo build --package ahnlich-db --all-features`
Expected: Success (downloads and compiles bitcode)

- [ ] **Step 3: Commit**

```bash
git add ahnlich/db/Cargo.toml
git commit -m "chore: add bitcode dev-dependency for benchmarks"
```

---

### Task 3: Create Benchmark File Skeleton

**Files:**
- Create: `ahnlich/db/benches/persistence.rs`

- [ ] **Step 1: Create file with imports and basic structure**

```rust
use ahnlich_db::engine::store::StoreHandler;
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::{HnswConfig, NonLinearAlgorithm};
use ahnlich_types::keyval::{StoreKey, StoreName, StoreValue};
use ahnlich_types::metadata::MetadataValue;
use criterion::{criterion_group, criterion_main, Criterion};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::num::NonZeroUsize;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tempfile::TempDir;

fn criterion_config(seconds: u64, sample_size: usize) -> Criterion {
    Criterion::default()
        .measurement_time(std::time::Duration::new(seconds, 0))
        .sample_size(sample_size)
}

fn bench_persistence(_c: &mut Criterion) {
    // TODO: implement benchmarks
}

criterion_group! {
    name = persistence;
    config = criterion_config(30, 10);
    targets = bench_persistence
}

criterion_main!(persistence);
```

- [ ] **Step 2: Verify benchmark compiles**

Run: `cargo bench --bench persistence --no-run`
Expected: Success (benchmark compiles but doesn't run)

- [ ] **Step 3: Commit**

```bash
git add ahnlich/db/benches/persistence.rs
git commit -m "feat: add persistence benchmark skeleton"
```

---

### Task 4: Implement Test Data Generator

**Files:**
- Modify: `ahnlich/db/benches/persistence.rs`

- [ ] **Step 1: Add helper to generate random metadata**

Add function before `bench_persistence`:
```rust
fn generate_random_metadata() -> HashMap<String, MetadataValue> {
    let categories = ["doc", "image", "video", "audio"];
    let mut metadata = HashMap::new();
    
    // Text field - simulated snippet
    let text = format!("Sample text content {}", fastrand::u32(..));
    metadata.insert(
        "text".to_string(),
        MetadataValue {
            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(text)),
        },
    );
    
    // Timestamp field
    let timestamp = chrono::Utc::now().to_rfc3339();
    metadata.insert(
        "timestamp".to_string(),
        MetadataValue {
            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(timestamp)),
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
```

- [ ] **Step 2: Add imports for chrono**

Add to `ahnlich/db/Cargo.toml` dev-dependencies:
```toml
chrono = { version = "0.4", default-features = false, features = ["std", "clock"] }
```

Add to imports in `persistence.rs`:
```rust
use chrono;
```

- [ ] **Step 3: Add test data generator function**

```rust
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
        non_linear_indices.insert(NonLinearAlgorithm::Hnsw(HnswConfig {
            ef_construction: NonZeroUsize::new(400).unwrap(),
            m: NonZeroUsize::new(64).unwrap(),
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
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo bench --bench persistence --no-run`
Expected: Success

- [ ] **Step 5: Commit**

```bash
git add ahnlich/db/benches/persistence.rs ahnlich/db/Cargo.toml
git commit -m "feat: add test data generator for persistence benchmarks"
```

---

### Task 5: Implement Serialization Helpers

**Files:**
- Modify: `ahnlich/db/benches/persistence.rs`

- [ ] **Step 1: Add JSON serialization helpers**

```rust
fn serialize_with_json(
    stores: &ahnlich_db::engine::store::Stores,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    let file = File::create(path)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer(writer, stores)?;
    Ok(())
}

fn deserialize_with_json(
    path: &Path,
) -> Result<ahnlich_db::engine::store::Stores, Box<dyn Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let stores = serde_json::from_reader(reader)?;
    Ok(stores)
}
```

- [ ] **Step 2: Add bitcode serialization helpers**

```rust
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
```

- [ ] **Step 3: Add file size measurement helper**

```rust
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
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo bench --bench persistence --no-run`
Expected: Success

- [ ] **Step 5: Commit**

```bash
git add ahnlich/db/benches/persistence.rs
git commit -m "feat: add serialization helpers for JSON and bitcode"
```

---

### Task 6: Implement Serialize Benchmarks

**Files:**
- Modify: `ahnlich/db/benches/persistence.rs`

- [ ] **Step 1: Replace bench_persistence with full implementation**

```rust
fn bench_persistence(c: &mut Criterion) {
    let sizes = [100, 1000, 10000, 100000];
    let dimension = 1024;
    
    let mut serialize_group = c.benchmark_group("persistence_serialize");
    serialize_group.sampling_mode(criterion::SamplingMode::Flat);
    
    for size in sizes {
        // Scenario 1: No index, JSON
        {
            let (handler, store_name) = create_test_store_with_data(size, dimension, false, true);
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
}
```

- [ ] **Step 2: Run benchmark to verify it works**

Run: `cargo bench --bench persistence serialize -- 100_entries`
Expected: Benchmarks run successfully for 100 entry tests

- [ ] **Step 3: Commit**

```bash
git add ahnlich/db/benches/persistence.rs
git commit -m "feat: implement serialize benchmarks for all scenarios"
```

---

### Task 7: Implement Deserialize Benchmarks

**Files:**
- Modify: `ahnlich/db/benches/persistence.rs`

- [ ] **Step 1: Add deserialize group after serialize_group.finish()**

```rust
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
                    let loaded = deserialize_with_json(&file_path)
                        .expect("Failed to deserialize with JSON");
                    
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
                    let loaded = deserialize_with_json(&file_path)
                        .expect("Failed to deserialize with JSON");
                    
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
```

- [ ] **Step 2: Run benchmark to verify it works**

Run: `cargo bench --bench persistence deserialize -- 100_entries`
Expected: Benchmarks run successfully, validation passes

- [ ] **Step 3: Commit**

```bash
git add ahnlich/db/benches/persistence.rs
git commit -m "feat: implement deserialize benchmarks with validation"
```

---

### Task 8: Final Testing and Documentation

**Files:**
- Modify: `ahnlich/db/benches/persistence.rs`

- [ ] **Step 1: Run full benchmark suite**

Run: `cargo bench --bench persistence`
Expected: All 32 benchmarks complete successfully (4 sizes × 2 indices × 2 formats × 2 operations)

- [ ] **Step 2: Verify file sizes are printed**

Check stdout during benchmark run
Expected: See lines like "File size for 1000_entries_no_index_json: X.XX MB"

- [ ] **Step 3: Add doc comments to main functions**

Add at top of `bench_persistence`:
```rust
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
```

Add to helper functions:
```rust
/// Creates a test store populated with random embeddings and optional metadata.
/// 
/// # Arguments
/// * `size` - Number of entries to generate
/// * `dimension` - Embedding vector dimension (typically 1024)
/// * `with_hnsw` - If true, creates HNSW index with production config
/// * `with_metadata` - If true, adds 3 metadata fields per entry
fn create_test_store_with_data(
```

- [ ] **Step 4: Run quick smoke test**

Run: `cargo bench --bench persistence -- 100_entries_no_index`
Expected: 4 benchmarks complete in reasonable time

- [ ] **Step 5: Final commit**

```bash
git add ahnlich/db/benches/persistence.rs
git commit -m "docs: add documentation to persistence benchmark functions"
```

---

### Task 9: Verify Success Criteria

**Files:**
- None (validation only)

- [ ] **Step 1: Verify all benchmarks run**

Run: `cargo bench --bench persistence 2>&1 | grep "entries_" | wc -l`
Expected: 32 (all test scenarios)

- [ ] **Step 2: Verify bincode is removed**

Run: `rg "bincode" ahnlich/Cargo.toml`
Expected: No matches

- [ ] **Step 3: Verify bitcode is added**

Run: `rg "bitcode" ahnlich/db/Cargo.toml`
Expected: Finds dev-dependency entry

- [ ] **Step 4: Run specific benchmark filters**

Run: `cargo bench --bench persistence serialize`
Expected: Only serialize benchmarks run (16 total)

Run: `cargo bench --bench persistence deserialize`
Expected: Only deserialize benchmarks run (16 total)

Run: `cargo bench --bench persistence 10000_entries`
Expected: Only 10K entry benchmarks run (8 total)

- [ ] **Step 5: Create summary document**

Create file or notes with:
- Command to run benchmarks: `cargo bench --bench persistence`
- Expected output examples
- How to interpret results (time, throughput, file sizes)
- Link to spec: `docs/specs/2026-03-23-persistence-benchmark-design.md`

---

## Running the Complete Implementation

Execute tasks 1-9 in order. Each task is independent and can be committed separately.

**Estimated time:** 45-60 minutes

**Key commands:**
```bash
# Run all benchmarks
cargo bench --bench persistence

# Run only serialize benchmarks
cargo bench --bench persistence serialize

# Run only deserialize benchmarks  
cargo bench --bench persistence deserialize

# Run specific size
cargo bench --bench persistence 10000_entries
```

**Success indicators:**
- All 32 benchmarks complete without errors
- File sizes printed to stdout during serialize benchmarks
- Deserialize benchmarks validate roundtrip correctness
- Results reproducible across runs (within statistical variance)
