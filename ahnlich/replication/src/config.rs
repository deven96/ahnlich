use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ValueEnum,
)]
pub enum RaftStorageEngine {
    /// In-memory storage. **Testing only: no durability across restarts.**
    /// Selecting this for a production deployment defeats the purpose of
    /// replication. The crate emits a runtime warning when this is chosen
    /// outside `cfg(test)` or the `test-utils` feature.
    #[clap(name = "memory")]
    Memory,
    /// Persistent storage backed by RocksDB. The default and only supported
    /// backend for production deployments.
    #[default]
    #[clap(name = "rocksdb")]
    RocksDb,
}

#[derive(Debug, Clone)]
pub struct RaftConfig {
    /// Stable node identifier. The spec calls for deriving this by hashing
    /// `raft_addr`; the derivation lives in the server crates so this struct
    /// stays decoupled from any particular hashing choice.
    pub node_id: u64,
    /// `host:port` for cluster traffic, both Raft RPCs (AppendEntries,
    /// InstallSnapshot, Vote) and cluster administration (membership,
    /// metrics) share this single port per the spec's `--cluster-addr`.
    pub raft_addr: SocketAddr,
    /// `host:port` of the public client-facing service. Recorded in Raft node
    /// metadata so peers can advertise it via `ClusterInfo`.
    pub service_addr: SocketAddr,
    pub storage: RaftStorageEngine,
    /// Filesystem path for RocksDB files. Required when
    /// `storage == RaftStorageEngine::RocksDb`.
    pub data_dir: Option<PathBuf>,
    /// Snapshot trigger: count-based. Snapshot after this many committed
    /// log entries since the last snapshot.
    pub snapshot_logs: u64,
    /// Snapshot trigger: time-based dirty check. A background task wakes
    /// every N milliseconds and triggers a snapshot if any mutations have
    /// been applied since the last one.
    pub snapshot_interval_ms: u64,
    /// What this node should do at startup. The three variants are mutually
    /// exclusive by construction; collapsing them into `Option<SocketAddr>`
    /// would conflate "bootstrap a new cluster" with "rejoin from local
    /// state", so the lifecycle is its own type.
    pub lifecycle: ClusterLifecycle,
}

/// Startup lifecycle decision for a Raft node, mapping 1:1 to the spec's
/// `--cluster-bootstrap` / `--cluster-join` / (neither) flag combinations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterLifecycle {
    /// Bootstrap a new single-node cluster on this node. The first node in
    /// a fresh deployment uses this; it elects itself leader and accepts
    /// joiners afterward via the cluster admin RPCs.
    Bootstrap,
    /// Contact an existing cluster member at the given address and join as
    /// a learner; openraft promotes to voter once caught up.
    Join(SocketAddr),
    /// Recover from this node's local Raft log and snapshot, used when the
    /// node was already a member and is restarting against persisted state.
    Existing,
}
