use bincode::config::DefaultOptions;
use bincode::config::Options;
use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::keyval::{StoreKey, StoreName, StoreValue};
use crate::metadata::MetadataKey;
use crate::predicate::PredicateCondition;
use crate::similarity::Algorithm;
use serde::Deserialize;
use serde::Serialize;

pub const LENGTH_HEADER_SIZE: usize = 8;
/// All possible queries for the server to respond to
///
///
/// Vec of queries are to be sent by clients in bincode
/// - Length encoding must use fixed int and not var int
/// - Endianess must be Big Endian.
/// - First 8 bytes must contain length of the entire vec of queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Query {
    Create {
        store: StoreName,
        dimension: NonZeroUsize,
        create_predicates: HashSet<MetadataKey>,
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

#[derive(Debug)]
pub struct SerializedQuery {
    data: Vec<u8>,
}

impl SerializedQuery {
    /// Sends an array of queries with their length encoding first specified in 8 bytes
    /// Enforces default configuration for bincode serialization
    pub fn from_queries(queries: &[Query]) -> Result<Self, bincode::Error> {
        // TODO: Make shareable across serialize and deserialize as perhaps a config stored on heap
        // initialized via once_cell. Currently cannot be done as Limit must be specified for the
        // Options trait and the Limit enum is not made public outside bincode
        let config = DefaultOptions::new()
            .with_fixint_encoding()
            .with_big_endian();
        let serialized_data = config.serialize(queries)?;
        let data_length = serialized_data.len() as u64;

        let mut buffer = Vec::with_capacity(LENGTH_HEADER_SIZE + serialized_data.len());
        buffer.extend(&data_length.to_be_bytes());
        buffer.extend(&serialized_data);
        Ok(SerializedQuery { data: buffer })
    }

    pub fn data(&self) -> &[u8] {
        &self.data
    }

    pub fn length(&self) -> usize {
        let mut length_buf = [0u8; LENGTH_HEADER_SIZE];
        length_buf.copy_from_slice(&self.data[0..LENGTH_HEADER_SIZE]);
        let length = u64::from_be_bytes(length_buf);
        length as usize
    }
}

/// Receives an array of bytes with their length encoding first specified in 8 bytes
/// Uses default configuration for bincode deserialization to attempt to retrieve queries
pub fn deserialize_queries(queries: &[u8]) -> Result<Vec<Query>, bincode::Error> {
    let config = DefaultOptions::new()
        .with_fixint_encoding()
        .with_big_endian();
    let deserialized_data = config.deserialize(queries)?;
    Ok(deserialized_data)
}
