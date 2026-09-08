# Predicate ingestion experiments

This target compares the existing-index candidate against the production
`PredicateIndices::add` implementation. The control was main at `ab4ffc70` when the
experiments were created. It calls production directly rather than copying the control
into a benchmark approximation.

| Target | Candidate change | Scenarios |
|---|---|---|
| `predicate_existing_index_ingestion` | Look up an existing predicate index before constructing a replacement candidate; preserve the existing miss/race fallback | Existing, missing, and partially existing indexes; 100/1,000 entries; one/four indexed fields; low/high value cardinality; sequential/parallel execution |

The candidate retains current metadata grouping. With only `bench-experiments` enabled,
it does not change request routing. The candidate implementation and its narrow fixture facade are compiled
only with `bench-experiments`, in `src/engine/predicate/experiments.rs`.

## Running

From `ahnlich/`:

```sh
# Keep Criterion output at the location expected by the PR workflow, even when
# automatic cargo-metadata discovery is unavailable in an offline environment.
export CRITERION_HOME="$(pwd)/target/criterion"

RAYON_NUM_THREADS=4 cargo bench -p db --features bench-experiments \
  --bench predicate_existing_index_ingestion -- --save-baseline predicate-ingestion

critcmp --target-dir target predicate-ingestion \
  --filter '(^|/)(control|candidate)/' \
  --group '^(.*)(?:control|candidate)/(.*)$'
```

Use `-- --test` instead for a one-iteration smoke run. Each target verifies all scenario
memberships against an independently computed expected result before measuring, including
a repeated insertion check for duplicate IDs.

Benchmark IDs use `<target>/control/<scenario>` and `<target>/candidate/<scenario>` to
match `.github/workflows/benchmark-experiments.yml`. New targets are compared within the
PR run; targets already present on the PR base are additionally run there by the workflow.
Opt in with the `benchmark-experiments` PR label. The shared fixture helper lives outside
`benches/experiments/` so the workflow does not mistake it for a benchmark target.

## Measurement boundaries

- Both sides call the complete ingestion function, not just the changed loop.
- Each measured invocation receives a fresh fixture and disjoint incoming IDs. The benchmark
  does not become a duplicate-insertion workload as Criterion repeats it.
- Fixture creation, input cloning, result inspection, and final fixture teardown are outside
  timing through `iter_batched_ref` with `BatchSize::PerIteration`. Temporary allocations and
  destruction performed inside the ingestion function remain part of the measured work.
- Both sides use identical input metadata and the same parallelism configuration. Parallel
  cases explicitly lower the batch threshold to 1,000 and assert that the parallel branch is
  selected; sequential cases disable it. This isolates branch behavior rather than claiming
  to reproduce the production default policy.
- Rayon uses `RAYON_NUM_THREADS` when supplied, or its default otherwise. Record the thread
  count and host when comparing runs. These are single-request microbenchmarks with
  `active_requests = 1`, not concurrent-request throughput tests.
- Missing and mixed-index fixtures exercise creation but do not prove concurrent
  drop/recreation or competing-creator correctness. Those need dedicated concurrency tests
  before promoting a candidate into production.

Run long enough to exceed noise on an idle host. Treat these experiments as mechanism
validation; use ghz for end-to-end throughput and mixed-load validation before choosing
whether to ship the change. The separate `bench-existing-predicate-index` feature enables
candidate 1 for ghz; see [Set A/B instructions](../../../../benchmarks/SET_BENCHMARKS.md).
