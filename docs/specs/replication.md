# Ahnlich Replication Spec

Raft-based replication for horizontal scalability of ahnlich-db and ahnlich-ai.

---

## Table of Contents

- [Overview and Goals](#overview-and-goals)
- [Background](#background)
  - [Why Replication](#why-replication)
  - [Why Raft](#why-raft)
  - [Why openraft](#why-openraft)
- [Architecture](#architecture)
  - [Current Architecture (Standalone)](#current-architecture-standalone)
  - [Clustered Architecture](#clustered-architecture)
  - [Independent DB and AI Clusters](#independent-db-and-ai-clusters)
  - [Service Surfaces](#service-surfaces)
- [Replication Boundary](#replication-boundary)
  - [DB: Raft-Routed Operations](#db-raft-routed-operations)
  - [AI: Raft-Routed Operations](#ai-raft-routed-operations)
  - [AI Delegation to DB](#ai-delegation-to-db)
  - [Read Path](#read-path)
  - [Write Path (Leader Forwarding)](#write-path-leader-forwarding)
  - [Client SDK Topology Discovery](#client-sdk-topology-discovery)
  - [Pipeline Operations](#pipeline-operations)
- [State Machine Design](#state-machine-design)
  - [Shared Mutation Functions](#shared-mutation-functions)
  - [DB State Machine](#db-state-machine)
  - [AI State Machine](#ai-state-machine)
  - [Snapshot and Recovery](#snapshot-and-recovery)
- [Storage](#storage)
  - [Raft Log and State Machine Storage](#raft-log-and-state-machine-storage)
  - [Persistence Interaction](#persistence-interaction)
- [Cluster Lifecycle](#cluster-lifecycle)
  - [Cluster Formation](#cluster-formation)
  - [Joining an Existing Cluster](#joining-an-existing-cluster)
  - [Graceful Shutdown](#graceful-shutdown)
  - [Failure and Recovery](#failure-and-recovery)
- [Configuration](#configuration)
  - [Server CLI Flags](#server-cli-flags)
  - [CLI Cluster Commands](#cli-cluster-commands)
- [Implementation Milestones](#implementation-milestones)
- [Testing Strategy](#testing-strategy)
- [Open Questions and Future Work](#open-questions-and-future-work)
- [References](#references)

---

## Overview and Goals

Ahnlich currently operates as a single-instance system. Each ahnlich-db or ahnlich-ai process runs independently with no coordination. This limits deployment to vertical scaling only and introduces a single point of failure.

This spec describes adding **Raft-based replication** to both ahnlich-db and ahnlich-ai, enabling:

1. **Fault tolerance**: A cluster requires a majority of nodes to be alive (quorum). Quorum size = ⌊N/2⌋ + 1, and the number of failures the cluster can tolerate is N minus the quorum size. For example: a 3-node cluster has a quorum of 2, so it tolerates 1 failure; a 5-node cluster has a quorum of 3, so it tolerates 2.
2. **Data durability**: Committed writes survive individual node failures through log and snapshot replication.
3. **Horizontal read scaling**: Most read operations are served locally by any node (leader or follower), distributing read load across the cluster. Only consistency-sensitive reads (e.g., `ListStores`) are forwarded to the leader.

**Non-goals for this iteration:**

- Sharding / data partitioning across nodes. All nodes hold a full copy of all data.
- Multi-region deployment with geo-aware routing.
- A built-in `SmartClient` that automatically routes requests across cluster nodes (see [Future Work](#smart-client-future)). Clients are cluster-aware via `ClusterInfo` and can implement their own routing.

---

## Background

### Why Replication

Ahnlich is an in-memory vector search engine. All data resides in memory, with optional periodic JSON snapshots to disk. A single node failure means data loss (until the snapshot is reloaded) and service downtime.

Replication solves both problems: data is replicated across multiple nodes, so surviving nodes still hold the complete dataset and can continue serving requests immediately.

We chose **full replication** (every node holds all data) over **sharding** (data partitioned across nodes) because:

- Ahnlich's primary use case is real-time similarity search, where query latency matters more than dataset size.
- Full replication provides simpler operations: any node can serve any query.
- Sharding adds complexity (routing, rebalancing, cross-shard queries) that is not yet needed.
- Sharding can be layered on top of replication later if dataset sizes exceed single-node memory.

### Why Raft

Raft is a consensus algorithm designed to be understandable and practically implementable. It provides:

- **Strong consistency**: All committed writes are visible across the cluster in the same order.
- **Leader-based protocol**: One node (the leader) coordinates all writes, simplifying conflict resolution.
- **Well-defined failure model**: Clear rules for leader election, log replication, and safety.

Alternatives considered:

| Algorithm      | Tradeoff                                                             |
|----------------|----------------------------------------------------------------------|
| Paxos          | Equivalent guarantees but harder to implement correctly              |
| CRDTs          | Eventual consistency only; not suitable for ordered index operations |
| Primary-backup | Simpler but no formalized correctness guarantees                     |

### Why openraft

[openraft](https://docs.rs/openraft/latest/openraft/) is an async-native Rust implementation of Raft (pre-1.0, API still evolving). It provides:

- Async runtime compatibility (tokio).
- Pluggable storage: bring your own log store, state machine, and network transport.
- Snapshot support for state machine compaction.

We implement three pluggable components:

1. **Network transport**: gRPC via tonic (fits our existing stack).
2. **Log store**: RocksDB (default) or in-memory (primarily for testing, but should probably completely leave it out).
3. **State machine**: Application-specific logic that applies Raft entries to `StoreHandler` / `AIStoreHandler`.

---

## Architecture

### Current Architecture (Standalone)

```
Client ──gRPC──▶ Server (DbService / AiService)
                    │
                    ▼
              StoreHandler/AIStoreHandler (in-memory stores)
                    │
                    ▼
              Persistence (periodic JSON snapshots)
```

A single server process exposes one gRPC service. The `StoreHandler`/`AIStoreHandler` manages all stores in memory. An optional `Persistence` task periodically serializes the stores to a JSON file on disk.

### Clustered Architecture

```
                           ┌──────────────────────────┐
                           │        Leader            │
Client ──gRPC──▶ Any Node  │  Public gRPC (--port)    │
                    │      │  Cluster gRPC (--cluster-│
                    │      │    addr): Raft RPCs +    │
                    │      │    cluster admin         │
                    │      └──────────────────────────┘
                    │                    │
                    ▼                    │ Raft Log Replication
             ┌──────────----┐            │
             │ Is this      │            ▼
             │ the          │    ┌──────────────────────────---------------------------------┐
             │ leader?      │    │      Follower(s)                                          │
             │              │    │  Public gRPC (--port)                                     │
             │ Yes → Execute│    |  Cluster gRPC (--cluster-addr): Raft RPCs + cluster admin │
             │ No  → Forward│    |                                                           |
             │ to leader    │    │                                                           |
             └─────────-----┘    └──────────────────────────---------------------------------┘
```

In cluster mode, each server process exposes **two** gRPC service surfaces on separate ports:

1. **Public service** (`--port`): The existing `DbService` / `AiService` for client traffic.
2. **Cluster service** (`--cluster-addr`): Node-to-node Raft RPCs (AppendEntries, Vote, InstallSnapshot) and cluster administration (membership changes, metrics). This separates internal cluster traffic from client traffic.

Write requests received by any node are forwarded to the leader if necessary. The leader submits them through Raft consensus. Once committed, the log entry is applied to all nodes' state machines.

### Independent DB and AI Clusters

DB and AI run as **separate, independent Raft clusters**:

```
┌─────────────────────────────┐    ┌─────────────────────────────┐
│       DB Raft Cluster       │    │       AI Raft Cluster       │
│                             │    │                             │
│  DB-Node-1 (leader)         │    │  AI-Node-1 (leader)         │
│  DB-Node-2 (follower)       │    │  AI-Node-2 (follower)       │
│  DB-Node-3 (follower)       │    │  AI-Node-3 (follower)       │
│                             │    │                             │
│  State: StoreHandler        │    │  State: AIStoreHandler      │
│  (vectors, indices, data)   │    │  (store metadata only)      │
└─────────────────────────────┘    └─────────────────────────────┘
         ▲                                    │
         │              db_client             │
         └────────────────────────────────────┘
```

**Rationale:**

- DB and AI have different data and different failure domains. A DB node failure should not affect AI availability, and vice versa.
- DB and AI can scale independently (e.g., 3 DB nodes, 5 AI nodes).
- AI is architecturally a proxy: most AI mutations delegate to DB via `db_client`. The AI Raft cluster only replicates AI-local state (store metadata: which model is associated with which store, `store_original` flags). The actual vector data is replicated by the DB Raft cluster.
- Each cluster has its own membership, its own leader election, and its own Raft log.

### Service Surfaces

Each server process (DB or AI) exposes two gRPC services when clustering is enabled:

| Service                       | Port Flag        | Purpose                                                                                                        |
|-------------------------------|------------------|----------------------------------------------------------------------------------------------------------------|
| Public (DbService/AiService)  | `--port`         | Client-facing CRUD and query operations                                                                        |
| Cluster                       | `--cluster-addr` | Node-to-node Raft RPCs (AppendEntries, Vote, InstallSnapshot) and cluster administration (membership, metrics) |

The Cluster service is internal to the cluster; client SDKs do not connect to it. However, the **public** service will now expose a read-only `ClusterInfo` query (see [Client SDK Topology Discovery](#client-sdk-topology-discovery)) that returns cluster topology, enabling clients to discover nodes and implement their own routing without exposing any admin or Raft operations.

---

## Replication Boundary

### DB: Raft-Routed Operations

All DB mutations that modify **DB store state** (the `StoreHandler`) are routed through Raft in cluster mode (i.e., when `--cluster-addr` is provided):

| Operation                        | Proto RPC                               | Effect                                                                  |
|----------------------------------|-----------------------------------------|-------------------------------------------------------------------------|
| `CreateStore`                    | `db.query.CreateStore`                  | Creates a new store with specified dimensions, predicates, and indices  |
| `CreatePredIndex`                | `db.query.CreatePredIndex`              | Adds predicate indices to an existing store                             |
| `CreateNonLinearAlgorithmIndex`  | `db.query.CreateNonLinearAlgorithmIndex`| Adds non-linear algorithm indices (HNSW, KDTree) to a store             |
| `Set`                            | `db.query.Set`                          | Upserts vector key-value pairs into a store                             |
| `DelKey`                         | `db.query.DelKey`                       | Deletes entries by vector keys                                          |
| `DelPred`                        | `db.query.DelPred`                      | Deletes entries matching a predicate condition                          |
| `DropPredIndex`                  | `db.query.DropPredIndex`                | Removes predicate indices from a store                                  |
| `DropNonLinearAlgorithmIndex`    | `db.query.DropNonLinearAlgorithmIndex`  | Removes non-linear algorithm indices from a store                       |
| `DropStore`                      | `db.query.DropStore`                    | Deletes an entire store                                                 |

**Not routed through Raft** (read-only operations):

- **Served locally by any node** (follower or leader): `GetKey`, `GetPred`, `GetSimN`, `ListClients`, `InfoServer`, `ClusterInfo`, `Ping` — these read from local state and tolerate slightly stale data.
- **Forwarded to leader** (`ensure_linearizable()`): `ListStores` — consistency-sensitive because users expect to see a store immediately after creating it.

### AI: Raft-Routed Operations

Only AI operations that modify **AI-local state** (the `AIStoreHandler`) are routed through the AI Raft cluster:

| Operation     | Proto RPC              | Effect                                                                                                                    |
|---------------|------------------------|---------------------------------------------------------------------------------------------------------------------------|
| `CreateStore` | `ai.query.CreateStore` | Registers a store in AIStoreHandler with model metadata, then delegates to DB via `db_client` to create the backing store |
| `DropStore`   | `ai.query.DropStore`   | Removes store metadata from AIStoreHandler only (does **not** currently appear to drop the backing DB store)              |
| `PurgeStores` | `ai.query.PurgeStores` | Removes all stores from AIStoreHandler only (does **not** currently appear to drop the backing DB stores)                 |

> **Note**: `DropStore` and `PurgeStores` currently leave the underlying DB stores orphaned. This will need to be addressed.

### AI Delegation to DB

The remaining AI mutations operate entirely through the DB client and do **not** go through the AI Raft cluster:

| AI Operation                     | Delegation                                                                     |
|----------------------------------|--------------------------------------------------------------------------------|
| `CreatePredIndex`                | Forwards to DB's `CreatePredIndex` via `db_client`                             |
| `CreateNonLinearAlgorithmIndex`  | Forwards to DB's `CreateNonLinearAlgorithmIndex` via `db_client`               |
| `Set`                            | Converts inputs to embeddings via model, then calls DB's `Set` via `db_client` |
| `DelKey`                         | Forwards to DB's `DelKey` via `db_client`                                      |
| `DelPred`                        | Forwards to DB's `DelPred` via `db_client`                                     |
| `DropPredIndex`                  | Forwards to DB's `DropPredIndex` via `db_client`                               |
| `DropNonLinearAlgorithmIndex`    | Forwards to DB's `DropNonLinearAlgorithmIndex` via `db_client`                 |
| `GetKey`, `GetPred`, `GetSimN`   | Forwards to corresponding DB read operations via `db_client`                   |

These operations modify DB state, not AI state. The DB Raft cluster handles their replication. The AI node is a stateless proxy for these operations.

**Important**: The AI `db_client` can connect to any node in the DB cluster. Leader forwarding (Milestone 3) ensures that writes arriving at a DB follower are transparently forwarded to the DB leader, so the AI node does not need to track which DB node is currently the leader.

### Read Path

Most read operations are served **locally by any node** — leader or follower — from the node's own in-memory state. This provides horizontal read scaling: read load is distributed across all nodes in the cluster.

```
Client → GetSimN → Any Node
                      │
                      ├─ Execute query on local `StoreHandler`/`AIStoreHandler`
                      │
                      └─ Return result
```

Follower state may lag the leader by a small number of log entries (typically milliseconds). For Ahnlich's primary use case (similarity search), this is acceptable — missing a just-inserted vector by a few milliseconds does not meaningfully affect results.

**Consistency-sensitive reads** (`ListStores`) are forwarded to the leader, where `ensure_linearizable()` confirms leader status before executing. This ensures that a `ListStores` call immediately after `CreateStore` always returns the new store.

```
Client → ListStores → Any Node
                         │
                         ├─ Is this the leader?
                         │    No  → forward to leader
                         │    Yes ↓
                         │
                         ├─ ensure_linearizable()
                         │
                         ├─ Execute ListStores on `StoreHandler`/`AIStoreHandler`
                         │
                         └─ Return result
```

### Write Path (Leader Forwarding)

When a write request arrives at a follower node:

```
Client → CreateStore → Follower
                          │
                          ├─ raft.client_write(command)
                          │    → Returns ForwardToLeader error
                          │
                          ├─ Look up leader's service address from NodeRegistry
                          │
                          ├─ Forward original gRPC request to leader
                          │
                          ├─ Leader processes through Raft
                          │    → Log entry committed and applied on all nodes
                          │
                          └─ Return leader's response to client
```

This transparent forwarding means clients can connect to any node in the cluster without needing to discover the leader. The trade-off is an extra network hop for writes that hit followers.

### Client SDK Topology Discovery

The public service (on `--port`) exposes a read-only `ClusterInfo` query that returns the current cluster topology and per-node health. This is the single source of truth for cluster state visible to clients and monitoring tools. No admin or Raft operations are exposed.

**Query and response types:**

```rust
// New query variant added to ServerQuery / AIServerQuery
ClusterInfo

// Response types (in ahnlich_types)
pub struct ClusterNode {
    pub node_id: u64,
    pub addr: String,       // public gRPC address (--port)
    pub role: NodeRole,
    pub term: u64,          // current Raft term
    pub commit_index: u64,  // last committed log index
}

pub enum NodeRole {
    Leader,
    Follower,
    Learner,
}

// Response variant
ClusterInfo(Vec<ClusterNode>)
```

**Standalone (non-clustered) nodes** return a single entry with `role: Leader`, so client code works uniformly regardless of deployment mode.

Clients can use the `ClusterInfo` response to connect to multiple nodes and implement their own routing strategy — for example, sending writes to the leader and distributing reads across followers. The existing `DbClient`/`AiClient` is a thin gRPC wrapper with no single-node limitation; users can create one instance per node. The `SmartClient` described in [Future Work](#smart-client-future) is a convenience wrapper we may provide in the SDK, but users are free to build their own routing on top of `ClusterInfo` from day one.

### Pipeline Operations

The `Pipeline` RPC accepts a sequence of operations and executes them sequentially. In cluster mode:

- Each mutation within the pipeline is individually routed through Raft (forwarded to leader if on a follower).
- Most read operations within the pipeline are served locally from the node's own state.
- Consistency-sensitive reads (`ListStores`) within the pipeline are forwarded to the leader.
- Individual operation failures do not fail the pipeline; errors are returned per-operation in the response sequence.

---

## State Machine Design

### Shared Mutation Functions

A critical design requirement is that the business logic for each mutation exists in **exactly one place**. Both the gRPC handler (standalone mode) and the Raft state machine (cluster mode) must call the same function.

For each mutation, we define a shared function:

```rust
// db/src/engine/mutations.rs (conceptual)

/// Executes a CreateStore mutation against the StoreHandler.
/// Called by both the gRPC handler and the Raft state machine.
pub fn execute_create_store(
    store_handler: &StoreHandler,
    params: &query::CreateStore,  // decoded proto
) -> Result<(), ServerError> {
    let dimensions = NonZeroUsize::new(params.dimension as usize)
        .ok_or(ServerError::InvalidDimension)?;
    // ... validation, type conversion ...
    store_handler.create_store(store_name, dimensions, ...)?;
    Ok(())
}
```

**gRPC handler path** (both standalone and cluster mode):

```rust
async fn create_store(&self, request: Request<CreateStore>) -> Result<Response<Unit>, Status> {
    let params = request.into_inner();

    if let Some(raft) = &self.raft {
        // Cluster mode: route through Raft
        let resp = raft.client_write(ClientWriteRequest {
            command: DbCommand::CreateStore(params.encode_to_vec()),
            ...
        }).await.map_err(map_raft_error)?;
        return Ok(Response::new(decode_response(resp)?));
    }

    // Standalone mode: execute directly
    execute_create_store(&self.store_handler, &params)?;
    Ok(Response::new(Unit {}))
}
```

**Raft state machine `apply()` path:**

```rust
fn apply_entry(&self, command: DbCommand) -> Result<DbResponse, ServerError> {
    match command {
        DbCommand::CreateStore(payload) => {
            let params = query::CreateStore::decode(&payload[..])?;
            execute_create_store(&self.store_handler, &params)?;
            Ok(DbResponse::unit())
        }
        // ... other commands
    }
}
```

This ensures that any bug fix or behavior change in a mutation only needs to happen once.

### DB State Machine

The DB state machine manages the full `StoreHandler` state. It applies 9 mutation commands:

```
DbCommand enum:
├── CreateStore(Vec<u8>)
├── CreatePredIndex(Vec<u8>)
├── CreateNonLinearAlgorithmIndex(Vec<u8>)
├── Set(Vec<u8>)
├── DelKey(Vec<u8>)
├── DelPred(Vec<u8>)
├── DropPredIndex(Vec<u8>)
├── DropNonLinearAlgorithmIndex(Vec<u8>)
└── DropStore(Vec<u8>)
```

Each variant carries the serialized protobuf payload (`Vec<u8>`), decoded and executed via the shared mutation functions.

### AI State Machine

The AI state machine manages only the `AIStoreHandler` metadata. It applies 3 commands:

```
AiCommand enum:
├── CreateStore(Vec<u8>)
├── DropStore(Vec<u8>)
└── PurgeStores(Vec<u8>)
```

### Snapshot and Recovery

Snapshots compact the Raft log by capturing the full in-memory state at a point in time, allowing older log entries to be purged. Two snapshot triggers work side by side — whichever fires first:

**1. Count-based** (`--cluster-snapshot-logs`): Trigger a snapshot after N log entries since the last snapshot. This bounds log size during write bursts. This is openraft's built-in `snapshot_policy::LogsThenSnapshot(n)`.

**2. Time-based with dirty check** (`--cluster-snapshot-interval`): A background task wakes every T milliseconds and checks whether any mutations have been applied since the last snapshot. If yes, trigger a snapshot; if no, skip. This catches slow trickles of writes that never hit the count threshold.

```rust
// Count-based: configured via openraft's snapshot policy (built-in)
let config = openraft::Config {
    snapshot_policy: SnapshotPolicy::LogsSinceLast(1000), // snapshot every 1000 entries
    ..Default::default()
};
```

```rust
// Time-based: background task, same pattern as utils::Persistence
struct RaftSnapshotTimer {
    raft: Arc<Raft<TypeConfig>>,
    interval_ms: u64,
}

impl Task for RaftSnapshotTimer {
    async fn run(&self) -> TaskState {
        tokio::time::sleep(Duration::from_millis(self.interval_ms)).await;

        let metrics = self.raft.metrics().borrow().clone();

        // Compare last snapshot index vs last applied index
        if let (Some(snapshot), Some(last_applied)) = (metrics.snapshot, metrics.last_applied) {
            if last_applied.index > snapshot.index {
                // Mutations since last snapshot — trigger one
                let _ = self.raft.trigger().snapshot().await;
            }
        }

        TaskState::Continue
    }
}
```

Together, these ensure: high write volume → count trigger fires promptly; low write volume → interval trigger catches stragglers; idle cluster → interval fires but dirty check skips (no unnecessary snapshots).

> **Open question for the team**: Do we want both triggers from the start, or ship with count-based only and add time-based later? Both are straightforward to implement, but having two snapshot mechanisms adds configuration surface. The time-based trigger follows the same `Task` pattern already used by `utils::Persistence` and the size calculation background task.

Our state machine snapshot captures the full in-memory state:

**DB snapshot contents:**
- All stores (name → `Store` struct), where each store contains: `dimension` (required embedding size), `id_to_value` (mapping of `StoreKeyId` → `(EmbeddingKey, Arc<StoreValue>)`), `predicate_indices`, `non_linear_indices`

**AI snapshot contents:**
- All AI store metadata (name → `AIStore` struct), where each AI store contains: `name`, `query_model`, `index_model`, `store_original` flag

**Snapshot creation:**

1. openraft calls `build_snapshot()`
2. State machine serializes `StoreHandler`/`AIStoreHandler` state via shared serialization functions (see [Persistence Interaction](#persistence-interaction))
3. Snapshot is persisted to the storage backend (see [Raft Log and State Machine Storage](#raft-log-and-state-machine-storage))
4. Log entries before the snapshot can be purged

**Recovery** (on node restart or new node joining): the storage backend restores the latest snapshot into memory, then replays any subsequent log entries. See [Raft Log and State Machine Storage](#raft-log-and-state-machine-storage) for the full recovery flow.

---

## Storage

### Raft Log and State Machine Storage

The state machine (`StoreHandler` / `AIStoreHandler`) is always **in-memory** at runtime — all queries and mutations execute against in-memory data. The storage backend is a separate **durability layer** underneath that persists Raft state to survive restarts. It is not read during normal operation, only during startup/recovery.

**What the storage backend holds:**

- **Raft log entries**: Each mutation (CreateStore, Set, DelKey, etc.) stored as a serialized command. Entries are persisted to the storage backend **before** they are considered committed — durability comes from the log, not from the state machine. A mutation is committed once a majority of nodes have persisted it to their own log. Only after commitment is the entry `apply()`-ed to the in-memory state machine.
- **Snapshots**: Periodic serialized captures of the entire in-memory state (see [Snapshot and Recovery](#snapshot-and-recovery) for what each snapshot contains). Snapshots are an optimization for log compaction — without them, the entire log would need to be kept and replayed from entry #1 on every restart. Once a snapshot exists at log index N, entries before N can be purged.
- **Raft metadata**: Current term, vote state, committed index — the bookkeeping openraft needs to resume correctly across restarts.

**The state machine is a derived view**: The in-memory `StoreHandler`/`AIStoreHandler` can always be reconstructed by replaying the log (or loading a snapshot + replaying remaining entries). It is never written back to the storage backend after each `apply()` — the log entries already provide durability. The backend is written to when (1) new log entries are appended (before commitment), (2) a snapshot is triggered for log compaction, or (3) Raft metadata changes (elections, term changes).

**Recovery flow** (on node restart):

```
1. Load latest snapshot from storage → deserialize into `StoreHandler`/`AIStoreHandler`
2. Replay any log entries after the snapshot via apply()
3. Node is now current and can rejoin the cluster
```

**Storage backends:**

| Backend            | Flag Value                  | Durability                      | Use Case         |
|--------------------|-----------------------------|---------------------------------|------------------|
| RocksDB (default)  | `--cluster-storage rocksdb` | Full (persisted to disk)        | All deployments  |
| Memory             | `--cluster-storage memory`  | None (data lost on restart)     | Testing only     |

When using RocksDB, `--cluster-data-dir` specifies the directory for the database files.

Both backends implement the same trait interface:

- **Log store**: Append/read/truncate Raft log entries
- **State machine store**: Apply entries, build/install snapshots

> **Open question for the team**: Do we want to keep the in-memory storage backend at all? It has no durability — a node restart loses all Raft state, which defeats the purpose of replication. Its only value is convenience in tests (no disk cleanup). If we keep it, should it be clearly gated as a testing-only option and excluded from any production documentation?

### Persistence Interaction

Ahnlich has an existing standalone persistence mechanism (`utils::Persistence`) that periodically serializes the in-memory stores to a JSON file on disk via `serde_json`. In cluster mode, `utils::Persistence` is **disabled** — Raft snapshots and log replication handle all persistence. In standalone mode, persistence continues to work as before.

**Shared serialization layer**: Both Raft snapshots and standalone persistence use the same serialization/deserialization functions:

```rust
// utils/src/snapshot.rs (conceptual)

/// Serialize a store snapshot to bytes.
pub fn serialize_snapshot<T: Serialize>(state: &T) -> Result<Vec<u8>, SnapshotError> {
    serde_json::to_vec(state).map_err(SnapshotError::from)
}

/// Deserialize a store snapshot from bytes.
pub fn deserialize_snapshot<T: DeserializeOwned>(data: &[u8]) -> Result<T, SnapshotError> {
    serde_json::from_slice(data).map_err(SnapshotError::from)
}
```

This means the serialization format is decided in exactly one place. If the format changes in the future, it changes for both paths.

**How cluster mode disables persistence**: The `Persistence` task is only spawned when `--enable-persistence` is set and `--cluster-addr` is absent. When cluster flags are present, the persistence task is skipped and a log message indicates that Raft handles persistence:

```rust
// In server startup (utils/src/server.rs, conceptual)
if let Some(persist_location) = self.config().persist_location {
    if self.config().cluster_addr.is_some() {
        log::info!("Cluster mode enabled — skipping standalone persistence (Raft handles persistence)");
    } else {
        let persistence_task = Persistence::task(...);
        task_manager.spawn_task_loop(persistence_task).await;
    }
}
```

- **Standalone mode** (no `--cluster-addr`): `utils::Persistence` operates as before, refactored to call `serialize_snapshot()` instead of `serde_json::to_writer` directly.
- **Cluster mode** (`--cluster-addr` provided): Raft log replication and Raft snapshots are the sole persistence mechanism. `utils::Persistence` is not spawned.

---

## Cluster Lifecycle

### Cluster Formation

To form a new cluster:

1. Start the first node with `--cluster-bootstrap`. This creates a single-node cluster and elects itself leader.
2. Start additional nodes with `--cluster-join <addr>`, pointing at any existing cluster node's `--cluster-addr`.

Node IDs are automatically derived by hashing the `--cluster-addr` value, so they are guaranteed unique as long as cluster addresses are unique (which they must be). The user never needs to specify or track node IDs.

```bash
# Bootstrap the first node (creates a new cluster)
ahnlich-db run --port 1369 --cluster-addr 127.0.0.1:9001 --cluster-bootstrap

# Join additional nodes
ahnlich-db run --port 1370 --cluster-addr 127.0.0.1:9002 --cluster-join 127.0.0.1:9001

ahnlich-db run --port 1371 --cluster-addr 127.0.0.1:9003 --cluster-join 127.0.0.1:9001
```

### Joining an Existing Cluster

When a node starts with `--cluster-join`:

1. The node derives its own node ID from its `--cluster-addr`.
2. It contacts the specified cluster address.
3. The cluster leader adds the new node as a **learner** (non-voting).
4. The leader sends a snapshot of the current state to the new node.
5. The new node applies the snapshot and begins receiving log entries.
6. Once caught up, the node is automatically promoted to a voter.

```bash
# Add a 4th node to an existing cluster
ahnlich-db run --port 1372 --cluster-addr 127.0.0.1:9004 --cluster-join 127.0.0.1:9001
```

### Graceful Shutdown

Raft cleanup is implemented as a `Task` with a `cleanup()` method, plugging into the existing `TaskManager` pattern. This means the same shutdown sequence runs regardless of trigger — SIGTERM, internal panic (via `install_panic_hook()`), or any other cancellation of the shared `CancellationToken`.

When shutdown is triggered:

1. The `TaskManager` cancels its `CancellationToken`, signalling all tasks.
2. The Raft cleanup task's `cleanup()` method runs:
   a. If the node is a voter, it removes itself from the voter set.
   b. If the node is the leader, it transfers leadership to another node before stepping down.
3. The node completes in-flight requests and shuts down.
4. `TaskManager::wait()` returns once all tasks (including Raft cleanup) have completed.

This prevents unnecessary leader elections and minimizes disruption. Because it uses the `TaskManager`/`Task` pattern already established in the codebase, panics in any thread trigger the same orderly shutdown.

### Failure and Recovery

**Single node failure** (minority): The cluster continues operating normally. The remaining nodes maintain quorum and can serve reads and writes.

**Majority failure** (quorum loss): The cluster becomes read-only (or unavailable, depending on implementation). No new writes can be committed. To recover:

1. Restart failed nodes.
2. Raft will automatically re-replicate logs and restore the cluster.

**All nodes restart**: With `--cluster-storage rocksdb`, nodes recover from their persisted Raft logs and snapshots. With `--cluster-storage memory`, all data is lost.

---

## Configuration

### Server CLI Flags

Flags added to both `ahnlich-db run` and `ahnlich-ai run`:

| Flag                          | Type   | Default   | Description                                                                                                          |
|-------------------------------|--------|-----------|----------------------------------------------------------------------------------------------------------------------|
| `--cluster-addr`              | string | (none)    | Address (`host:port`) for cluster traffic (Raft RPCs and admin). Node ID is derived by hashing this address.         |
| `--cluster-bootstrap`         | bool   | `false`   | Bootstrap a new single-node cluster. This node becomes the initial leader. Mutually exclusive with `--cluster-join`. |
| `--cluster-join`              | string | (none)    | Cluster address of an existing node to join as a learner. Mutually exclusive with `--cluster-bootstrap`.             |
| `--cluster-storage`           | enum   | `rocksdb` | Storage engine: `rocksdb` (recommended) or `memory` (testing only, no durability).                                   |
| `--cluster-data-dir`          | path   | (none)    | Filesystem path for RocksDB files. Required when `--cluster-storage rocksdb`.                                        |
| `--cluster-snapshot-logs`     | u64    | `1000`    | Snapshot after this many log entries. Larger = fewer snapshots; smaller = bounded log size.                          |
| `--cluster-snapshot-interval` | u64    | `300000`  | Milliseconds between snapshot checks. If mutations occurred since last snapshot, triggers one.                       |

### CLI Cluster Commands

Cluster administration is exposed via `ahnlich-cli cluster`:

| Command             | Flags                                                          | Description                                                            |
|---------------------|----------------------------------------------------------------|------------------------------------------------------------------------|
| `add-learner`       | `--cluster-addr`, `--node-cluster-addr`, `--node-service-addr` | Add a learner (non-voting) node to the cluster.                        |
| `change-membership` | `--cluster-addr`, `--node-ids`                                 | Change the set of voting members. Comma-delimited list of node IDs.    |
| `remove`            | `--cluster-addr`, `--node-id`                                  | Remove a node from the cluster.                                        |
| `metrics`           | `--cluster-addr`                                               | Get cluster metrics (current term, commit index, leader, etc.).        |
| `leader`            | `--cluster-addr`                                               | Get the current leader's node info.                                    |
| `snapshot`          | `--cluster-addr`                                               | Trigger an immediate snapshot.                                         |

---

## Implementation Milestones

Each milestone is an independently mergeable PR that produces a working, testable state.

### Milestone 1: Replication Crate Foundation

**Goal**: Create the shared `ahnlich/replication/` crate with all Raft infrastructure that both DB and AI will use. Add the necessary proto definitions.

**What this includes:**

- **`replication/src/config.rs`**: `RaftConfig` struct and `RaftStorageEngine` enum (Memory/RocksDb).
- **`replication/src/types.rs`**: Generic `ClientWriteRequest<C>` / `ClientWriteResponse<R>` wrappers. Concrete `DbCommand` enum (9 variants) and `AiCommand` enum (3 variants: CreateStore, DropStore, PurgeStores only).
- **`replication/src/network.rs`**: gRPC-based Raft network transport using tonic. Implements openraft's `RaftNetworkFactory` and `RaftNetwork` traits.
- **`replication/src/storage/`**: Log store and state machine storage abstraction supporting memory and RocksDB backends.
- **`replication/src/admin.rs`**: `ClusterAdminService` implementation (add_learner, change_membership, remove, metrics, leader, snapshot).
- **`replication/proto/raft_internal.proto`**: Raft-internal RPCs with binary payload wrappers.
- **`replication/proto/cluster_admin.proto`**: Cluster admin operations with `NodeInfo` structure.
- **`replication/build.rs`**: Proto codegen wiring within the replication crate's build system (these are internal protos, not public client protos).
- **`replication/src/cluster_info.rs`**: A function that takes a Raft handle and returns `Vec<ClusterNode>` with each node's address, role, term, and commit index. Wired into DB in Milestone 2 and AI in Milestone 4.
- **Shared serialization functions** (`utils/src/snapshot.rs`): `serialize_snapshot()` and `deserialize_snapshot()` using `serde_json`. These are the single source of truth for the serialization format, used by both Raft snapshots and standalone persistence.

**What this does NOT include**: Any changes to DB or AI server behavior. The crate is a library only.

**Testing**:

- Unit tests for config parsing, type serialization/deserialization, storage operations.
- Unit tests for shared serialization functions: round-trip with representative test structs (the actual `StoreHandler` and `AIStoreHandler` types live in higher-level crates; round-trip tests with those types belong in Milestones 2 and 4).

---

### Milestone 2: DB Raft Integration

**Goal**: Integrate Raft into ahnlich-db. All 9 mutations are routed through Raft in cluster mode. Shared mutation functions eliminate code duplication.

**What this includes:**

- **Shared mutation functions** (`db/src/engine/mutations.rs` or equivalent): One function per DB mutation that takes the decoded proto message and the `StoreHandler`, executes the mutation, and returns the result. Both the gRPC handler and the Raft state machine call these functions.
- **DB state machine** (`db/src/replication/mod.rs`): Implements openraft's `StateMachineStore` trait. Applies log entries by decoding the proto payload and calling the shared mutation function. Snapshot build/install uses the shared serialization functions from Milestone 1.
- **DB server changes** (`db/src/server/handler.rs`): In cluster mode, mutation gRPC handlers encode the request and submit via `raft.client_write()`. Most read handlers serve from local state. `ListStores` calls `ensure_linearizable()` and is forwarded to leader if on a follower (forwarding added in Milestone 3; until then, `ListStores` on followers returns an error).
- **ClusterInfo query**: Wire the shared `ClusterInfo` handler from the replication crate into DB's public service.
- **CLI flags** (`db/src/cli/server.rs`): All cluster flags from the Configuration section.
- **Server startup** (`db/src/server/handler.rs`): Conditionally create Raft node, start cluster service, handle `--cluster-join` / `--cluster-bootstrap` flow.

**Testing**:

- Unit test for shared serialization round-trip with actual `StoreHandler` snapshot type.
- Unit tests for DB state machine apply/snapshot/restore.
- Integration test: single-node cluster performing all CRUD operations.
- Integration test: 3-node cluster with basic replication verification.
- Integration test: call `ClusterInfo` on a 3-node cluster, verify it returns all 3 nodes with correct roles, terms, and commit indices.

---

### Milestone 3: Leader Forwarding

**Goal**: Any node can accept writes. Followers transparently forward to the leader. This is essential before AI integration, since the AI `db_client` needs to connect to any DB node without tracking which is the leader.

**What this includes:**

- In mutation handlers, when `raft.client_write()` returns `ForwardToLeader { leader_id, .. }`, look up the leader's service address from `NodeRegistry` and forward the gRPC request.
- The forwarding client should use a connection pool or cached channel to avoid per-request connection overhead.
- Same forwarding logic for `ListStores` when `ensure_linearizable()` fails on a follower.

**Testing**:

- Integration test: send writes to a known follower node, verify they succeed (forwarded to leader).
- Integration test: send `ListStores` to a follower after creating a store on the leader, verify the new store appears (forwarded to leader).
- Integration test: send `GetSimN` to a follower, verify it succeeds from local state (not forwarded).

---

### Milestone 4: AI Raft Integration

**Goal**: Integrate Raft into ahnlich-ai for the 3 AI-local mutations. Same shared-mutation-function pattern as DB.

**What this includes:**

- **Shared mutation functions** for AI's CreateStore, DropStore, PurgeStores.
- **AI state machine** (`ai/src/replication/mod.rs`): Implements openraft's `StateMachineStore` trait. Applies log entries by decoding the proto payload and calling the shared mutation function. Only 3 AI commands (CreateStore, DropStore, PurgeStores) are replicated — other mutations flow through `db_client` to the DB cluster (leader forwarding from Milestone 3 ensures this works regardless of which DB node the `db_client` connects to). Snapshot build/install uses the shared serialization functions from Milestone 1.
- **AI server changes** (`ai/src/server/handler.rs`): CreateStore, DropStore, PurgeStores routed through Raft when clustered.
- **ClusterInfo query**: Wire the shared `ClusterInfo` handler into AI's public service.
- **CLI flags** (`ai/src/cli/server.rs`): Same cluster flags as DB.
- **Server startup**: Same 2-service-surface pattern as DB.

**Testing**:

- Unit test for shared serialization round-trip with actual `AIStoreHandler` snapshot type.
- Unit tests for AI state machine apply/snapshot/restore.
- Integration test: single-node AI cluster performing all CRUD operations.
- Integration test: 3-node cluster with basic replication verification.
- Integration test: call `ClusterInfo` on a 3-node AI cluster, verify it returns all 3 nodes with correct roles, terms, and commit indices.

---

### Milestone 5: CLI Cluster Commands

**Goal**: Add the `ahnlich-cli cluster` subcommand group for cluster administration.

**What this includes:**

- **`cli/src/cluster.rs`**: Implementation of admin commands (add-learner, change-membership, remove, metrics, leader, snapshot).
- **`cli/src/config/cli.rs`**: CLI argument definitions for cluster commands.

**Testing**:

- Integration test: bootstrap a cluster, add a learner via CLI, verify membership updates.
- Integration test: call `metrics` and `leader` against a running cluster, verify responses reflect actual cluster state.
- Integration test: trigger `snapshot` via CLI, verify it completes successfully.
- Integration test: `remove` a node, verify membership shrinks.

---

### Milestone 6: Persistence in Cluster Mode

**Goal**: Refactor standalone persistence to use shared serialization functions and disable `utils::Persistence` in cluster mode.

**What this includes:**

- **Refactor `utils::Persistence`**: Replace direct `serde_json::to_writer` / `serde_json::from_reader` calls with the shared serialization functions from Milestone 1. Standalone behavior is unchanged.
- **Disable persistence in cluster mode**: Skip spawning the `Persistence` task when `--cluster-addr` is present. Log a message indicating Raft handles persistence.

**Testing**:

- Integration test: start a standalone node with persistence enabled, write data, restart, verify data is restored (standalone persistence still works after refactor).
- Integration test: start a cluster with `--enable-persistence`, verify persistence task is not spawned and a log message is emitted.

---

### Milestone 7: Cluster Lifecycle Improvements

**Goal**: Reduce manual operational steps for cluster management.

**What this includes:**

- **Auto-join completion**: When `--cluster-join` is specified, the node automatically completes the full join flow (add as learner → receive snapshot → promote to voter once caught up).
- **Graceful shutdown**: Implement Raft cleanup as a `Task` with a `cleanup()` method (see [Graceful Shutdown](#graceful-shutdown)). On any shutdown trigger, remove from voter set and transfer leadership if leader.

**Testing**:

- Integration test: start 3-node cluster, gracefully shut down one node, verify remaining 2 maintain quorum.
- Integration test: auto-join a 4th node, verify it receives data and begins replicating.

---

### Milestone 8: Comprehensive Testing and CI Stabilization

**Goal**: Ensure production readiness with thorough test coverage.

**What this includes:**

- **Extended integration tests:**

  - All DB CRUD operations through a 3-node cluster.
  - All AI CRUD operations through a 3-node AI + 3-node DB deployment.
  - Leader failover: kill leader, verify new leader elected, writes continue.
  - Double failover: kill 2 of 3 nodes, verify quorum loss behavior, restore and verify recovery.
  - Write to follower with forwarding.
  - Snapshot transfer to newly joined node with large data volume.
  - Pipeline operations through Raft.
  
- **CI configuration:**

  - Run cluster tests with `--test-threads=1` to avoid resource contention.
  - Add timeout annotations to prevent CI hangs during Raft election timeouts.
  - Monitor for flaky tests and stabilize.
  
- **Stress tests:**

  - Concurrent writes from multiple clients during leader failover.
  - Large snapshot transfer (many stores, many entries).

---

## Testing Strategy

### Unit Tests

Each state machine implementation has unit tests covering:

- Every command type produces the expected result.
- Malformed payloads return errors without corrupting state.
- Snapshot serialization round-trips correctly.

### Integration Tests

Integration tests start actual server processes and exercise the full gRPC path:

| Scenario                     | What it validates                                                    |
|------------------------------|----------------------------------------------------------------------|
| Single-node cluster CRUD     | Basic Raft path works end-to-end                                     |
| 3-node replication           | Writes replicate to all followers                                    |
| Single failover              | Leader death triggers election, new leader serves writes             |
| Double failover (quorum loss)| Writes fail without quorum, cluster recovers when quorum restored    |
| Write to follower            | Leader forwarding works transparently                                |
| Read from follower           | Follower serves `GetSimN`/`GetKey`/`GetPred` from local state        |
| `ListStores` from follower   | Forwarded to leader, returns consistent result                       |
| Node join with snapshot      | New node receives full state and begins replicating                  |
| Graceful shutdown            | Remaining nodes maintain quorum without unnecessary election         |
| Restart with rocksdb         | Node recovers state from persisted Raft log/snapshots                |
| ClusterInfo query            | Returns correct topology for clustered and standalone nodes          |

### CI Considerations

- Cluster integration tests should run with `--test-threads=1` to avoid port contention and resource exhaustion.
- Use OS-assigned ports (`0`) for all test servers to avoid conflicts.
- Set reasonable timeouts on Raft operations in tests (shorter election timeouts for faster failover detection).
- Monitor for SIGKILL issues in AI tests (observed during parallel execution due to model loading memory pressure).

---

## Open Questions and Future Work

### Per-Request Consistency Override (Future)

By default, most reads are served locally by any node (eventual consistency) and only `ListStores` is forwarded to the leader (linearizable). A future improvement could allow clients to override consistency per-request via a gRPC metadata header (e.g., `x-ahnlich-consistency: linearizable`) to force any request to go through the leader when strict consistency is needed.

### SmartClient: SDK-Side Routing (Future) {#smart-client-future}

`ClusterInfo` provides the raw topology data for clients to build their own routing. A future `SmartClient` wrapper in the SDK would provide this out of the box:

- **Seed-based discovery**: User provides one node address, `SmartClient` calls `ClusterInfo` to discover all members and connect to each.
- **Write-to-leader routing**: Send writes directly to the leader, skipping the forwarding hop for lower latency.
- **Read distribution**: Spread reads across followers for horizontal scaling.
- **Background topology refresh**: Periodically poll `ClusterInfo` to track leadership changes and membership updates.
- **Graceful fallback**: If the topology view is stale, server-side leader forwarding still works — `SmartClient` is an optimization, not a requirement.

### Standalone-to-Cluster Migration (Future)

- Allow bootstrapping a new cluster from a standalone node's existing persistence snapshot, avoiding a cold start when transitioning from standalone to clustered deployment.

### Sharding (Future)

If dataset sizes exceed single-node memory, sharding can be layered on top of replication:

- Consistent hashing to partition stores across shard groups.
- Each shard group is a separate Raft cluster.
- A routing layer directs requests to the appropriate shard.

This is a separate, larger project and is explicitly out of scope for this iteration.

---

## References

- [Raft consensus algorithm paper](https://raft.github.io/raft.pdf)
- [openraft documentation](https://docs.rs/openraft/latest/openraft/)
- [openraft getting started guide](https://docs.rs/openraft/latest/openraft/docs/getting_started/index.html)
- [GitHub issue #271: Horizontal scalability](https://github.com/deven96/ahnlich/issues/271)
- [GitHub PR #276: Replication implementation discussion](https://github.com/deven96/ahnlich/pull/276)
