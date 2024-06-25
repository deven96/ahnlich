#![allow(dead_code)]
#![allow(clippy::size_of_ref)]
use clap::Parser;
use std::error::Error;

mod algorithm;
pub mod cli;
mod client;
mod engine;
mod errors;
mod network;
mod persistence;
mod server;
mod storage;

#[cfg(test)]
mod fixtures;

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
