#![allow(dead_code)]
#![allow(clippy::size_of_ref)]
mod algorithm;
mod engine;
mod errors;
mod network;
mod storage;
use std::error::Error;

use clap::{ArgAction, Args, Parser, Subcommand};
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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Args, Debug)]
struct Run {
    #[arg(long, default_value_t = String::from("127.0.0.1"))]
    host: String,

    #[arg(long, default_value_t = 1369)]
    port: u16,

    /// Allows server to persist data to disk on occassion
    #[arg(long, default_value_t = false, action=ArgAction::SetTrue)]
    enable_persistence: bool,

    /// persistence location
    #[arg(long, requires_if("true", "enable_persistence"))]
    persist_location: Option<std::path::PathBuf>,

    /// persistence intervals in milliseconds
    #[arg(long, default_value_t = 1000 * 60 * 5)]
    persistence_intervals: u64,
}

#[derive(Args, Debug)]
struct Persistence {}

#[derive(Subcommand)]
enum Commands {
    /// Starts Anhlich server
    Run(Run),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Run(run) => {
            println!("{:?}", run);
            run_server().await?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
