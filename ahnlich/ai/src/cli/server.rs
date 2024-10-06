use ahnlich_types::ai::AIModel;
use clap::{Args, Parser, Subcommand, ValueEnum};
use strum::VariantArray;

use crate::engine::ai::models::{Model, ModelInfo};
use std::io::Write;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};
use utils::cli::CommandLineConfig;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash, Ord, ValueEnum, VariantArray)]
pub enum SupportedModels {
    Llama3,
    Dalle3,
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

#[derive(Args, Debug, Clone)]
pub struct AIProxyConfig {
    /// Ahnlich AI proxy port
    #[arg(long, default_value_t = 1370)]
    pub port: u16,

    /// Ahnlich Database Host
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub db_host: String,

    /// Ahnlich Database port
    #[arg(long, default_value_t = 1369)]
    pub db_port: u16,

    /// Ahnlich Database Client Connection Pool Size
    #[arg(long, default_value_t = 10)]
    pub db_client_pool_size: usize,

    /// List of ai models to support in your aiproxy stores
    #[arg(long, required(true), value_delimiter = ',')]
    pub(crate) supported_models: Vec<SupportedModels>,

    /// AI Model Idle Time in seconds
    /// Defaults to 5 mins
    /// Time in seconds for an unused/idle supported model to be dropped
    /// Futher calls after a drop reinitializes the model from scratch
    #[arg(long, default_value_t = 60 * 5)]
    pub(crate) ai_model_idle_time: u64,

    #[clap(flatten)]
    pub(crate) common: CommandLineConfig,
}

impl Default for AIProxyConfig {
    fn default() -> Self {
        Self {
            port: 1370,
            db_host: String::from("127.0.0.1"),
            db_port: 1369,
            db_client_pool_size: 10,
            supported_models: vec![SupportedModels::Llama3, SupportedModels::Dalle3],
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

    #[cfg(test)]
    pub fn set_supported_models(mut self, models: Vec<SupportedModels>) -> Self {
        self.supported_models = models;
        self
    }
}

impl From<&AIModel> for SupportedModels {
    fn from(value: &AIModel) -> Self {
        match value {
            AIModel::Llama3 => SupportedModels::Llama3,
            AIModel::DALLE3 => SupportedModels::Dalle3,
        }
    }
}

impl From<&SupportedModels> for AIModel {
    fn from(value: &SupportedModels) -> Self {
        match value {
            SupportedModels::Llama3 => AIModel::Llama3,
            SupportedModels::Dalle3 => AIModel::DALLE3,
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
