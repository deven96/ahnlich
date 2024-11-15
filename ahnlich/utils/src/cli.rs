use clap::{ArgAction, Args};
use std::sync::OnceLock;

static DEFAULT_CONFIG: OnceLock<CommandLineConfig> = OnceLock::new();
const MIN_ALLOCATION_SIZE: usize = 10 * 1024 * 1024; // 10mb

#[derive(Args, Debug, Clone)]
pub struct CommandLineConfig {
    /// Host
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).host.clone())]
    pub host: String,

    /// Allows server to persist data to disk on occassion
    #[arg(long, action=ArgAction::SetTrue, default_value_t =
    DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).enable_persistence.clone())]
    pub enable_persistence: bool,

    /// persistence location
    #[arg(long, requires_if("true", "enable_persistence"))]
    pub persist_location: Option<std::path::PathBuf>,
    /// Controls whether we crash or not on startup if persisting load fails
    #[arg(long, action=ArgAction::SetFalse, default_value_t =
    DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).fail_on_startup_if_persist_load_fails.clone())]
    pub fail_on_startup_if_persist_load_fails: bool,

    /// persistence interval in milliseconds
    /// A new persistence round would be scheduled for persistence_interval into the future after
    /// current persistence round is completed
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).persistence_interval.clone())]
    pub persistence_interval: u64,

    /// sets size(in bytes) for global allocator used
    /// Defaults to 1 Gi (1 * 1024 * 1024 * 1024)
    /// Would throw a memory allocation error and stopping the server
    #[arg(long, value_parser = validate_allocator_size,
        default_value_t = DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).allocator_size.clone()
    )]
    pub allocator_size: usize,

    /// limits the message size of expected messages, defaults to 1MiB (1 * 1024 * 1024)
    /// Would disconnect the client only
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).message_size.clone())]
    pub message_size: usize,
    /// Allows enables tracing
    #[arg(long, action=ArgAction::SetTrue, default_value_t =
    DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).enable_tracing.clone())]
    pub enable_tracing: bool,
    /// Otel collector url to send traces to
    #[arg(long, requires_if("true", "enable_tracing"))]
    pub otel_endpoint: Option<String>,

    ///  Log level
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).log_level.clone())]
    pub log_level: String,

    ///  Maximum client connections allowed
    ///  Defaults to 1000
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).maximum_clients.clone())]
    pub maximum_clients: usize,

    ///  CPU threadpool size
    ///  Defaults to 16
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(CommandLineConfig::default).threadpool_size.clone())]
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
            log_level: String::from("info,hf_hub=warn"),
            maximum_clients: 1000,
            threadpool_size: 16,
        }
    }
}

fn validate_allocator_size(val: &str) -> Result<usize, String> {
    let size: usize = val.parse::<usize>().map_err(|err| err.to_string())?;

    if size < MIN_ALLOCATION_SIZE {
        Err(format!(
            "Size mut be atleast {} bytes (10 MB)",
            MIN_ALLOCATION_SIZE
        ))
    } else {
        Ok(size)
    }
}

pub fn validate_persistence(
    allocated_size: usize,
    persistence_file: Option<&std::path::PathBuf>,
) -> Result<(), String> {
    if let Some(path_file) = persistence_file {
        let path = path_file.as_path();
        if path.is_file() {
            let file_metadata = std::fs::metadata(path).map_err(|err| err.to_string())?;
            if (allocated_size / file_metadata.len() as usize) < 2 {
                return Err(
                    "Allocated memory should be more than two times your persistence_file size"
                        .to_string(),
                );
            }
        }
    }

    Ok(())
}
