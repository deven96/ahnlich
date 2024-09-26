use super::config::cli::Agent;
use ahnlich_client_rs::{
    ai::{AIClient, AIConnManager},
    db::{DbClient, DbConnManager},
    prelude::{AIServerResponse, ServerResponse},
};
use ahnlich_types::ServerType;
use deadpool::managed::Pool;

#[derive(Debug)]
pub enum AgentPool {
    AI(Pool<AIConnManager>),
    DB(Pool<DbConnManager>),
}

impl AgentPool {
    pub fn create_pool(agent: Agent, host: &str, port: u16) -> Result<Self, String> {
        match agent {
            Agent::AI => {
                let pool = Pool::builder(AIConnManager::new(host.to_owned(), port))
                    .build()
                    .map_err(|err| err.to_string())?;
                Ok(Self::AI(pool))
            }
            Agent::DB => {
                let pool = Pool::builder(DbConnManager::new(host.to_owned(), port))
                    .build()
                    .map_err(|err| err.to_string())?;

                Ok(Self::DB(pool))
            }
        }
    }

    /// Checks if the connection to to a host and post is alive, also checks the cli is connected
    /// to the right server( ahnlich ai or db)
    pub async fn is_valid_connection(&self) -> Result<bool, String> {
        match self {
            AgentPool::AI(pool) => {
                let client = AIClient::new_with_pool(pool.clone());

                let result = client
                    .info_server(None)
                    .await
                    .map_err(|err| err.to_string())?;
                if let AIServerResponse::InfoServer(server_details) = result {
                    match server_details.r#type {
                        ServerType::AI => return Ok(true),
                        ServerType::Database => return Ok(false),
                    }
                }
            }
            AgentPool::DB(pool) => {
                let client = DbClient::new_with_pool(pool.clone());

                let result = client
                    .info_server(None)
                    .await
                    .map_err(|err| err.to_string())?;
                if let ServerResponse::InfoServer(server_details) = result {
                    match server_details.r#type {
                        ServerType::AI => return Ok(true),
                        ServerType::Database => return Ok(false),
                    }
                }
            }
        }

        Ok(false)
    }
}

impl std::fmt::Display for AgentPool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentPool::AI(_) => f.write_str("AI"),
            AgentPool::DB(_) => f.write_str("DB"),
        }
    }
}
