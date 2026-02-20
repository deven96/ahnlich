# Ahnlich replication RFC (implementation status)

This document records what has been implemented on the current replication branch, what behavior exists today, and what remains open.

## Scope and goals

- Add horizontal scalability via Raft replication.
- Keep DB and AI as separate Raft clusters (separate node ids/addresses/membership).
- Keep default mode as non-clustered (`cluster_enabled = false`).
- Keep existing `utils::Persistence` independent from cluster mode (if configured, it still runs).

## Implemented architecture

### 1) Shared replication crate

A new crate was added at `ahnlich/replication/` to host shared Raft plumbing used by DB and AI:

- `src/config.rs`
  - `RaftStorageEngine` enum (`memory`, `rocksdb`) used by CLI/runtime.
  - `RaftConfig` shape for node/address/storage/snapshot/join settings.
- `src/types.rs`
  - shared `ClientWriteRequest`/`ClientWriteResponse`.
  - command/response types for DB and AI.
- `src/network.rs`
  - tonic-based Openraft network transport.
  - `GrpcRaftNetworkFactory` and `GrpcRaftService`.
- `src/storage/mod.rs`
  - shared storage adaptor layer, mem + rocksdb log/state-machine support.
- `src/admin.rs`
  - `ClusterAdminService` implementation (init/add learner/change membership/remove/metrics/leader/snapshot).

### 2) New protobuf services

New proto definitions were added:

- `protos/services/raft_internal.proto`
- `protos/services/cluster_admin.proto`

And types were wired in:

- `ahnlich/types/src/services/raft_internal.rs`
- `ahnlich/types/src/services/cluster_admin.rs`
- `ahnlich/types/src/services/mod.rs`

### 3) Server integration (DB + AI)

DB and AI now support cluster mode initialization and runtime wiring:

- Create Raft node and storage in cluster mode.
- Start 3 service surfaces:
  - existing public service (`DbService` / `AiService`)
  - `RaftInternalService`
  - `ClusterAdminService`
- Support pending join flow via admin endpoint.

### 4) Admin CLI

A new cluster admin path was added to the CLI:

- `ahnlich/cli/src/cluster.rs`
- `ahnlich/cli/src/config/cli.rs`

Supported admin commands:

- `init`
- `join`
- `add-learner`
- `change-membership`
- `remove`
- `metrics`
- `leader`
- `snapshot`

## Current API/config behavior

### DB/AI server flags

Both DB and AI CLIs now include cluster flags:

- `--cluster-enabled`
- `--raft-node-id`
- `--raft-addr`
- `--admin-addr`
- `--raft-storage` (`memory` or `rocksdb`)
- `--raft-data-dir` (required when `--raft-storage rocksdb`)
- `--raft-snapshot-logs`
- `--raft-join`

### Storage

Two storage engines for the Raft snapshots were implemented, one in-memory and one with RocksDB, and the storage engine type is shared from `ahnlich_replication::config::RaftStorageEngine`.

## Replication boundary (current behavior)

### DB (Raft-routed mutating operations)

In cluster mode, the following DB mutators are routed through Raft:

- `CreateStore`
- `CreatePredIndex`
- `CreateNonLinearAlgorithmIndex`
- `Set`
- `DropPredIndex`
- `DropNonLinearAlgorithmIndex`
- `DelKey`
- `DelPred`
- `DropStore`

### AI (Raft-routed mutating operations)

In cluster mode, the following AI mutators are routed through Raft:

- `CreateStore`
- `DropStore`
- `PurgeStores`

Other AI mutators are currently not routed through Raft in the same way.

### Read semantics

In cluster mode, read RPCs call `ensure_linearizable()`. Reads are leader-gated and followers can return `UNAVAILABLE`/forwarding-related errors.

## Persistence and snapshot behavior

- Raft snapshot/log durability is provided by selected raft storage (`memory` or `rocksdb`).
- Existing `utils::Persistence` remains independent and still runs when configured.
- Startup flow keeps existing persistence load behavior; Raft reconciliation still applies via log/snapshot replication once cluster is active.

## Testing implemented

### Unit tests

#### AI state machine (`ahnlich/ai/src/replication/mod.rs`)

- `apply_create_store_writes_store`
- `apply_drop_store_deletes_store`
- `apply_purge_stores_deletes_all`
- `apply_idempotent_replay_returns_cached_response`
- `apply_unsupported_command_returns_error`

#### DB state machine (`ahnlich/db/src/replication/mod.rs`)

- `apply_create_store_writes_store`
- `apply_drop_store_deletes_store`
- `apply_set_writes_entries`
- `apply_del_key_deletes_expected_count`
- `apply_del_pred_deletes_expected_count`
- `apply_create_pred_index_returns_created_count`
- `apply_drop_pred_index_returns_deleted_count`
- `apply_create_non_linear_algorithm_index_returns_created_count`
- `apply_drop_non_linear_algorithm_index_returns_deleted_count`
- `apply_idempotent_replay_returns_cached_response`
- `apply_malformed_payload_returns_error`

### Integration tests

#### AI integration (`ahnlich/ai/src/tests/aiproxy_test.rs`)

- single-node clustered create/drop/purge path.
- 3-node single failover test.
- 3-node double failover test:
  - quorum-loss expectations,
  - optional stronger step restoring quorum and re-verifying committed state.

#### DB integration (`ahnlich/db/src/tests/server_tests.rs`)

- single-node clustered create/set/drop path.
- 3-node single failover test.
- 3-node double failover test:
  - quorum-loss expectations,
  - optional stronger step restoring quorum and re-verifying committed state.

## Validation status

All tests are passing.
Note: running full AI tests in default parallel mode in this environment intermittently hit `SIGKILL`; single-thread execution was stable.

## Open items

- Deduplicate service handlers (ahnlich/ai/src/server/handler.rs & ahnlich/db/src/server/handler.rs) and raft handlers (ahnlich/ai/src/replication/mod.rs & ahnlich/db/src/replication/mod.rs) logic. Currently we're copying the logic from the service handler into the raft handler, which can introduce drift/bugs.
- Decide and codify final behavior for AI mutators that are not fully Raft-routed yet (explicit fail-fast vs supported path).
- Add/lock explicit boundary tests for that final behavior.
- Keep failover suites under repeated runs in CI to monitor flakiness.
