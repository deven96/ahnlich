use std::fmt::Debug;

use super::config::cli::Agent;
use ahnlich_client_rs::{ai::AiClient, db::DbClient};
use ahnlich_types::ai::pipeline::{self as ai_pipeline};
use ahnlich_types::{
    ai::pipeline::AiServerResponse, db::pipeline::DbServerResponse, server_types::ServerType,
    shared::info::ErrorResponse,
};

use dsl::{ai::parse_ai_query, db::parse_db_query};

use crossterm::style::Stylize;

#[derive(Debug)]
pub enum AgentClient {
    AI(AiClient),
    DB(DbClient),
}

impl AgentClient {
    pub async fn create_client(
        agent: Agent,
        host: &str,
        port: Option<u16>,
    ) -> Result<Self, String> {
        let host = if !(host.starts_with("https://") || host.starts_with("http://")) {
            format!("http://{host}")
        } else {
            host.to_string()
        };
        let port = port.unwrap_or_else(|| match agent {
            Agent::AI => 1370,
            Agent::DB => 1369,
        });
        let addr = format!("{host}:{port}");
        match agent {
            Agent::AI => {
                let client = AiClient::new(addr).await.map_err(|err| err.to_string())?;
                Ok(Self::AI(client))
            }
            Agent::DB => {
                let client = DbClient::new(addr).await.map_err(|err| err.to_string())?;

                Ok(Self::DB(client))
            }
        }
    }

    /// Returns the commands for the agent pool in question
    pub fn commands(&self) -> &[&str] {
        match self {
            &Self::AI(_) => dsl::ai::COMMANDS,
            Self::DB(_) => dsl::db::COMMANDS,
        }
    }

    /// Checks if the connection to to a host and post is alive, also checks the cli is connected
    /// to the right server( ahnlich ai or db)
    pub async fn is_valid_connection(&self) -> Result<bool, String> {
        match &self {
            &Self::AI(client) => {
                let server_info = client
                    .info_server(None)
                    .await
                    .map_err(|err| err.to_string())?;

                let server_type =
                    ServerType::try_from(server_info.r#type).expect("Failed to get server type");
                match server_type {
                    ServerType::Ai => return Ok(true),
                    ServerType::Database => return Ok(false),
                }
            }
            Self::DB(client) => {
                let server_info = client
                    .info_server(None)
                    .await
                    .map_err(|err| err.to_string())?;

                let server_type =
                    ServerType::try_from(server_info.r#type).expect("Failed to get server type");

                match server_type {
                    ServerType::Ai => return Ok(false),
                    ServerType::Database => return Ok(true),
                }
            }
        }
    }

    pub async fn parse_queries(&self, input: &str) -> Result<Vec<String>, String> {
        match &self {
            &Self::AI(client) => {
                let queries = parse_ai_query(input).map_err(|err| err.to_string())?;

                let mut pipeline = client.pipeline(None);
                pipeline.set_queries(queries);

                let response = pipeline.exec().await.map_err(|err| err.to_string())?;

                Ok(render_ai_responses(response.responses))
            }
            Self::DB(client) => {
                let queries = parse_db_query(input).map_err(|err| err.to_string())?;

                let mut pipeline = client.pipeline(None);
                pipeline.set_queries(queries);

                let response = pipeline.exec().await.map_err(|err| err.to_string())?;

                Ok(render_db_responses(response.responses))
            }
        }
    }
}

impl std::fmt::Display for AgentClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AI(_) => f.write_str("AI"),
            &Self::DB(_) => f.write_str("DB"),
        }
    }
}

fn render_ai_responses(input: Vec<AiServerResponse>) -> Vec<String> {
    input
        .into_iter()
        .flat_map(|val| match val.response {
            Some(ai_pipeline::ai_server_response::Response::Error(ErrorResponse {
                code: _,
                message,
            })) => Some(format_error(message)),
            Some(success) => Some(format_success(format!("{:#?}", &success))),
            None => None,
        })
        .collect()
}

fn render_db_responses(input: Vec<DbServerResponse>) -> Vec<String> {
    input
        .into_iter()
        .flat_map(|val| match val.response {
            Some(ahnlich_types::db::pipeline::db_server_response::Response::Error(
                ErrorResponse { code: _, message },
            )) => Some(format_error(message)),
            Some(success) => Some(format_success(format!("{:#?}", &success))),
            None => None,
        })
        .collect()
}

fn format_success(input: String) -> String {
    format!("{}", input.green())
}

fn format_error(input: String) -> String {
    format!("{}", input.red())
}
