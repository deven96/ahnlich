# HNSW counter shard memory experiment

All variants share one index-local collector. Control uses Papaya's default
CPU-based counter sharding. Candidates use one or two shards only for per-node
neighbor maps, neighbor sets (including pruning replacements), and backlinks.
The index-wide node map, graph map, and graph-layer membership sets keep default
sharding. Graph parameters, input vectors, and insertion order are identical.

The memory target reuses the collector-sharing fixtures:

| Scenario | Vectors | Dimensions | m | m_zero | ef_construction |
|---|---:|---:|---:|---:|---:|
| `1k_128d_defaults` | 1,000 | 128 | 16 | 32 | 200 |
| `1k_768d_zvec_1m` | 1,000 | 768 | 15 | 30 | 500 |
| `10k_768d_zvec_1m` | 10,000 | 768 | 15 | 30 | 500 |

Cosine distance, serial writes, and both construction flags disabled. These are
synthetic vectors, not the Cohere corpus.

Three fresh subprocesses per variant/scenario (27 total) measure jemalloc
allocated, active, and resident deltas while populated, after deletion of all
nodes, and after dropping the index. Input vectors are generated before the
baseline and retained throughout, excluding their payload from the deltas.
Results are medians per metric, not full-process RSS measurements.

The report records detected CPUs, default shards, architecture, stored nodes,
and live per-node collection counts. It verifies equal structural counts across
samples and variants. On x86_64/ARM64, predicted live-counter savings are:

```text
per-node collections × (default shards − candidate shards) × 128 bytes
```

This prediction excludes retired collections and allocator size-class effects.
It should help interpret the populated allocation difference, not be treated as
an exact prediction for active/resident memory or deletion/drop retention.
On one-CPU systems two shards can use more memory than the default.

Run from the repository root, capturing both the report and terminal output:

```bash
result_dir="$PWD/benchmarks/results/hnsw_counter_shards-$(date +%Y%m%d_%H%M%S)"
mkdir -p "$result_dir"
set -o pipefail

cargo bench --manifest-path ahnlich/similarity/Cargo.toml \
  --features bench-experiments --bench hnsw_counter_shards_memory -- \
  --output "$result_dir/memory.md" \
  2>&1 | tee "$result_dir/memory.txt"
```

Use `bench-experiments` for this report: it explicitly constructs all three
variants. `bench-hnsw-counter-shards` instead routes the DB binary and restore
path to the two-shard candidate for later end-to-end experiments.

The existing `benchmark-experiments.yml` discovers this target under
`ahnlich/similarity/benches/memory/`. Once committed and pushed on a PR, adding
`benchmark-experiments` runs the report and publishes it in the memory-results
comment. No workflow edits are needed.

This report tests construction/deletion/drop memory. It does not benchmark
write performance, concurrent insertion, persistence peaks, or VM RSS.
