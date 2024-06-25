#![allow(clippy::size_of_ref)]
use clap::Parser;
use std::error::Error;

mod algorithm;
pub mod cli;
mod engine;
mod errors;
mod network;
mod server;
mod storage;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = cli::Cli::parse();
    match &cli.command {
        cli::Commands::Run(config) => {
            let server = server::handler::Server::new(config).await?;
            server.start().await?;
        }
    }
    Ok(())
}
