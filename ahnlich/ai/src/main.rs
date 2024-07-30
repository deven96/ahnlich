use clap::Parser;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = ahnlich_ai_proxy::cli::Cli::parse();
    match &cli.command {
        ahnlich_ai_proxy::cli::Commands::Start(config) => {
            let server =
                ahnlich_ai_proxy::server::handler::AIProxyServer::new(config.clone()).await?;
            server.start().await?;
        }
    }
    Ok(())
}
