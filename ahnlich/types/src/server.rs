use crate::bincode::BinCodeSerAndDeser;
use crate::keyval::StoreKey;
use crate::keyval::StoreName;
use crate::keyval::StoreValue;
use crate::similarity::Similarity;
use crate::version::Version;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;
use std::hash::Hash;
use std::hash::Hasher;
use std::time::SystemTime;

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
    // TODO: Define return types for queries, e.t.c
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServerType {
    Database,
    AI,
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialOrd, Ord)]
pub struct ConnectedClient {
    pub address: String,
    // NOTE: We are using System specific time so the time marked by clients cannot be relied on to
    // be monotonic and the size depends on operating system
    pub time_connected: SystemTime,
}

// NOTE: ConnectedClient should be unique purely by address assuming we are not doing any TCP magic
// to allow port reuse
impl Hash for ConnectedClient {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.address.hash(state)
    }
}

impl PartialEq for ConnectedClient {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&other.address)
    }
}

// ServerResult: Given that an array of queries are sent in, we expect that an array of responses
// be returned each being a potential error
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerResult {
    results: Vec<Result<ServerResponse, String>>,
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

    pub fn from_error(err: String) -> Self {
        Self {
            results: vec![Err(err)],
        }
    }

    pub fn push(&mut self, entry: Result<ServerResponse, String>) {
        self.results.push(entry)
    }
}
