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
    #[clap(flatten)]
    pub common: CommandLineConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 1369,
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

    pub fn maximum_clients(mut self, maximum_clients: usize) -> Self {
        self.common.maximum_clients = maximum_clients;
        self
    }
}
