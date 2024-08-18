use clap::Parser;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = ahnlich_ai_proxy::cli::Cli::parse();
    match cli.command {
        ahnlich_ai_proxy::cli::Commands::Start(config) => {
            let server = ahnlich_ai_proxy::server::handler::AIProxyServer::new(config).await?;
            server.start().await?;
        }
        ahnlich_ai_proxy::cli::Commands::SupportedModels(config) => {
            println!("\nDisplaying Supported Models \n");
            if config.names.len() > 0 {
                println!("{}", config.list_supported_models_verbose())
            } else {
                println!("{}", config.list_supported_models())
            }
        }
    }
    Ok(())
}
