use clap::{ArgAction, Args};

#[derive(Args, Debug, Clone)]
pub struct CommandLineConfig {
    /// Host
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    /// Allows server to persist data to disk on occassion
    #[arg(long, default_value_t = false, action=ArgAction::SetTrue)]
    pub enable_persistence: bool,

    /// persistence location
    #[arg(long, requires_if("true", "enable_persistence"))]
    pub persist_location: Option<std::path::PathBuf>,
    /// Controls whether we crash or not on startup if persisting load fails
    #[arg(long, default_value_t = false, action=ArgAction::SetFalse)]
    pub fail_on_startup_if_persist_load_fails: bool,

    /// persistence interval in milliseconds
    /// A new persistence round would be scheduled for persistence_interval into the future after
    /// current persistence round is completed
    #[arg(long, default_value_t = 1000 * 60 * 5)]
    pub persistence_interval: u64,

    /// sets size(in bytes) for global allocator used
    /// Defaults to 1 Gi (1 * 1024 * 1024 * 1024)
    #[arg(long, default_value_t = 1_073_741_824)]
    pub allocator_size: usize,

    /// limits the message size of expected messages, defaults to 1MiB (1 * 1024 * 1024)
    #[arg(long, default_value_t = 1_048_576)]
    pub message_size: usize,
    /// Allows enables tracing
    #[arg(long, default_value_t = false, action=ArgAction::SetTrue)]
    pub enable_tracing: bool,
    /// Otel collector url to send traces to
    #[arg(long, requires_if("true", "enable_tracing"))]
    pub otel_endpoint: Option<String>,

    ///  Log level
    #[arg(long, default_value_t = String::from("info"))]
    pub log_level: String,

    ///  Maximum client connections allowed
    ///  Defaults to 1000
    #[arg(long, default_value_t = 1000)]
    pub maximum_clients: usize,

    ///  CPU threadpool size
    ///  Defaults to 16
    #[arg(long, default_value_t = 16)]
    pub threadpool_size: usize,
}

impl Default for CommandLineConfig {
    fn default() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            enable_persistence: false,
            persist_location: None,
            fail_on_startup_if_persist_load_fails: false,
            persistence_interval: 1000 * 60 * 5,
            allocator_size: 1_073_741_824,
            message_size: 1_048_576,

            enable_tracing: false,
            otel_endpoint: None,
            log_level: String::from("info"),
            maximum_clients: 1000,
            threadpool_size: 16,
        }
    }
}
