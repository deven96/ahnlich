#![allow(dead_code)]
#![allow(clippy::size_of_ref)]
mod algorithm;
mod cli;
mod engine;
mod errors;
mod network;
mod storage;
use std::error::Error;

use crate::cli::{Cli, Commands, ServerConfig};
use clap::Parser;
use env_logger::Env;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn run_server(config: &ServerConfig) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("{}:{}", &config.host, config.port)).await?;

    loop {
        let (stream, connect_addr) = listener.accept().await?;
        log::info!("Connecting to {}", connect_addr);
        tokio::spawn(async move {
            if let Err(e) = process_stream(stream).await {
                log::error!("Error handling connection: {}", e)
            };
        });
    }
}

async fn process_stream(stream: TcpStream) -> Result<(), tokio::io::Error> {
    stream.readable().await?;
    let mut reader = BufReader::new(stream);
    loop {
        let mut message = String::new();
        let _ = reader.read_line(&mut message).await?;
        reader.get_mut().write_all(message.as_bytes()).await?;
        message.clear();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let cli = Cli::parse();
    match &cli.command {
        Commands::Run(config) => {
            run_server(config).await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
