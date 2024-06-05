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

    /// sets size(in bytes) for global allocator used
    /// Defaults to 1 Gi (1 * 1024 * 1024 * 1024)
    #[arg(long, default_value_t = 1_073_741_824)]
    pub allocator_size: usize,

    /// limits the message size of expected messages, defaults to 1MiB (1 * 1024 * 1024)
    #[arg(long, default_value_t = 1_048_576)]
    pub message_size: usize,
    /// Allows enables tracing
    #[arg(long, default_value_t = false, action=ArgAction::SetTrue)]
    pub(crate) enable_tracing: bool,

    /// Otel collector url to send traces to
    #[arg(long, requires_if("true", "enable_tracing"))]
    pub(crate) otel_endpoint: Option<String>,

    ///  Log level
    #[arg(long, default_value_t = String::from("info"))]
    pub(crate) log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            #[cfg(not(test))]
            port: 1369,
            // allow OS to pick a port
            #[cfg(test)]
            port: 0,
            enable_persistence: false,
            persist_location: None,
            persistence_intervals: 1000 * 60 * 5,
            allocator_size: 1_073_741_824,
            message_size: 1_048_576,

            enable_tracing: false,
            otel_endpoint: None,
            log_level: String::from("info"),
        }
    }
}
