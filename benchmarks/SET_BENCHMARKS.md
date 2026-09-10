# ahnlich-db Set benchmark harness

Measures end-to-end `Set` QPS for ahnlich-db under concurrent gRPC load.

For the `GetSimN` search benchmark, see
[SEARCH_BENCHMARKS.md](SEARCH_BENCHMARKS.md). It uses SIFT fixtures and a separate ghz runner.

## Requirements

- Rust toolchain and protoc
- [`ghz`](https://ghz.sh): `brew install ghz` or `go install github.com/bojand/ghz/cmd/ghz@latest`

Set uses deterministic synthetic vectors because ingestion performance depends on vector
size, batch size, metadata, and indexes rather than the vector distribution. This also
provides unlimited stable keys and directly controllable metadata cardinality.

## Running

```bash
./run_set.sh
```

Run a prebuilt server binary with:

```bash
SERVER_BIN=/path/to/ahnlich-db ./run_set.sh
```

Builds the release binaries when needed, starts a server, creates the stores, runs the
sweep, verifies the stored data, and writes a summary. Paths resolve relative to the
script. The server is stopped on exit, including on failure and Ctrl-C.

Output lands in `results/set_<timestamp>/`:

```text
SUMMARY.md                  results table
RUN.txt                     commit, host, ghz version, configuration
<label>_c<N>_r<K>.json      raw ghz report per run
<label>_c<N>_r<K>.cpu       server CPU per request
payloads/                   generated ghz request data
specs/                      exact fixture definitions
server.log
```

## What it measures

Each store contains deterministic 128-dimensional vectors with four metadata fields and
a 64-character unindexed payload. The measured requests update a bounded pool of existing
keys with the same values, keeping the store size and operation type stable across repeats.

| row | predicate indexes | indexed value cardinality |
|---|---:|---:|
| `set_no_index` | 0 | 100 |
| `set_1_index` | 1 | 100 |
| `set_4_indexes` | 4 | 100 |

The zero-index row is the negative control for predicate ingestion. Every indexed store
contains a sentinel membership created before timing. No HNSW or KD-tree index is created.

Payload files are generated once and reused for every concurrency level and repeat. The
first two vector coordinates encode request and row identity, allowing the harness to
verify the stored values after the sweep.

## Reference numbers

M1 development machine, control `ab4ffc70` versus `ab4ffc70-working-tree`, three repeats
per case. All requests returned `OK` and all post-run state checks passed. Values below are
the median candidate RPS change; positive values favor the candidate.

| existing indexes | batch | c=1 | c=4 | c=16 |
|---:|---:|---:|---:|---:|
| 0 | 100 | +0.2% | +0.4% | +4.7% |
| 1 | 100 | +9.0% | +20.0% | +33.8% |
| 4 | 100 | +35.0% | +58.0% | +92.9% |
| 0 | 1,000 | +0.4% | -0.1% | +3.1% |
| 1 | 1,000 | +3.6% | +9.0% | +14.3% |
| 4 | 1,000 | +12.6% | +22.8% | +38.7% |

At batch 100 with four existing indexes and concurrency 16, throughput increased from
376.2 to 725.6 RPS and p50 latency fell from 43.14 ms to 21.85 ms.

## Configuration

| variable | default | notes |
|---|---|---|
| `HOST` / `PORT` | `127.0.0.1` / `1369` | |
| `CONCURRENCY_LEVELS` | `1 10 50 100` | space separated |
| `SCENARIOS` | all scenarios | space-separated scenario names to run |
| `TOTAL_REQUESTS` | `10000` | measured requests per run |
| `WARMUP_REQUESTS` | `500` | issued first, excluded from stats |
| `REPEATS` | `3` | runs per configuration |
| `CONNECTIONS` | `8` | ghz connections; capped at concurrency |
| `BATCH_SIZE` | `1000` | entries per `Set` request |
| `VECTOR_DIMENSION` | `128` | floats per vector |
| `POOL_REQUESTS` | `32` | distinct request batches cycled by ghz |
| `THREADPOOL_SIZE` | `16` | server Rayon pool size |
| `SIZE_CALCULATION_INTERVAL` | `60` | milliseconds |
| `REQUEST_TIMEOUT` | `60s` | |
| `SERVER_BIN` | built locally | optional prebuilt `ahnlich-db` binary |
| `RESULTS_DIR` | timestamped | |

```bash
BATCH_SIZE=100 CONCURRENCY_LEVELS="1 4 16" ./run_set.sh
```

To compare two revisions, build both binaries and run the same configuration into separate
directories:

```bash
SERVER_BIN=/tmp/control/ahnlich-db RESULTS_DIR=/tmp/set-control ./run_set.sh
SERVER_BIN=/tmp/candidate/ahnlich-db RESULTS_DIR=/tmp/set-candidate ./run_set.sh
```

## Reading the results

- Every figure is the median of `REPEATS` runs. `RPS range` is the spread across them;
  changes smaller than the spread are not measurable.
- `server us/req` is the server process's own CPU time. It excludes client cost.
- `RPS` includes client cost. At concurrency 1 ghz is a larger part of the round trip, so
  use higher concurrency for throughput and concurrency 1 to isolate per-request changes.
- `set_no_index` shows whether a change affects general `Set` processing independently of
  predicate indexes.
- Store length, predicate configuration, sentinel membership, and sampled values are
  verified after the measured runs. Any non-`OK` response fails the run.

## Profiling

Run the server under [samply](https://github.com/mstange/samply) and drive it with ghz.

```bash
cargo install samply --locked
```

Build with symbols first, or the profile is only addresses:

```bash
CARGO_PROFILE_RELEASE_DEBUG=line-tables-only \
  cargo build --release --manifest-path ../ahnlich/Cargo.toml --bin ahnlich-db
```

Use the generated payload for the scenario being profiled and call
`services.db_service.DBService/Set`. Keep the batch size, concurrency, connections, and
request count identical when comparing profiles.

When reading a profile:

- Select only the ghz measurement window; store creation and preload happen earlier.
- Filter to the `ahnlich-db` binary. Parked Rayon threads are sampled too.
- Compare the CPU accounted for against the measured latency. A large gap means time was
  spent waiting rather than in the top self-time entry.

Confirm a profile finding by running the same configuration against both prebuilt binaries.

## Layout

Standalone crate with its own lockfile, outside the ahnlich workspace. It depends on
ahnlich by path and is not built by `cargo build --workspace`.
