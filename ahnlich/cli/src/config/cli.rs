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
    Cluster(ClusterCliConfig),
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
    #[arg(long)]
    pub port: Option<u16>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum ClusterCommands {
    Init(ClusterInit),
    Join(ClusterJoin),
    AddLearner(ClusterAddLearner),
    ChangeMembership(ClusterChangeMembership),
    Remove(ClusterRemove),
    Metrics(ClusterMetrics),
    Leader(ClusterLeader),
    Snapshot(ClusterSnapshot),
}

#[derive(Args, Debug, Clone)]
pub struct ClusterCliConfig {
    #[command(subcommand)]
    pub command: ClusterCommands,
}

#[derive(Args, Debug, Clone)]
pub struct ClusterNodeInfo {
    #[arg(long)]
    pub node_id: u64,
    #[arg(long)]
    pub raft_addr: String,
    #[arg(long)]
    pub admin_addr: String,
    #[arg(long)]
    pub service_addr: String,
}

#[derive(Args, Debug, Clone)]
pub struct ClusterInit {
    #[arg(long)]
    pub admin_addr: String,
    #[arg(long, value_delimiter = ';')]
    pub nodes: Vec<String>,
}

#[derive(Args, Debug, Clone)]
pub struct ClusterJoin {
    #[command(flatten)]
    pub node: ClusterNodeInfo,
    #[arg(long)]
    pub join: String,
}

#[derive(Args, Debug, Clone)]
pub struct ClusterAddLearner {
    #[command(flatten)]
    pub node: ClusterNodeInfo,
    #[arg(long)]
    pub admin_addr: String,
}

#[derive(Args, Debug, Clone)]
pub struct ClusterChangeMembership {
    #[arg(long)]
    pub admin_addr: String,
    #[arg(long, value_delimiter = ',')]
    pub node_ids: Vec<u64>,
}

#[derive(Args, Debug, Clone)]
pub struct ClusterRemove {
    #[arg(long)]
    pub admin_addr: String,
    #[arg(long)]
    pub node_id: u64,
}

#[derive(Args, Debug, Clone)]
pub struct ClusterMetrics {
    #[arg(long)]
    pub admin_addr: String,
}

#[derive(Args, Debug, Clone)]
pub struct ClusterLeader {
    #[arg(long)]
    pub admin_addr: String,
}

#[derive(Args, Debug, Clone)]
pub struct ClusterSnapshot {
    #[arg(long)]
    pub admin_addr: String,
}
