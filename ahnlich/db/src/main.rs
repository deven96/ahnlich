use clap::Parser;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = ahnlich_db::cli::Cli::parse();
    match &cli.command {
        ahnlich_db::cli::Commands::Run(config) => {
            let server = ahnlich_db::server::handler::Server::new(config).await?;
            server.start().await?;
        }
    }
    Ok(())
}
