# Persistence Benchmark Design

**Date:** 2026-03-23  
**Status:** Approved  
**Author:** OpenCode

## Overview

Add comprehensive serialization/deserialization benchmarks for ahnlich store persistence to measure the performance characteristics of JSON (current) vs bitcode (potential alternative) serialization formats. This establishes baseline metrics for persistence operations and enables data-driven decisions about serialization format choices.

## Background

Currently, ahnlich uses `serde_json` for persisting stores to disk (`ahnlich/utils/src/persistence.rs`). While JSON is human-readable and easy to debug, it may have performance and storage size limitations at scale. This benchmark will quantify actual serialization performance with realistic store configurations including embeddings, metadata, and indices.

## Goals

1. Establish baseline performance metrics for current JSON persistence
2. Compare JSON vs bitcode (with serde feature) for serialization speed, file size, and deserialization speed
3. Measure impact of HNSW indices on persistence performance
4. Test at production-relevant scales: 100, 1K, 10K, 100K entries
5. Use realistic test data including metadata and index configurations

## Non-Goals

- Testing other serialization formats (bincode, rkyv, postcard, etc.)
- Benchmarking in-memory serialization (we measure file I/O matching real persistence)
- Testing AI store persistence (db stores only for now)
- Compression benchmarks (zstd, zlib)

## Architecture

### File Structure

**New Files:**
- `ahnlich/db/benches/persistence.rs` - Main persistence benchmark

**Modified Files:**
- `ahnlich/Cargo.toml` - Remove unused `bincode` dependency
- `ahnlich/db/Cargo.toml` - Add `bitcode` dev-dependency

### Dependencies

**Add to `ahnlich/db/Cargo.toml` dev-dependencies:**
```toml
bitcode = { version = "0.6", features = ["serde"] }
```

**Remove from `ahnlich/Cargo.toml` workspace dependencies:**
```toml
bincode = "2.0.1"  # No longer used anywhere in codebase
```

### Benchmark Organization

```
bench_persistence()
├── For each size: [100, 1000, 10000, 100000]
│   ├── Scenario: no-index
│   │   ├── JSON: serialize → measure file size → deserialize
│   │   └── Bitcode: serialize → measure file size → deserialize
│   └── Scenario: HNSW index
│       ├── JSON: serialize → measure file size → deserialize
│       └── Bitcode: serialize → measure file size → deserialize
```

**Criterion Groups:**
- `persistence_serialize` - Write performance benchmarks
- `persistence_deserialize` - Read performance benchmarks
- Configuration: 30s measurement time, 10 samples (matching existing db benchmarks)

## Implementation Details

### Test Data Generation

**Function:**
```rust
fn create_test_store_with_data(
    size: usize,
    dimension: usize,
    with_hnsw: bool,
    with_metadata: bool
) -> (StoreHandler, StoreName)
```

**Configuration:**
- **Dimension:** 1024 (consistent with existing benchmarks)
- **Store count:** 1 (single store per test)
- **Entries:** Variable (100, 1K, 10K, 100K)

**Metadata Setup (`with_metadata = true`):**
Each `StoreValue` contains 3 metadata fields:
- `"text"`: RawString - simulated text snippet (~50 chars)
- `"timestamp"`: RawString - ISO8601 format timestamp
- `"category"`: RawString - random from ["doc", "image", "video", "audio"]

**HNSW Setup (`with_hnsw = true`):**
- Algorithm: `Algorithm::CosineSimilarity`
- Config: `HnswConfig { ef_construction: 400, m: 64 }`
- Index fully built before serialization begins

### Serialization Implementation

**File I/O Pattern** (matches current `persistence.rs`):

```rust
// Serialize:
1. Get snapshot: stores = handler.get_snapshot()
2. Create temp file in benchmark temp directory
3. Serialize to temp file (serde_json or bitcode)
4. Atomic rename temp → final location
5. Measure file size with std::fs::metadata()

// Deserialize:
1. Open persisted file
2. Deserialize from file
3. Validate correctness (store count, sample entry dimension)
```

**Helper Functions:**
```rust
fn serialize_with_json(stores: &Stores, path: &Path) -> Result<(), Box<dyn Error>>
fn serialize_with_bitcode(stores: &Stores, path: &Path) -> Result<(), Box<dyn Error>>

fn deserialize_with_json(path: &Path) -> Result<Stores, Box<dyn Error>>
fn deserialize_with_bitcode(path: &Path) -> Result<Stores, Box<dyn Error>>
```

**Error Handling:**
- Helpers return `Result<T, Box<dyn Error>>`
- Benchmarks panic on error with descriptive messages
- Use `.expect()` with clear context: `"Failed to serialize stores with JSON"`

**Temporary Files:**
- Use `tempfile::Builder::new().prefix("ahnlich_bench_").tempdir()`
- One temp directory per benchmark iteration
- Auto-cleanup when TempDir drops
- File naming: `{temp_dir}/stores.json` or `{temp_dir}/stores.bitcode`

### Metrics & Output

**Criterion Output:**

For each test case (e.g., `1000_entries_no_index_json_serialize`):
- **Time:** Mean with confidence interval (auto by Criterion)
- **Throughput:** Elements/second (auto by Criterion)

**Benchmark Naming Convention:**
```
{size}_entries_{index_type}_{format}_{operation}

Examples:
- 100_entries_no_index_json_serialize
- 1000_entries_hnsw_bitcode_deserialize
- 10000_entries_no_index_json_serialize
- 100000_entries_hnsw_bitcode_serialize
```

**File Size Reporting:**

After each serialize benchmark, print to stdout:
```
File size for 1000_entries_no_index_json: 2.34 MB
File size for 1000_entries_no_index_bitcode: 0.89 MB (38% of JSON)
File size for 1000_entries_hnsw_json: 5.67 MB
File size for 1000_entries_hnsw_bitcode: 2.12 MB (37% of JSON)
```

**Validation:**

Each deserialize benchmark includes assertions:
- Store count matches original
- Random sample entry has correct embedding dimension
- Panic with clear message if roundtrip validation fails

## Test Scenarios

**Matrix of tests (4 sizes × 2 index configs × 2 formats × 2 operations = 32 benchmarks):**

| Size | Index | Format | Operation |
|------|-------|--------|-----------|
| 100 | none | JSON | serialize |
| 100 | none | JSON | deserialize |
| 100 | none | bitcode | serialize |
| 100 | none | bitcode | deserialize |
| 100 | HNSW | JSON | serialize |
| 100 | HNSW | JSON | deserialize |
| 100 | HNSW | bitcode | serialize |
| 100 | HNSW | bitcode | deserialize |
| ... | ... | ... | ... |
| 100K | HNSW | bitcode | deserialize |

## Running the Benchmarks

```bash
# Run all persistence benchmarks
cargo bench --bench persistence

# Run only serialize benchmarks
cargo bench --bench persistence serialize

# Run only deserialize benchmarks
cargo bench --bench persistence deserialize

# Run specific size
cargo bench --bench persistence 10000_entries
```

## Success Criteria

1. Benchmarks run successfully for all 32 test cases
2. File sizes are reported to stdout with JSON/bitcode comparison
3. Roundtrip validation passes for all deserialize tests
4. Results are reproducible across runs (within statistical variance)
5. Bincode dependency is fully removed from workspace

## Expected Outcomes

Based on external benchmarks, we expect:
- **Bitcode serialize:** 10-20x faster than JSON
- **Bitcode deserialize:** 4-10x faster than JSON
- **Bitcode file size:** 60-70% smaller than JSON
- **HNSW impact:** Significant increase in serialization time and file size for both formats

## Future Enhancements

- Add compression benchmarks (zstd on top of JSON vs bitcode)
- Benchmark AI store persistence separately
- Test native bitcode `Encode`/`Decode` traits (requires refactoring away from serde)
- Add predicate index configurations to test matrix
- Memory usage profiling during serialization
