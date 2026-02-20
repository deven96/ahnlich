pub mod admin;
pub mod config;
pub mod network;
pub mod storage;
pub mod types;

pub mod proto {
    pub mod cluster_admin {
        tonic::include_proto!("services.cluster_admin");
    }

    pub mod raft_internal {
        tonic::include_proto!("services.raft_internal");
    }
}
