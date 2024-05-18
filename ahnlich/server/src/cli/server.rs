use clap::{ArgAction, Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Commands,
}

#[derive(Args, Debug)]
pub(crate) struct ServerConfig {
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub(crate) host: String,

    #[arg(long, default_value_t = 1369)]
    pub(crate) port: u16,

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

#[derive(Subcommand)]
pub(crate) enum Commands {
    /// Starts Anhlich database
    Run(ServerConfig),
}
