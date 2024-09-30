use super::config::cli::Agent;
use ahnlich_client_rs::{
    ai::{AIClient, AIConnManager, AIPipeline},
    db::{DbClient, DbConnManager, DbPipeline},
    prelude::{AIServerResponse, ServerResponse},
};
use ahnlich_types::{ai::AIServerQuery, db::ServerDBQuery, ServerType};
use deadpool::managed::Pool;
use dsl::{ai::parse_ai_query, db::parse_db_query};

use crossterm::style::Stylize;
use serde::Serialize;

#[derive(Debug)]
pub enum AgentPool {
    AI(Pool<AIConnManager>),
    DB(Pool<DbConnManager>),
}

impl AgentPool {
    pub fn create_pool(agent: Agent, host: &str, port: Option<u16>) -> Result<Self, String> {
        match agent {
            Agent::AI => {
                let pool = Pool::builder(AIConnManager::new(host.to_owned(), port.unwrap_or(1370)))
                    .build()
                    .map_err(|err| err.to_string())?;
                Ok(Self::AI(pool))
            }
            Agent::DB => {
                let pool = Pool::builder(DbConnManager::new(host.to_owned(), port.unwrap_or(1369)))
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
                        ServerType::AI => return Ok(false),
                        ServerType::Database => return Ok(true),
                    }
                }
            }
        }

        Ok(false)
    }

    pub async fn parse_queries(&self, input: &str) -> Result<Vec<String>, String> {
        match self {
            AgentPool::AI(pool) => {
                let queries = parse_ai_query(input).map_err(|err| err.to_string())?;

                let server_query = AIServerQuery::from_queries(&queries);

                let conn = pool
                    .get()
                    .await
                    .map_err(|err| format!("Could not get ai client connection {err}"))?;

                let pipeline = AIPipeline::new_from_queries_and_conn(server_query, conn);

                let response = pipeline.exec().await.map_err(|err| err.to_string())?;

                Ok(render(response.into_inner()))
            }
            AgentPool::DB(pool) => {
                let queries = parse_db_query(input).map_err(|err| err.to_string())?;

                let server_query = ServerDBQuery::from_queries(&queries);

                let conn = pool
                    .get()
                    .await
                    .map_err(|err| format!("Could not get db client connection {err}"))?;

                let pipeline = DbPipeline::new_from_queries_and_conn(server_query, conn);

                let response = pipeline.exec().await.map_err(|err| err.to_string())?;

                Ok(render(response.into_inner()))
            }
        }
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

fn render(input: Vec<Result<impl Serialize, String>>) -> Vec<String> {
    input
        .into_iter()
        .map(|val| match val {
            Ok(success) => format_success(
                serde_json::to_string_pretty(&success)
                    .map_err(|err| err.to_string())
                    .expect("Failed to parse success response to json"),
            ),
            Err(err) => format_error(err),
        })
        .collect()
}

fn format_success(input: String) -> String {
    format!("{}", input.green())
}

fn format_error(input: String) -> String {
    format!("{}", input.red())
}
