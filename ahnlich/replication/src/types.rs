use serde::{Deserialize, Serialize};

/// Application-level write envelope wrapping a single command. Kept as a
/// distinct type (rather than passing `C` directly) so that any future
/// metadata: at-most-once idempotency tags, tracing context, originating
/// client info, can be added in one place without rippling through every
/// `Raft::client_write` call site. No metadata fields exist today; add them
/// only with a concrete generator and consumer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientWriteRequest<C> {
    pub command: C,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientWriteResponse<R> {
    pub response: R,
}

/// DB Raft commands. Each variant carries the protobuf-encoded
/// [`ahnlich_types::db::query::*`] payload (kept as opaque bytes here to avoid
/// pulling the public type crate into this internal one). The DB state
/// machine decodes the payload and dispatches to a shared mutation function
/// in Milestone 2.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DbCommand {
    CreateStore(Vec<u8>),
    CreatePredIndex(Vec<u8>),
    CreateNonLinearAlgorithmIndex(Vec<u8>),
    Set(Vec<u8>),
    DelKey(Vec<u8>),
    DelPred(Vec<u8>),
    DropPredIndex(Vec<u8>),
    DropNonLinearAlgorithmIndex(Vec<u8>),
    DropStore(Vec<u8>),
    DropSchema(Vec<u8>),
}

/// AI Raft commands. Only operations that mutate AI-local state
/// (`AIStoreHandler`) are replicated here; everything else delegates to the
/// DB cluster via `db_client` and is replicated by the DB Raft cluster.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiCommand {
    CreateStore(Vec<u8>),
    DropStore(Vec<u8>),
    PurgeStores(Vec<u8>),
}

/// Successful response from applying a DB Raft command. `Unit` covers
/// mutations whose protocol response is empty; `Bytes` carries an encoded raw
/// operation result for mutations that return a value. The gRPC handler is
/// responsible for decoding those bytes into the expected operation result
/// type, then shaping them into public response messages.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum DbResponse {
    #[default]
    Unit,
    Bytes(Vec<u8>),
}

/// Successful response from applying an AI Raft command. Same shape as
/// [`DbResponse`].
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum AiResponse {
    #[default]
    Unit,
    Bytes(Vec<u8>),
}
