use ahnlich_cli::{connect::AgentPool, term::Term};
use clap::Parser;
use std::io;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let cli = ahnlich_cli::config::cli::Cli::parse();

    match cli.commands {
        ahnlich_cli::config::cli::Commands::Ahnlich(config) => {
            let agent_pool = AgentPool::create_pool(config.agent, &config.host, config.port)
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

            if !agent_pool
                .is_valid_connection()
                .await
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?
            {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    format!("Connected Server is not a valid {} Server", agent_pool),
                ));
            }
            let term = Term::new(agent_pool);
            term.welcome_message()?;
            term.run().await?;
        }
    }
    Ok(())
}
