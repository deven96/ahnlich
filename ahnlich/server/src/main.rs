use clap::Parser;
use server::cli::{Cli, Commands};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Run(config) => {
            let server = server::Server::new(config).await?;
            server.start().await?;
        }
    }
    Ok(())
}
