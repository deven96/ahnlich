use super::AIModel;
use super::AIStoreType;
use crate::bincode::BinCodeSerAndDeser;
use crate::db::{ConnectedClient, ServerInfo, StoreUpsert};
use crate::keyval::StoreInput;
use crate::keyval::StoreName;
use crate::keyval::StoreValue;
use crate::similarity::Similarity;
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
    StoreList(HashSet<AIStoreInfo>),
    InfoServer(ServerInfo),
    Set(StoreUpsert),
    // Always returned in order of the key request, however when GetPred is used, there is no key
    // request so the order can be mixed up
    Get(Vec<(StoreInput, StoreValue)>),
    GetSimN(Vec<(StoreInput, StoreValue, Similarity)>),
    // number of deleted entities
    Del(usize),
    // number of created indexes
    CreateIndex(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AIStoreInfo {
    pub name: StoreName,
    pub r#type: AIStoreType,
    pub model: AIModel,
    pub embedding_size: usize,
    pub size_in_bytes: usize,
}
