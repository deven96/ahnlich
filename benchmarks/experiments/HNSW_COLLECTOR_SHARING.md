# HNSW collector sharing candidate

The Cohere 100K diagnostic retained 8.34 GiB of allocator memory with HNSW,
versus 0.42 GiB with linear search. This candidate tests whether independent
Papaya reclamation collectors contribute to that footprint. It makes no claim
about memory savings until measured.

The default DB build uses `HNSW::new_with_shared_collector` and a shared collector per
index. The original constructors remain available for the control. Similarity
benchmarks can enable `bench-experiments` and call
`new_with_config` (control) and `new_with_config_and_shared_collector` in the same binary.

The candidate shares one collector per HNSW index across its node map, layer
map, graph-layer sets, per-node neighbor maps, per-layer neighbor sets, and
backlink sets. Pruning replacements use that collector too. Capacities and HNSW
parameters are preserved. Query-only temporary nodes retain the existing path.

Persistence contains no collector state or experiment marker. DB builds
restore all nested collections with one fresh index-local collector. The
persisted data format is unchanged.

Shared reclamation lets a guard delay cleanup across the whole index. There is
also a nested ownership cycle: retired node/maps can own references to their
collector. The candidate drains retired objects during exclusive index drop,
after clearing its private maps and ending their guards. This uses
`Collector::reclaim_all`; the safety argument relies on collector privacy and
exclusive access during drop. Node snapshots clone onto independent collectors.
Tests cover drop after serial inserts from multiple worker threads.

For the next diagnostic, build the control from the pre-migration revision
`1c848f3a` and the candidate from this checkout in separate target folders,
then give each binary to the existing VectorDBBench `diagnose_ahnlich_memory`
tool. Keep Small Cohere, loading concurrency 1, batch size 100, and
`m=15`, `m_zero=30`, `ef_construction=500`, `ef_search=180` identical. Compare
allocator usage and RSS during loading and the idle minute. No concurrent-write
fix or graph/search algorithm change is included.


## Local experiments

Criterion compares construction/drop, initial insertion, insertion into a populated
index, search, deletion, and populated-index drop. Deterministic vectors, cosine
distance, input order, and HNSW parameters are identical across paired variants.
Fixture generation and initial population are outside the timed routine where
appropriate. Insert/delete output destruction is outside the timed routine;
`drop_populated` explicitly measures destruction. No concurrent writes are used.

The memory target follows #424: three isolated processes per variant/scenario,
reporting median jemalloc allocated/active/resident deltas while populated,
after deleting all nodes, and after index drop. Input vectors are held across
all phases and excluded from the baseline delta. Jemalloc resident is not process
RSS. Scenarios cover 1K/128D defaults and 1K/768D plus 10K/768D with the zvec 1M
parameters. Results from CI are not substitutes for measurements on the VM.

```bash
result_dir="$PWD/benchmarks/results/hnsw_collector_sharing-$(date +%Y%m%d_%H%M%S)"
mkdir -p "$result_dir"
set -o pipefail
cargo bench --manifest-path ahnlich/similarity/Cargo.toml \
  --features bench-experiments --bench hnsw_collector_sharing \
  2>&1 | tee "$result_dir/criterion.txt"
cargo bench --manifest-path ahnlich/similarity/Cargo.toml \
  --features bench-experiments --bench hnsw_collector_memory -- \
  --output "$result_dir/memory.md" \
  2>&1 | tee "$result_dir/memory.txt"
```

Both targets are discovered by `benchmark-experiments.yml` when newly added or
modified under `benches/experiments` and `benches/memory`. Add the
`benchmark-experiments` PR label to run them. New Criterion targets compare
`candidate` and `control` within the PR binary; memory targets publish their
isolated A/B Markdown reports. Use `bench-experiments` to compare both control
and candidate in the same binary.
