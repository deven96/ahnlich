use ahnlich_cli::{connect::AgentClient, term::Term};
use clap::Parser;
use std::io;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = ahnlich_cli::config::cli::Cli::parse();

    match cli.commands {
        ahnlich_cli::config::cli::Commands::Ahnlich(config) => {
            // In non-interactive mode, check for input first before connecting
            if config.no_interactive {
                use std::io::Read;
                let mut input_raw = String::new();
                io::stdin().read_to_string(&mut input_raw)?;
                let input = input_raw.trim();

                if input.is_empty() {
                    eprintln!("Error: No input provided");
                    std::process::exit(1);
                }

                // Now connect and execute
                let client = AgentClient::create_client(config.agent, &config.host, config.port)
                    .await
                    .map_err(io::Error::other)?;

                if !client
                    .is_valid_connection()
                    .await
                    .map_err(io::Error::other)?
                {
                    return Err(io::Error::other(format!(
                        "Connected Server is not a valid {client} Server"
                    )));
                }

                let term = Term::new(client);
                term.execute_non_interactive(input).await?;
            } else {
                let client = AgentClient::create_client(config.agent, &config.host, config.port)
                    .await
                    .map_err(io::Error::other)?;

                if !client
                    .is_valid_connection()
                    .await
                    .map_err(io::Error::other)?
                {
                    return Err(io::Error::other(format!(
                        "Connected Server is not a valid {client} Server"
                    )));
                }

                let term = Term::new(client);
                term.welcome_message()?;
                term.run().await?;
            }
        }
    }
    Ok(())
}
