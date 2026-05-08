//! ahnlich-replication
//!
//! Internal Raft-based replication primitives shared between `ahnlich-db` and
//! `ahnlich-ai`. This crate exposes:
//!
//! * gRPC transport for openraft RPCs (private to the cluster)
//! * pluggable log/state-machine storage with a RocksDB backend (production)
//!   and an optional in-memory backend (gated by the `test-utils` feature)
//! * a `ClusterAdminService` for membership operations
//! * shared command/response types and config structures
//!
//! No DB or AI runtime behavior lives here. The `ClusterInfo` topology helper
//! returns a private [`cluster_info::ClusterNodeInfo`] struct; mapping into the
//! public `ahnlich_types::shared::cluster` proto types is done in the DB and
//! AI server crates.

pub mod admin;
pub mod cluster_info;
pub mod config;
pub mod network;
pub mod node;
pub mod storage;
pub mod test_utils;
pub mod types;

pub mod proto {
    pub mod cluster_admin {
        tonic::include_proto!("services.cluster_admin");
    }

    pub mod raft_internal {
        tonic::include_proto!("services.raft_internal");
    }
}
