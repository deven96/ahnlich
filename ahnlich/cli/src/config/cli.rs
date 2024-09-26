use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub commands: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Ahnlich(AhnlichCliConfig),
}

#[derive(Debug, Copy, Clone, Hash, ValueEnum)]
pub enum Agent {
    DB,
    AI,
}

#[derive(Args, Debug, Clone)]
pub struct AhnlichCliConfig {
    /// The Ahnlich server to connect to (DB or AI)
    #[arg(long, required(true))]
    pub agent: Agent,

    /// Host to connect to Ahnlich AI or DB
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    /// Host to connect to Ahnlich AI or DB
    #[arg(long, default_value_t = 1369)]
    pub port: u16,
}
