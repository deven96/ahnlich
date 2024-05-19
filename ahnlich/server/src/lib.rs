#![allow(dead_code)]
#![allow(clippy::size_of_ref)]
mod algorithm;
pub mod cli;
mod engine;
mod errors;
mod network;
mod storage;
pub use crate::cli::ServerConfig;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

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
