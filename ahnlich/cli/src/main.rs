use ahnlich_cli::{connect::AgentClient, term::Term};
use clap::Parser;
use std::io;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = ahnlich_cli::config::cli::Cli::parse();

    match cli.commands {
        ahnlich_cli::config::cli::Commands::Ahnlich(config) => {
            let client = AgentClient::create_client(config.agent, &config.host, config.port)
                .await
                .map_err(io::Error::other)?;

            if !client
                .is_valid_connection()
                .await
                .map_err(io::Error::other)?
            {
                return Err(io::Error::other(format!(
                    "Connected Server is not a valid {client} Server"
                )));
            }
            let term = Term::new(client);
            term.welcome_message()?;
            term.run().await?;
        }
        ahnlich_cli::config::cli::Commands::Cluster(config) => {
            use ahnlich_cli::config::cli::ClusterCommands;
            match config.command {
                ClusterCommands::Init(cmd) => {
                    ahnlich_cli::cluster::init_cluster(cmd)
                        .await
                        .map_err(io::Error::other)?;
                }
                ClusterCommands::Join(cmd) => {
                    ahnlich_cli::cluster::join_cluster(cmd)
                        .await
                        .map_err(io::Error::other)?;
                }
                ClusterCommands::AddLearner(cmd) => {
                    ahnlich_cli::cluster::add_learner(cmd)
                        .await
                        .map_err(io::Error::other)?;
                }
                ClusterCommands::ChangeMembership(cmd) => {
                    ahnlich_cli::cluster::change_membership(cmd)
                        .await
                        .map_err(io::Error::other)?;
                }
                ClusterCommands::Remove(cmd) => {
                    ahnlich_cli::cluster::remove_node(cmd)
                        .await
                        .map_err(io::Error::other)?;
                }
                ClusterCommands::Metrics(cmd) => {
                    let metrics = ahnlich_cli::cluster::metrics(cmd)
                        .await
                        .map_err(io::Error::other)?;
                    println!("{}", String::from_utf8_lossy(&metrics));
                }
                ClusterCommands::Leader(cmd) => {
                    let (id, addr) = ahnlich_cli::cluster::leader(cmd)
                        .await
                        .map_err(io::Error::other)?;
                    println!("leader_id={id} leader_addr={addr}");
                }
                ClusterCommands::Snapshot(cmd) => {
                    ahnlich_cli::cluster::snapshot(cmd)
                        .await
                        .map_err(io::Error::other)?;
                }
            }
        }
    }
    Ok(())
}
