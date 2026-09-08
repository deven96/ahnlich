use clap::Parser;

use std::error::Error;
use utils::{cli::validate_persistence, server::AhnlichServerUtils};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = ahnlich_db::cli::Cli::parse();
    match &cli.command {
        ahnlich_db::cli::Commands::Run(config) => {
            #[cfg(feature = "bench-existing-predicate-index")]
            eprintln!("benchmark: existing-predicate-index candidate enabled");
            if config.common.enable_persistence {
                validate_persistence(
                    config.common.allocator_size,
                    config.common.persist_location.as_ref(),
                )?;
            }
            let server = ahnlich_db::server::handler::Server::new(config).await?;

            server.start().await?;
        }
    }
    Ok(())
}
