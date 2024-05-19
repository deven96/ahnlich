#![allow(dead_code)]
#![allow(clippy::size_of_ref)]
mod algorithm;
pub mod cli;
mod engine;
mod errors;
mod network;
mod storage;
use clap::{ArgAction, Args};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

#[derive(Args, Debug, Clone)]
pub struct ServerConfig {
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    pub host: String,

    #[arg(long, default_value_t = 1369)]
    pub port: u16,

    /// Allows server to persist data to disk on occassion
    #[arg(long, default_value_t = false, action=ArgAction::SetTrue)]
    pub(crate) enable_persistence: bool,

    /// persistence location
    #[arg(long, requires_if("true", "enable_persistence"))]
    pub(crate) persist_location: Option<std::path::PathBuf>,

    /// persistence intervals in milliseconds
    #[arg(long, default_value_t = 1000 * 60 * 5)]
    pub(crate) persistence_intervals: u64,
}

impl ServerConfig {
    fn new() -> Self {
        Self {
            host: String::from("127.0.0.1"),
            port: 1396,
            enable_persistence: false,
            persist_location: None,
            persistence_intervals: 1000 * 60 * 5,
        }
    }
}
impl Default for ServerConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run_server(config: ServerConfig) -> std::io::Result<()> {
    let listener =
        tokio::net::TcpListener::bind(format!("{}:{}", &config.host, &config.port)).await?;

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

async fn process_stream(stream: tokio::net::TcpStream) -> Result<(), tokio::io::Error> {
    stream.readable().await?;
    let mut reader = BufReader::new(stream);
    loop {
        let mut message = String::new();
        let _ = reader.read_line(&mut message).await?;
        reader.get_mut().write_all(message.as_bytes()).await?;
        message.clear();
    }
}

#[cfg(test)]
mod tests {
    // Import the fixtures for use in tests
    pub use super::fixtures::*;
}

#[cfg(test)]
mod fixtures;
