use crate::bincode::{BinCodeSerAndDeser, BinCodeSerAndDeserResponse};
use crate::client::ConnectedClient;
use crate::keyval::StoreKey;
use crate::keyval::StoreName;
use crate::keyval::StoreValue;
use crate::similarity::Similarity;
use crate::version::Version;
use crate::ServerType;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ServerResponse {
    // Unit variant for no action
    Unit,
    Pong,
    // List of connected clients. Potentially outdated at the point of read
    ClientList(HashSet<ConnectedClient>),
    StoreList(HashSet<StoreInfo>),
    InfoServer(ServerInfo),
    Set(StoreUpsert),
    // Always returned in order of the key request, however when GetPred is used, there is no key
    // request so the order can be mixed up
    Get(Vec<(StoreKey, StoreValue)>),
    GetSimN(Vec<(StoreKey, StoreValue, Similarity)>),
    // number of deleted entities
    Del(usize),
    // number of created indexes
    CreateIndex(usize),
}

/// StoreUpsert shows how many entries were inserted and updated during a store add call
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreUpsert {
    pub inserted: usize,
    pub updated: usize,
}

impl StoreUpsert {
    pub fn modified(&self) -> bool {
        self.inserted + self.updated > 0
    }
}

/// StoreInfo just shows store name, size and length
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StoreInfo {
    pub name: StoreName,
    pub len: usize,
    pub size_in_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialOrd, Ord)]
pub struct ServerInfo {
    pub address: String,
    pub version: Version,
    pub r#type: ServerType,
    pub limit: usize,
    pub remaining: usize,
}

/// ignore `remaining` field during comparison for server info as a server might allocate memory
impl PartialEq for ServerInfo {
    fn eq(&self, other: &Self) -> bool {
        self.version.eq(&other.version)
            && self.r#type.eq(&other.r#type)
            && self.limit.eq(&other.limit)
    }
}

pub type ServerResultInner = Vec<Result<ServerResponse, String>>;
// ServerResult: Given that an array of queries are sent in, we expect that an array of responses
// be returned each being a potential error
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerResult {
    results: ServerResultInner,
}

impl BinCodeSerAndDeser for ServerResult {}

impl ServerResult {
    pub fn with_capacity(len: usize) -> Self {
        Self {
            results: Vec::with_capacity(len),
        }
    }

    pub fn pop(mut self) -> Option<Result<ServerResponse, String>> {
        self.results.pop()
    }

    pub fn push(&mut self, entry: Result<ServerResponse, String>) {
        self.results.push(entry)
    }
}

impl BinCodeSerAndDeserResponse for ServerResult {
    fn from_error(err: String) -> Self {
        Self {
            results: vec![Err(err)],
        }
    }
}
