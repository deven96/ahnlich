# ahnlich-db benchmark harness

Measures end-to-end QPS for ahnlich-db under concurrent gRPC load.

## Requirements

- Rust toolchain
- [`ghz`](https://ghz.sh): `brew install ghz` or `go install github.com/bojand/ghz/cmd/ghz@latest`

The 10k SIFT set ships with the repo at `../ahnlich/similarity/sift/`. For larger runs,
point `SIFT_DIR` at [SIFT1M](http://corpus-texmex.irisa.fr/).

## Running

```bash
./run_baseline.sh
```

Builds the release binaries, starts a server, loads the data, runs the sweep, writes a
summary. Paths resolve relative to the script. The server is stopped on exit, including
on failure and Ctrl-C.

Output lands in `results/baseline_<timestamp>/`:

```
SUMMARY.md                  results table
RUN.txt                     commit, host, ghz version, configuration
<label>_c<N>_r<K>.json      raw ghz report per run
<label>_c<N>_r<K>.cpu       server CPU per request
payloads/                   generated ghz request data
server.log
```

## What it measures

Both stores hold the same vectors. Only the search path differs.

| row | store | index | query algorithm |
|---|---|---|---|
| `linear` | `sift_linear` | none | matches `DISTANCE_METRIC` |
| `hnsw` | `sift_hnsw` | HNSW | `HNSW` |
| `ping` | none | none | `DBService/Ping`, does no work |

## Configuration

| variable | default | notes |
|---|---|---|
| `HOST` / `PORT` | `127.0.0.1` / `1369` | |
| `CONCURRENCY_LEVELS` | `1 10 50 100` | space separated |
| `TOTAL_REQUESTS` | `10000` | measured requests per run |
| `WARMUP_REQUESTS` | `500` | issued first, excluded from stats |
| `REPEATS` | `3` | runs per configuration |
| `CONNECTIONS` | `8` | ghz connections; capped at the concurrency level |
| `DISTANCE_METRIC` | `euclidean` | `euclidean` \| `cosine` \| `dotproduct` |
| `CLOSEST_N` | `10` | |
| `STORE_SIZE` | whole dataset | truncates only; errors if larger than the dataset |
| `EF_CONSTRUCTION` | `100` | matches the server default; drives build time and recall |
| `BATCH_SIZE` | `2000` | vectors per `Set` request |
| `THREADPOOL_SIZE` | `16` | rayon pool size |
| `SIFT_DIR` | `../ahnlich/similarity/sift` | |
| `REQUEST_TIMEOUT` | `60s` | |
| `RESULTS_DIR` | timestamped | |

```bash
TOTAL_REQUESTS=50000 CONCURRENCY_LEVELS="1 4 8 16 32" ./run_baseline.sh
```

## Reading the results

- Every figure is the median of `REPEATS` runs. `RPS range` is the spread across them;
  changes smaller than the spread are not measurable.
- `server µs/req` is the server process's own CPU time. It excludes client cost.
- `RPS` includes client cost. At concurrency 1 ghz is about half the round trip, so use
  concurrency 10 and above for throughput figures, and concurrency 1 for A/B comparisons.
- Entries are stored with empty metadata. The value path and predicates are not exercised.
- `ping` does no work, so it bounds every other row.

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

samply launches the process. Attaching with `-p <pid>` is Linux only.

```bash
samply record -- ../ahnlich/target/release/ahnlich-db run --port 1399
```

In another shell:

```bash
AHNLICH_DB_ADDR=127.0.0.1:1399 PAYLOAD_DIR=/tmp/prof \
  cargo run --release --bin setup_sift

ghz --insecure \
    --proto ../protos/services/db_service.proto --import-paths ../protos \
    --call services.db_service.DBService/GetSimN \
    --data-file /tmp/prof/getsimn_hnsw.json \
    --concurrency 1 --connections 1 --total 30000 \
    localhost:1399
```

Kill the server and samply opens the Firefox Profiler.

When reading it:

- Select the ghz window in the timeline. The recording also covers the index build, which
  is far longer than the queries.
- Filter to the `ahnlich-db` binary. Parked rayon threads are sampled too and reached 73%
  of one profile.
- Compare the CPU the profile accounts for against the measured latency. A large gap
  means the time went on waiting, not on the top self-time entry.

Confirm what a profile suggests with an A/B at `CONCURRENCY_LEVELS="1"`, watching the
ping row for machine drift.

## Layout

Standalone crate with its own lockfile, outside the ahnlich workspace. It depends on
ahnlich by path and is not built by `cargo build --workspace`.
