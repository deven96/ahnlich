use clap::ValueEnum;
use std::net::SocketAddr;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, ValueEnum)]
pub enum RaftStorageEngine {
    #[clap(name = "memory")]
    Memory,
    #[clap(name = "rocksdb")]
    RocksDb,
}

#[derive(Debug, Clone)]
pub struct RaftConfig {
    pub node_id: u64,
    pub raft_addr: SocketAddr,
    pub admin_addr: SocketAddr,
    pub service_addr: SocketAddr,
    pub storage: RaftStorageEngine,
    pub data_dir: Option<PathBuf>,
    pub snapshot_logs: u64,
    pub join: Option<SocketAddr>,
}
