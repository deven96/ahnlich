use ahnlich_replication::config::RaftStorageEngine;
use clap::{Args, Parser, Subcommand};
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
    #[arg(long)]
    pub cluster_addr: Option<std::net::SocketAddr>,
    #[arg(long, default_value_t = false, conflicts_with = "cluster_join")]
    pub cluster_bootstrap: bool,
    #[arg(long, conflicts_with = "cluster_bootstrap")]
    pub cluster_join: Option<std::net::SocketAddr>,
    #[arg(long, value_enum, default_value_t = RaftStorageEngine::RocksDb)]
    pub cluster_storage: RaftStorageEngine,
    #[arg(long, requires_if("rocksdb", "cluster_storage"))]
    pub cluster_data_dir: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = 1000)]
    pub cluster_snapshot_logs: u64,
    #[arg(long, default_value_t = 300_000)]
    pub cluster_snapshot_interval: u64,
    #[clap(flatten)]
    pub common: CommandLineConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 1369,
            cluster_addr: None,
            cluster_bootstrap: false,
            cluster_join: None,
            cluster_storage: RaftStorageEngine::RocksDb,
            cluster_data_dir: None,
            cluster_snapshot_logs: 1000,
            cluster_snapshot_interval: 300_000,
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

    pub fn is_clustered(&self) -> bool {
        self.cluster_addr.is_some()
    }

    pub fn cluster_addr(mut self, addr: std::net::SocketAddr) -> Self {
        self.cluster_addr = Some(addr);
        self
    }

    pub fn cluster_bootstrap(mut self, addr: std::net::SocketAddr) -> Self {
        self.cluster_addr = Some(addr);
        self.cluster_bootstrap = true;
        self.cluster_join = None;
        self
    }

    pub fn cluster_join(mut self, addr: std::net::SocketAddr, join: std::net::SocketAddr) -> Self {
        self.cluster_addr = Some(addr);
        self.cluster_bootstrap = false;
        self.cluster_join = Some(join);
        self
    }

    pub fn cluster_data_dir(mut self, dir: std::path::PathBuf) -> Self {
        self.cluster_data_dir = Some(dir);
        self
    }

    pub fn cluster_storage(mut self, storage: RaftStorageEngine) -> Self {
        self.cluster_storage = storage;
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

    pub fn disable_mmap(mut self) -> Self {
        self.common.enable_mmap = false;
        self
    }
}
