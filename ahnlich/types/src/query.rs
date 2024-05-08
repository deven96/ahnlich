use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;

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
        dimension: NonZeroUsize,
        create_predicates: HashSet<MetadataKey>,
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
        predicates: HashSet<MetadataKey>,
    },
    DropIndexPred {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
    },
    Set {
        store: StoreName,
        inputs: HashMap<StoreKey, StoreValue>,
    },
    DelKey {
        store: StoreName,
        key: Vec<StoreKey>,
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
