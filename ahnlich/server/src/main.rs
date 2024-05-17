#![allow(dead_code)]
#![allow(clippy::size_of_ref)]
mod algorithm;
mod engine;
mod errors;
mod network;
mod storage;
use std::error::Error;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

async fn run_server() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:1369").await?;

    loop {
        let (stream, connect_addr) = listener.accept().await?;
        println!("Connecting to {}", connect_addr);
        tokio::spawn(async move {
            if let Err(e) = process_stream(stream).await {
                eprintln!("Error handling connection: {}", e)
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
    run_server().await?;
    Ok(())
}

#[cfg(test)]
mod tests;
