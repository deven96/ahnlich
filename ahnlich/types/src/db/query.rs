use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::bincode::{BinCodeSerAndDeser, BinCodeSerAndDeserQuery};
use crate::keyval::{StoreKey, StoreName, StoreValue};
use crate::metadata::MetadataKey;
use crate::predicate::PredicateCondition;
use crate::similarity::Algorithm;
use crate::similarity::NonLinearAlgorithm;
use serde::{Deserialize, Serialize};

/// All possible queries for the server to respond to
///
///
/// Vec of queries are to be sent by clients in bincode
/// - Length encoding must use fixed int and not var int
/// - Endianess must be Little Endian.
/// - First 8 bytes must contain length of the entire vec of queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Query {
    CreateStore {
        store: StoreName,
        dimension: NonZeroUsize,
        create_predicates: HashSet<MetadataKey>,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
        error_if_exists: bool,
    },
    GetKey {
        store: StoreName,
        keys: Vec<StoreKey>,
    },
    GetPred {
        store: StoreName,
        condition: PredicateCondition,
    },
    GetSimN {
        store: StoreName,
        search_input: StoreKey,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
        condition: Option<PredicateCondition>,
    },
    CreatePredIndex {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
    },
    CreateNonLinearAlgorithmIndex {
        store: StoreName,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
    },
    DropPredIndex {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
        error_if_not_exists: bool,
    },
    DropNonLinearAlgorithmIndex {
        store: StoreName,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
        error_if_not_exists: bool,
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
        error_if_not_exists: bool,
    },
    InfoServer,
    ListStores,
    ListClients,
    Ping,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ServerQuery {
    queries: Vec<Query>,
    trace_id: Option<String>,
}

impl ServerQuery {
    pub fn with_capacity(len: usize) -> Self {
        Self {
            queries: Vec::with_capacity(len),
            trace_id: None,
        }
    }
    pub fn with_capacity_and_tracing_id(len: usize, trace_id: Option<String>) -> Self {
        Self {
            queries: Vec::with_capacity(len),
            trace_id,
        }
    }

    pub fn push(&mut self, entry: Query) {
        self.queries.push(entry)
    }

    pub fn from_queries(queries: &[Query]) -> Self {
        Self {
            queries: queries.to_vec(),
            trace_id: None,
        }
    }
}

impl BinCodeSerAndDeser for ServerQuery {}

impl BinCodeSerAndDeserQuery for ServerQuery {
    type Inner = Vec<Query>;

    fn into_inner(self) -> Vec<Query> {
        self.queries
    }
    // TODO: might change default value
    fn get_traceparent(&self) -> Option<String> {
        self.trace_id.clone()
    }
}
