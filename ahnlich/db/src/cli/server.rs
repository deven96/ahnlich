use ahnlich_replication::config::RaftStorageEngine;
use clap::{ArgAction, Args, Parser, Subcommand};
use utils::cli::CommandLineConfig;

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Starts Anhlich database
    Run(ServerConfig),
}

#[derive(Args, Debug, Clone)]
pub struct ServerConfig {
    #[arg(long, default_value_t = 1369)]
    pub port: u16,
    /// Enable Raft clustering
    #[arg(long, action=ArgAction::SetTrue)]
    pub cluster_enabled: bool,
    /// Raft node id
    #[arg(long, requires = "cluster_enabled")]
    pub raft_node_id: u64,
    /// Raft internal address host:port
    #[arg(long, requires = "cluster_enabled")]
    pub raft_addr: String,
    /// Raft admin address host:port
    #[arg(long, requires = "cluster_enabled")]
    pub admin_addr: String,
    /// Raft storage: mem or rocksdb
    #[arg(long, value_enum, default_value_t = RaftStorageEngine::Memory, requires = "cluster_enabled")]
    pub raft_storage: RaftStorageEngine,
    /// Raft data dir (required for rocksdb storage)
    #[arg(
        long,
        requires = "cluster_enabled",
        required_if_eq("raft_storage", "rocksdb")
    )]
    pub raft_data_dir: Option<std::path::PathBuf>,
    /// Snapshot after N logs
    #[arg(long, default_value_t = 1000, requires = "cluster_enabled")]
    pub raft_snapshot_logs: u64,
    /// Join existing cluster via admin addr host:port
    #[arg(long)]
    pub raft_join: Option<String>,
    #[clap(flatten)]
    pub common: CommandLineConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 1369,
            cluster_enabled: false,
            raft_node_id: 0,
            raft_addr: String::from("127.0.0.1:0"),
            admin_addr: String::from("127.0.0.1:0"),
            raft_storage: RaftStorageEngine::Memory,
            raft_data_dir: None,
            raft_snapshot_logs: 1000,
            raft_join: None,
            common: CommandLineConfig::default(),
        }
    }
}

impl ServerConfig {
    pub fn os_select_port(mut self) -> Self {
        // allow OS to pick a port
        self.port = 0;
        self
    }

    pub fn persist_location(mut self, location: std::path::PathBuf) -> Self {
        self.common.persist_location = Some(location);
        self
    }

    pub fn persistence_interval(mut self, interval: u64) -> Self {
        self.common.enable_persistence = true;
        self.common.persistence_interval = interval;
        self
    }

    pub fn enable_tracing(mut self) -> Self {
        self.common.enable_tracing = true;
        self
    }

    pub fn maximum_clients(mut self, maximum_clients: usize) -> Self {
        self.common.maximum_clients = maximum_clients;
        self
    }

    pub fn with_auth(
        mut self,
        auth_config: std::path::PathBuf,
        tls_cert: std::path::PathBuf,
        tls_key: std::path::PathBuf,
    ) -> Self {
        self.common.enable_auth = true;
        self.common.auth_config = Some(auth_config);
        self.common.tls_cert = Some(tls_cert);
        self.common.tls_key = Some(tls_key);
        self
    }
}
