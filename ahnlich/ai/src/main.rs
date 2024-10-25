use clap::Parser;

use std::error::Error;
use utils::{cli::validate_persistence, server::AhnlichServerUtils};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = ahnlich_ai_proxy::cli::Cli::parse();
    match cli.command {
        ahnlich_ai_proxy::cli::Commands::Run(config) => {
            if config.common.enable_persistence {
                validate_persistence(
                    config.common.allocator_size,
                    config.common.persist_location.as_ref(),
                )?;
            }

            let server = ahnlich_ai_proxy::server::handler::AIProxyServer::new(config).await?;
            server.start().await?;
        }
        ahnlich_ai_proxy::cli::Commands::SupportedModels(config) => config.output(),
    }
    Ok(())
}
