use serde::{Deserialize, Serialize};

pub type ClientId = String;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientWriteRequest<C> {
    pub client_id: ClientId,
    pub request_id: u64,
    pub command: C,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientWriteResponse<R> {
    pub response: R,
}

impl<R: Default> Default for ClientWriteResponse<R> {
    fn default() -> Self {
        Self {
            response: R::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbCommand {
    CreateStore(Vec<u8>),
    CreatePredIndex(Vec<u8>),
    CreateNonLinearAlgorithmIndex(Vec<u8>),
    Set(Vec<u8>),
    DropPredIndex(Vec<u8>),
    DropNonLinearAlgorithmIndex(Vec<u8>),
    DelKey(Vec<u8>),
    DelPred(Vec<u8>),
    DropStore(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiCommand {
    CreateStore(Vec<u8>),
    CreatePredIndex(Vec<u8>),
    CreateNonLinearAlgorithmIndex(Vec<u8>),
    Set(Vec<u8>),
    DropPredIndex(Vec<u8>),
    DropNonLinearAlgorithmIndex(Vec<u8>),
    DelKey(Vec<u8>),
    DelPred(Vec<u8>),
    DropStore(Vec<u8>),
    PurgeStores(Vec<u8>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum DbResponseKind {
    Unit,
    CreateIndex,
    Set,
    Del,
}

impl Default for DbResponseKind {
    fn default() -> Self {
        Self::Unit
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbResponse {
    pub kind: DbResponseKind,
    pub payload: Vec<u8>,
}

impl Default for DbResponse {
    fn default() -> Self {
        Self {
            kind: DbResponseKind::Unit,
            payload: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiResponseKind {
    Unit,
    CreateIndex,
    Set,
    Del,
}

impl Default for AiResponseKind {
    fn default() -> Self {
        Self::Unit
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub kind: AiResponseKind,
    pub payload: Vec<u8>,
}

impl Default for AiResponse {
    fn default() -> Self {
        Self {
            kind: AiResponseKind::Unit,
            payload: Vec::new(),
        }
    }
}
