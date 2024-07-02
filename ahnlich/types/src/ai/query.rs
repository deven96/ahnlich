use super::{AIModel, AIStoreType};
use crate::keyval::{StoreInput, StoreName};
use crate::metadata::MetadataKey;
use crate::predicate::PredicateCondition;
use crate::similarity::Algorithm;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::bincode::BinCodeSerAndDeser;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AIQuery {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AIServerQuery {
    queries: Vec<AIQuery>,
}

impl AIServerQuery {
    pub fn with_capacity(len: usize) -> Self {
        Self {
            queries: Vec::with_capacity(len),
        }
    }

    pub fn push(&mut self, entry: AIQuery) {
        self.queries.push(entry)
    }

    pub fn from_queries(queries: &[AIQuery]) -> Self {
        Self {
            queries: queries.to_vec(),
        }
    }

    pub fn into_inner(self) -> Vec<AIQuery> {
        self.queries
    }
}

impl BinCodeSerAndDeser for AIServerQuery {}
