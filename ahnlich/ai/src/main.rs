use clap::Parser;

use std::error::Error;
use utils::server::AhnlichServerUtils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = ahnlich_ai_proxy::cli::Cli::parse();
    match cli.command {
        ahnlich_ai_proxy::cli::Commands::Run(config) => {
            let server = ahnlich_ai_proxy::server::handler::AIProxyServer::new(config).await?;
            // TODO: Use server task manager here to spawn inference thread;
            server.start().await?;
        }
        ahnlich_ai_proxy::cli::Commands::SupportedModels(config) => config.output(),
    }
    Ok(())
}
