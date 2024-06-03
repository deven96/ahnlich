use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::bincode::BinCodeSerAndDeser;
use crate::keyval::{StoreKey, StoreName, StoreValue};
use crate::metadata::MetadataKey;
use crate::predicate::PredicateCondition;
use crate::similarity::Algorithm;
use serde::Deserialize;
use serde::Serialize;

/// All possible queries for the server to respond to
///
///
/// Vec of queries are to be sent by clients in bincode
/// - Length encoding must use fixed int and not var int
/// - Endianess must be Big Endian.
/// - First 8 bytes must contain length of the entire vec of queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Query {
    CreateStore {
        store: StoreName,
        dimension: NonZeroUsize,
        create_predicates: HashSet<MetadataKey>,
        error_if_exists: bool,
    },
    GetKey {
        keys: Vec<StoreKey>,
    },
    GetPred {
        store: StoreName,
        condition: PredicateCondition,
    },
    GetSimN {
        store: StoreName,
        closest_n: NonZeroUsize,
        input: StoreKey,
        algorithm: Algorithm,
        condition: Option<PredicateCondition>,
    },
    ReIndex {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
    },
    DropIndexPred {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
    },
    Set {
        store: StoreName,
        inputs: Vec<(StoreKey, StoreValue)>,
    },
    DelKey {
        store: StoreName,
        keys: Vec<StoreKey>,
    },
    DelPred {
        store: StoreName,
        condition: PredicateCondition,
    },
    DropStore {
        store: StoreName,
    },
    InfoServer,
    ListStores,
    ListClients,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerQuery {
    queries: Vec<Query>,
}

impl ServerQuery {
    pub fn with_capacity(len: usize) -> Self {
        Self {
            queries: Vec::with_capacity(len),
        }
    }

    pub fn push(&mut self, entry: Query) {
        self.queries.push(entry)
    }

    pub fn from_queries(queries: &[Query]) -> Self {
        Self {
            queries: queries.to_vec(),
        }
    }

    pub fn into_inner(self) -> Vec<Query> {
        self.queries
    }
}

impl BinCodeSerAndDeser<'_> for ServerQuery {}
