use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    ///  Generate Yaml file for query and server response types
    Generate(GenerateTypesConfig),
    /// Create type definitions for a language
    CreateClient(CreateClientConfig),
}

#[derive(Args, Debug, Clone)]
pub struct GenerateTypesConfig {
    /// location to output generated client language spec
    #[arg(long)]
    pub(crate) output_dir: Option<std::path::PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct CreateClientConfig {
    #[arg(long)]
    pub(crate) input_spec_dir: Option<std::path::PathBuf>,

    /// location to output generated client language spec
    #[arg(long)]
    pub(crate) output_dir: Option<std::path::PathBuf>,

    /// Language for client. Could be python, go etc
    #[arg(value_enum)]
    pub(crate) language: Language,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum Language {
    /// python
    Python,
    /// golang
    Golang,
}
