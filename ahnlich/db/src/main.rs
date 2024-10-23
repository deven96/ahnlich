use clap::Parser;

use std::{error::Error, os::unix::fs::MetadataExt, path::PathBuf};
use utils::server::AhnlichServerUtils;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = ahnlich_db::cli::Cli::parse();
    match &cli.command {
        ahnlich_db::cli::Commands::Run(config) => {
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

fn validate_persistence(
    allocated_size: usize,
    persistence_file: Option<&PathBuf>,
) -> Result<(), String> {
    if let Some(path_file) = persistence_file {
        let path = path_file.as_path();
        if path.is_file() {
            let metadata = std::fs::metadata(path).map_err(|err| err.to_string())?;
            if (allocated_size / metadata.size() as usize) < 2 {
                return Err(
                    "Allocated memory should be more than two times your persistence_file size"
                        .to_string(),
                );
            }
        }
    }

    Ok(())
}
