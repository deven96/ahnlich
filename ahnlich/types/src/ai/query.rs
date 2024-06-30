use super::{AIModel, AIStoreType};
use crate::keyval::{StoreInput, StoreName};
use crate::metadata::MetadataKey;
use crate::predicate::PredicateCondition;
use crate::similarity::Algorithm;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AiQuery {
    CreateStore {
        r#type: AIStoreType,
        store: StoreName,
        model: AIModel,
        predicates: HashSet<MetadataKey>,
    },
    GetPred {
        store: StoreName,
        condition: PredicateCondition,
    },
    GetSimN {
        store: StoreName,
        search_input: StoreInput,
        condition: Option<PredicateCondition>,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
    },
    CreateIndex {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
    },
    DropIndexPred {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
        error_if_not_exists: bool,
    },
    Set {
        store: StoreName,
        inputs: Vec<StoreInput>,
    },
    DelKey {
        store: StoreName,
        key: StoreInput,
    },
    DropStore {
        store: StoreName,
        error_if_not_exists: bool,
    },
    InfoServer,
    ListStores,
    Ping,
}
