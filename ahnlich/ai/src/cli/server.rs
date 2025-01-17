use ahnlich_types::ai::AIModel;
use clap::{Args, Parser, Subcommand, ValueEnum};
use dirs::home_dir;
use std::fmt;
use strum::VariantArray;

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::sync::OnceLock;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use utils::cli::CommandLineConfig;

#[derive(
    Default,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Hash,
    Ord,
    ValueEnum,
    VariantArray,
    Serialize,
    Deserialize,
)]
pub enum SupportedModels {
    #[default]
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
    #[clap(name = "clip-vit-b32-image")]
    ClipVitB32Image,
    #[clap(name = "clip-vit-b32-text")]
    ClipVitB32Text,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[allow(clippy::large_enum_variant)]
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
    /// Ahnlich AI proxy port
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(AIProxyConfig::default).port.clone())]
    pub port: u16,

    /// Ahnlich Database Host
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(AIProxyConfig::default).db_host.clone())]
    pub db_host: String,

    /// Ahnlich Database port
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(AIProxyConfig::default).db_port.clone())]
    pub db_port: u16,

    /// Ahnlich Database Client Connection Pool Size
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(AIProxyConfig::default).db_client_pool_size.clone())]
    pub db_client_pool_size: usize,

    /// List of ai models to support in your aiproxy stores
    #[arg(long, value_enum, value_delimiter = ',', default_values_t =
    DEFAULT_CONFIG.get_or_init(AIProxyConfig::default).supported_models.clone())]
    pub(crate) supported_models: Vec<SupportedModels>,

    /// AI Model Idle Time in seconds
    /// Defaults to 5 mins
    /// Time in seconds for an unused/idle supported model to be dropped
    /// Futher calls after a drop reinitializes the model from scratch
    #[arg(long, default_value_t =
    DEFAULT_CONFIG.get_or_init(AIProxyConfig::default).ai_model_idle_time.clone())]
    pub(crate) ai_model_idle_time: u64,

    /// Directory path for storing the model artifacts
    #[arg(long, default_value_os_t =
    DEFAULT_CONFIG.get_or_init(AIProxyConfig::default).model_cache_location.clone())]
    pub(crate) model_cache_location: std::path::PathBuf,

    #[clap(flatten)]
    pub common: CommandLineConfig,
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
            port: 1370,
            db_host: String::from("127.0.0.1"),
            db_port: 1369,
            db_client_pool_size: 10,
            supported_models: vec![
                SupportedModels::AllMiniLML6V2,
                SupportedModels::AllMiniLML12V2,
                SupportedModels::BGEBaseEnV15,
                SupportedModels::BGELargeEnV15,
                SupportedModels::ClipVitB32Text,
                SupportedModels::Resnet50,
                SupportedModels::ClipVitB32Image,
            ],
            model_cache_location: home_dir()
                .map(|mut path| {
                    path.push(".ahnlich");
                    path.push("models");
                    path
                })
                .expect("Default directory could not be resolved."),
            ai_model_idle_time: 60 * 5,
            common: CommandLineConfig::default(),
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
        self.common.persist_location = Some(location);
        self
    }

    pub fn set_persistence_interval(mut self, interval: u64) -> Self {
        self.common.enable_persistence = true;
        self.common.persistence_interval = interval;
        self
    }

    pub fn set_maximum_clients(mut self, maximum_clients: usize) -> Self {
        self.common.maximum_clients = maximum_clients;
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
            SupportedModels::ClipVitB32Image => write!(f, "ClipVit-B32-Image"),
            SupportedModels::ClipVitB32Text => write!(f, "ClipVit-B32-Text"),
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
            AIModel::ClipVitB32Image => SupportedModels::ClipVitB32Image,
            AIModel::ClipVitB32Text => SupportedModels::ClipVitB32Text,
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
            SupportedModels::ClipVitB32Image => AIModel::ClipVitB32Image,
            SupportedModels::ClipVitB32Text => AIModel::ClipVitB32Text,
        }
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
            output.push_str(format!("{}, ", supported_model).as_str())
        }
        output
    }
    pub fn list_supported_models_verbose(&self) -> String {
        let mut output = vec![];

        for supported_model in self.names.iter() {
            let model_details = supported_model.to_model_details();
            output.push(model_details)
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
