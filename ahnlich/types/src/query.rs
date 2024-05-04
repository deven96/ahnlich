use std::num::NonZeroU32;

use crate::keyval::{SearchInput, StoreKey, StoreName, StoreValue};
use crate::metadata::MetadataKey;
use crate::predicate::PredicateCondition;
use crate::similarity::Algorithm;

/// All possible queries for the server to respond to
#[derive(Debug, Clone)]
pub enum Query {
    Connect,
    Disconnect,
    Create {
        store: StoreName,
        dimension: NonZeroU32,
        create_predicates: Vec<MetadataKey>,
    },
    GetKey {
        key: StoreKey,
    },
    GetPred {
        store: StoreName,
        condition: PredicateCondition,
    },
    GetSimN {
        store: StoreName,
        closest_n: u32,
        input: SearchInput,
        algorithm: Algorithm,
        condition: Option<PredicateCondition>,
    },
    IndexPred {
        store: StoreName,
        predicates: Vec<MetadataKey>,
    },
    DropIndexPred {
        store: StoreName,
        predicates: Vec<MetadataKey>,
    },
    Set {
        store: StoreName,
        inputs: Vec<(StoreKey, StoreValue)>,
    },
    DelKey {
        store: StoreName,
        key: StoreKey,
    },
    DelPred {
        store: StoreName,
        condition: PredicateCondition,
    },
    DropStore {
        store: StoreName,
    },
    ShutdownServer {
        reason: Option<String>,
    },
    InfoServer,
    ListStores,
    ListClients,
    Close,
}
