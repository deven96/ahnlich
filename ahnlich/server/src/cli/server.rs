use clap::{ArgAction, Args, Parser, Subcommand};

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
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    #[arg(long, default_value_t = 1369)]
    pub port: u16,

    /// Allows server to persist data to disk on occassion
    #[arg(long, default_value_t = false, action=ArgAction::SetTrue)]
    pub(crate) enable_persistence: bool,

    /// persistence location
    #[arg(long, requires_if("true", "enable_persistence"))]
    pub(crate) persist_location: Option<std::path::PathBuf>,

    /// persistence intervals in milliseconds
    #[arg(long, default_value_t = 1000 * 60 * 5)]
    pub(crate) persistence_intervals: u64,
}

impl ServerConfig {
    fn new() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            port: 1396,
            enable_persistence: false,
            persist_location: None,
            persistence_intervals: 1000 * 60 * 5,
        }
    }
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}
