# Existing predicate index: end-to-end Set A/B

`run_set_ab.py` compares a prebuilt main/control server with a prebuilt candidate server
using ghz over gRPC. It never builds, changes branches, or contacts an existing deployment.
It starts fresh standalone servers on localhost, one at a time, and stops its own servers
on completion or failure. Results directories must be new.

The candidate binary uses the opt-in `bench-existing-predicate-index` Cargo feature.
This routes Set's predicate ingestion to `add_existing_index_candidate`, the same candidate
used by the first Criterion experiment. The borrowed-metadata candidate is not enabled.
Default builds retain the current implementation. Do not enable this feature in release
artifacts intended for production.

## Build, then run

Requires Rust/protoc for building, Python 3.11+, and ghz (CI pins v0.121.0).
From the repository root, prepare an isolated checkout of the control revision:

```sh
git worktree add --detach /tmp/ahnlich-set-main main

cargo build --release --manifest-path /tmp/ahnlich-set-main/ahnlich/Cargo.toml \
  --bin ahnlich-db --target-dir /tmp/ahnlich-set-control-target

cargo build --release --manifest-path ahnlich/Cargo.toml --bin ahnlich-db \
  --features bench-existing-predicate-index --target-dir /tmp/ahnlich-set-candidate-target

cargo build --release --manifest-path benchmarks/Cargo.toml --bin setup_set \
  --target-dir /tmp/ahnlich-set-harness-target

python3 benchmarks/run_set_ab.py \
  --control-bin /tmp/ahnlich-set-control-target/release/ahnlich-db \
  --candidate-bin /tmp/ahnlich-set-candidate-target/release/ahnlich-db \
  --setup-bin /tmp/ahnlich-set-harness-target/release/setup_set \
  --control-ref "$(git -C /tmp/ahnlich-set-main rev-parse HEAD)" \
  --candidate-ref "$(git rev-parse HEAD)-working-tree" \
  --results benchmarks/results/set_existing_index_ab
```

Use unused paths or your existing worktree/output directories. For reproducible source
attribution, build committed revisions and pass their exact SHAs. Ref labels are supplied
by the operator; the runner records binary SHA-256 hashes but cannot infer their source
commits. Use `--setup-bin` when Cargo output is redirected.

The default sweep covers batches 100/1,000, concurrency 1/4/16, three repeats, and four
metadata cases: zero indexed fields (negative control), one indexed field with one value,
one indexed field with 100 values, and four indexed fields with 100 values. The first
binary alternates by repeat: control/candidate, candidate/control, control/candidate.
Budget roughly 45 minutes plus setup for the full default sweep.

For a narrower first comparison, append:

```sh
--batches 1000 --concurrency 1 4 --cases 0:100 1:100 4:100 --seconds 15 --repeats 3
```

## Workloads and fixture identity

Every measured store contains a sentinel entry before timing. Writing that entry populates
the configured predicate indexes, and the helper checks each predicate's sentinel query.
No HNSW or KD-tree index is created. Each payload has the same four metadata fields and
a 64-character unindexed payload regardless of the indexed-field count.

| Workload | Preparation | Measured calls |
|---|---|---|
| `update` (default) | Preload a bounded pool of 32 request batches | Cycle the same batches for 15 seconds; all keys already exist and metadata values remain unchanged |
| `insert` | Preload only the sentinel | Send exactly `--total-requests` disjoint batches once each |
| `mixed` | Preload the first half of every request batch | Send each batch once; first half updates existing keys, remaining rows insert new keys |

Vectors are deterministic synthetic fixtures, default dimension 128. Their first two
coordinates encode request and row identity using exactly representable f32 integers.
This requires no SIFT download and isolates ingestion rather than similarity quality.
The index value distribution is controlled by `--cases indexed-fields:cardinality`.

Payload files are generated once per scenario and reused byte-for-byte for both binaries
and all repeats. The [ghz array-input contract](https://ghz.sh/docs/options#-d---data)
cycles messages round-robin. Templates are disabled. Insert/mixed use a request count
equal to the array length to avoid cycling into an update workload; final store-length
checks detect missing distinct entries. Update intentionally cycles a bounded key set.

Growth workloads require more memory and larger input files. The runner defaults to a
200,000-entry fixture cap. For example, append these settings to the same command:

```sh
--workloads insert mixed --batches 100 --total-requests 2000 \
--concurrency 1 4 --cases 1:100 4:100
```

Runs under ten seconds are flagged in the summary; increase the request budget and the
explicit `--max-fixture-entries` limit as your machine permits. The cap counts vectors,
not bytes; account for dimensions, payload JSON, metadata, and index memory. Inspect client
CPU because a large payload pool can make ghz the bottleneck.

## Timing and validation

- Every binary/scenario/concurrency/repeat uses a fresh server, preventing store growth,
  old buckets, or allocator state from leaking across runs.
- Warmup uses a separate, bounded update store for three seconds. The measured store is
  prepared afterward. Warmup traffic, fixture setup, and verification are excluded from
  ghz's measured report. The warmup store remains present in both variants.
- Warmup and measurement use separate ghz invocations. Connections are re-established for
  measurement; duration runs should be long enough to amortize their startup cost.
- The server Rayon pool defaults to 16 threads. Predicate parallelism keeps the normal
  150,000 batch threshold; this experiment does not force the parallel branch as the
  Criterion grouping experiment did. Override `--parallel-batch-threshold` explicitly
  for a separate parallel-policy run.
- Connections default to eight, capped at concurrency. Defaults pin message/allocator
  limits and set size calculation to 60,000 ms to reduce background scanning during the
  short isolated runs. The usual default is 60 ms; rerun with
  `--size-calculation-interval 60` to assess that production background load.
- Persistence, authentication, and replication are not enabled. This first experiment
  isolates standalone Set with predicate indexing.
- Every ghz invocation must report a positive count, only OK statuses, and no errors.
  Insert/mixed must complete the exact configured count. Duration runs wait for in-flight
  requests, rather than ignoring their errors at the cutoff.
- After measurement, the helper verifies exact store length, configured predicate names,
  sentinel predicate membership, and sampled first/last request entries including metadata.
  These checks do not exhaustively validate every result or concurrent index lifecycle.
- `expected_inserted` and `expected_updated` in `records.jsonl` are derived from the fixture,
  not collected from each ghz response body. Actual final lengths and sample values are checked.

## Outputs

- `RUN.json`: configuration, operator-supplied revisions, host, ghz version, binary and payload hashes.
- `fixtures/`: exact specs and shared payloads.
- One directory per run: server command/log, warmup and measured ghz reports, setup and verification logs.
- `records.jsonl`: validated per-run RPS, entries/sec, p50/p95/p99, CPU, and expected mutation counts.
- `SUMMARY.md`: median metrics, RPS ranges, and candidate/control RPS changes.

Server CPU/request covers the ghz process invocation window, excluding setup/verification.
The recorded ghz CPU time helps identify load-generator saturation. CPU and wall-time
windows include client startup and are not identical to ghz's internal timing window.
Failures abort the sweep and preserve logs; a final summary is emitted only after success.

## CI

Add the `set-benchmark-experiments` label to a PR to opt into
`.github/workflows/set-benchmark-experiments.yml`. It compares the PR base SHA against the
PR head built with the candidate feature. The Criterion opt-in label is separate.
The workflow also supports manual dispatch once available on the default branch, using
`control_ref` (default main) and the selected candidate branch. It runs a smaller sweep
and uploads results. Shared-runner measurements
are exploratory, not a regression gate. No workflow or benchmark was launched while
creating this setup.
