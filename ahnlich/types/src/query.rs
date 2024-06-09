use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::bincode::BinCodeSerAndDeser;
use crate::keyval::{StoreKey, StoreName, StoreValue};
use crate::metadata::MetadataKey;
use crate::predicate::PredicateCondition;
use crate::similarity::Algorithm;
use serde::{Deserialize, Serialize};

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
    CreateIndex {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
    },
    DropIndex {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
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
fn serialize_non_zero_usize<S>(value: &NonZeroUsize, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_u64(value.get() as u64)
}

fn deserialize_non_zero_usize<'de, D>(deserializer: D) -> Result<NonZeroUsize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = u64::deserialize(deserializer)?;

    NonZeroUsize::new(value as usize)
        .ok_or_else(|| serde::de::Error::custom("NonZeroUsize value must be non-zero"))
}

fn default_nonzerousize() -> NonZeroUsize {
    return NonZeroUsize::new(1).expect("Failed to create default non-zero value");
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
