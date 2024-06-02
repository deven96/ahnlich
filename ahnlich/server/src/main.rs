use clap::Parser;
use env_logger::Env;
use server::cli::{Cli, Commands};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Run(config) => {
            let server = server::Server::new(config).await?;
            server.start().await?;
        }
    }
    Ok(())
}
