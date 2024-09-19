use ahnlich_types::ai::AIModel;
use clap::{ArgAction, Args, Parser, Subcommand, ValueEnum};
use dirs::home_dir;
use std::fmt;
use strum::VariantArray;

use crate::engine::ai::models::{Model, ModelInfo};
use std::io::Write;
use std::sync::OnceLock;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash, Ord, ValueEnum, VariantArray)]
pub enum SupportedModels {
    #[clap(name = "all-minilm-l6-v2")]
    AllMiniLML6V2,
    #[clap(name = "all-minilm-l12-v2")]
    AllMiniLML12V2,
    #[clap(name = "bge-base-en-v1.5")]
    BGEBaseEnV15,
    #[clap(name = "bge-large-en-v1.5")]
    BGELargeEnV15,
    #[clap(name = "resnet-50")]
    Resnet50,
    #[clap(name = "clip-vit-b32")]
    ClipVitB32,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Starts Anhlich AI Proxy
    Run(AIProxyConfig),

    /// Outputs all supported models by aiproxy
    SupportedModels(SupportedModelArgs),
}

static DEFAULT_CONFIG: OnceLock<AIProxyConfig> = OnceLock::new();

#[derive(Args, Debug, Clone)]
pub struct AIProxyConfig {
    /// Ahnlich AI proxy host
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    /// Ahnlich AI proxy port
    #[arg(long, default_value_t = 1370)]
    pub port: u16,

    /// Allows server to persist data to disk on occassion
    #[arg(long, default_value_t = false, action=ArgAction::SetTrue)]
    pub(crate) enable_persistence: bool,

    /// persistence location
    #[arg(long, requires_if("true", "enable_persistence"))]
    pub(crate) persist_location: Option<std::path::PathBuf>,
    /// Controls whether we crash or not on startup if persisting load fails
    #[arg(long, default_value_t = false, action=ArgAction::SetFalse)]
    pub(crate) fail_on_startup_if_persist_load_fails: bool,

    /// persistence interval in milliseconds
    /// A new persistence round would be scheduled for persistence_interval into the future after
    /// current persistence round is completed
    #[arg(long, default_value_t = 1000 * 60 * 5)]
    pub(crate) persistence_interval: u64,

    /// Ahnlich Database Host
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub db_host: String,

    /// Ahnlich Database port
    #[arg(long, default_value_t = 1369)]
    pub db_port: u16,

    /// Ahnlich Database Client Connection Pool Size
    #[arg(long, default_value_t = 10)]
    pub db_client_pool_size: usize,

    /// sets size(in bytes) for global allocator used
    /// Defaults to 1 Gi (1 * 1024 * 1024 * 1024)
    #[arg(long, default_value_t = 1_073_741_824)]
    pub allocator_size: usize,

    /// limits the message size of expected messages, defaults to 1MiB (1 * 1024 * 1024)
    #[arg(long, default_value_t = 1_048_576)]
    pub message_size: usize,
    /// Allows enables tracing
    #[arg(long, default_value_t = true, action=ArgAction::SetTrue)]
    pub(crate) enable_tracing: bool,
    /// Otel collector url to send traces to
    #[arg(long, requires_if("true", "enable_tracing"))]
    pub(crate) otel_endpoint: Option<String>,

    ///  Log level
    #[arg(long, default_value_t = String::from("info"))]
    pub(crate) log_level: String,

    ///  Maximum client connections allowed
    ///  Defaults to 1000
    #[arg(long, default_value_t = 1000)]
    pub(crate) maximum_clients: usize,

    /// List of ai models to support in your aiproxy stores
    #[arg(long, value_enum, value_delimiter = ',',
        default_values_t =
    DEFAULT_CONFIG.get_or_init(AIProxyConfig::default)
    .supported_models.clone())
    ]
    pub(crate) supported_models: Vec<SupportedModels>,

    /// AI Model Idle Time in seconds
    /// Defaults to 5 mins
    /// Time in seconds for an unused/idle supported model to be dropped
    /// Futher calls after a drop reinitializes the model from scratch
    #[arg(long, default_value_t = 60 * 5)]
    pub(crate) ai_model_idle_time: u64,

    /// Directory path for storing the model artifacts
    #[arg(long, default_value_os_t =
    DEFAULT_CONFIG.get_or_init(AIProxyConfig::default).model_cache_location.clone())]
    pub(crate) model_cache_location: std::path::PathBuf,
}

#[derive(Debug)]
pub struct ModelConfig {
    pub(crate) supported_models: Vec<SupportedModels>,
    pub(crate) model_cache_location: std::path::PathBuf,
    pub(crate) model_idle_time: u64,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            supported_models: vec![SupportedModels::AllMiniLML6V2],
            model_cache_location: home_dir()
                .map(|mut path| {
                    path.push(".ahnlich");
                    path.push("models");
                    path
                })
                .expect("Default directory could not be resolved."),
            model_idle_time: 60 * 5,
        }
    }
}

impl From<&AIProxyConfig> for ModelConfig {
    fn from(config: &AIProxyConfig) -> Self {
        Self {
            supported_models: config.supported_models.clone(),
            model_cache_location: config.model_cache_location.clone(),
            model_idle_time: config.ai_model_idle_time,
        }
    }
}

impl Default for AIProxyConfig {
    fn default() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            port: 1370,
            enable_persistence: false,
            persist_location: None,
            fail_on_startup_if_persist_load_fails: false,
            persistence_interval: 1000 * 60 * 5,

            db_host: String::from("127.0.0.1"),
            db_port: 1369,
            db_client_pool_size: 10,

            allocator_size: 1_073_741_824,
            message_size: 1_048_576,

            enable_tracing: false,
            otel_endpoint: None,
            log_level: String::from("info"),
            maximum_clients: 1000,
            supported_models: vec![
                // SupportedModels::AllMiniLML6V2,
                // SupportedModels::AllMiniLML12V2,
                // SupportedModels::BGEBaseEnV15,
                SupportedModels::Resnet50,
            ],
            model_cache_location: home_dir()
                .map(|mut path| {
                    path.push(".ahnlich");
                    path.push("models");
                    path
                })
                .expect("Default directory could not be resolved."),
            ai_model_idle_time: 60 * 5,
        }
    }
}

impl AIProxyConfig {
    pub fn os_select_port(mut self) -> Self {
        // allow OS to pick a port
        self.port = 0;
        self
    }

    pub fn set_persist_location(mut self, location: std::path::PathBuf) -> Self {
        self.persist_location = Some(location);
        self
    }

    pub fn set_persistence_interval(mut self, interval: u64) -> Self {
        self.enable_persistence = true;
        self.persistence_interval = interval;
        self
    }

    pub fn set_maximum_clients(mut self, maximum_clients: usize) -> Self {
        self.maximum_clients = maximum_clients;
        self
    }

    pub fn set_model_cache_location(mut self, location: std::path::PathBuf) -> Self {
        self.model_cache_location = location;
        self
    }

    #[cfg(test)]
    pub fn set_supported_models(mut self, models: Vec<SupportedModels>) -> Self {
        self.supported_models = models;
        self
    }
}

impl fmt::Display for SupportedModels {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SupportedModels::AllMiniLML6V2 => write!(f, "all-MiniLM-L6-v2"),
            SupportedModels::AllMiniLML12V2 => write!(f, "all-MiniLM-L12-v2"),
            SupportedModels::BGEBaseEnV15 => write!(f, "BGEBase-En-v1.5"),
            SupportedModels::BGELargeEnV15 => write!(f, "BGELarge-En-v1.5"),
            SupportedModels::Resnet50 => write!(f, "Resnet-50"),
            SupportedModels::ClipVitB32 => write!(f, "ClipVit-B32"),
        }
    }
}

impl From<&AIModel> for SupportedModels {
    fn from(value: &AIModel) -> Self {
        match value {
            AIModel::AllMiniLML6V2 => SupportedModels::AllMiniLML6V2,
            AIModel::AllMiniLML12V2 => SupportedModels::AllMiniLML12V2,
            AIModel::BGEBaseEnV15 => SupportedModels::BGEBaseEnV15,
            AIModel::BGELargeEnV15 => SupportedModels::BGELargeEnV15,
            AIModel::Resnet50 => SupportedModels::Resnet50,
            AIModel::ClipVitB32 => SupportedModels::ClipVitB32,
        }
    }
}

impl From<&SupportedModels> for AIModel {
    fn from(value: &SupportedModels) -> Self {
        match value {
            SupportedModels::AllMiniLML6V2 => AIModel::AllMiniLML6V2,
            SupportedModels::AllMiniLML12V2 => AIModel::AllMiniLML12V2,
            SupportedModels::BGEBaseEnV15 => AIModel::BGEBaseEnV15,
            SupportedModels::BGELargeEnV15 => AIModel::BGELargeEnV15,
            SupportedModels::Resnet50 => AIModel::Resnet50,
            SupportedModels::ClipVitB32 => AIModel::ClipVitB32,
        }
    }
}

impl From<&SupportedModels> for Model {
    fn from(value: &SupportedModels) -> Self {
        let ai_model: AIModel = value.into();
        (&ai_model).into()
    }
}

#[derive(Args, Debug, Clone)]
pub struct SupportedModelArgs {
    ///  Models to display information about
    #[arg(long, value_delimiter = ',')]
    pub names: Vec<SupportedModels>,
}

impl SupportedModelArgs {
    pub fn list_supported_models(&self) -> String {
        let mut output = String::new();

        for supported_model in SupportedModels::VARIANTS.iter() {
            let aimodel: AIModel = supported_model.into();
            let model: Model = (&aimodel).into();
            output.push_str(format!("{}, ", model.model_name()).as_str())
        }
        output
    }
    pub fn list_supported_models_verbose(&self) -> String {
        let mut output = vec![];

        for supported_model in self.names.iter() {
            let aimodel: AIModel = supported_model.into();
            let model: Model = (&aimodel).into();
            output.push(ModelInfo::build(&model))
        }
        serde_json::to_string_pretty(&output)
            .expect("Failed Generate Supported Models Verbose Text")
    }

    pub fn output(&self) {
        let mut stdout = StandardStream::stdout(ColorChoice::Always);
        stdout
            .set_color(ColorSpec::new().set_fg(Some(Color::Green)))
            .expect("Failed to set output Color");

        let mut text = "\n\nDisplaying Supported Models \n\n".to_string();
        if !self.names.is_empty() {
            text.push_str(&self.list_supported_models_verbose());
        } else {
            text.push_str(&self.list_supported_models());
        }

        writeln!(&mut stdout, "{}", text).expect("Failed to write output");
    }
}
