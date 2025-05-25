use ahnlich_cli::{connect::AgentClient, term::Term};
use clap::Parser;
use std::io;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = ahnlich_cli::config::cli::Cli::parse();

    match cli.commands {
        ahnlich_cli::config::cli::Commands::Ahnlich(config) => {
            let client = AgentClient::create_client(config.agent, &config.host, config.port)
                .await
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

            if !client
                .is_valid_connection()
                .await
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
            {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Connected Server is not a valid {} Server", client),
                ));
            }
            let term = Term::new(client);
            term.welcome_message()?;
            term.run().await?;
        }
    }
    Ok(())
}
