use crate::bincode::BinCodeSerAndDeser;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerResponse {
    // Unit variant for no action
    Unit,
    Pong,
    // List of connected clients. Potentially outdated at the point of read
    ClientList(HashSet<ConnectedClient>),
    // TODO: Define return types for queries, e.t.c
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ConnectedClient {
    pub address: String,
}

// ServerResult: Given that an array of queries are sent in, we expect that an array of responses
// be returned each being a potential error
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerResult {
    results: Vec<Result<ServerResponse, String>>,
}

impl BinCodeSerAndDeser<'_> for ServerResult {}

impl ServerResult {
    pub fn with_capacity(len: usize) -> Self {
        Self {
            results: Vec::with_capacity(len),
        }
    }

    pub fn push(&mut self, entry: Result<ServerResponse, String>) {
        self.results.push(entry)
    }
}
