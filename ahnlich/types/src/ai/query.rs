use super::{AIModel, AIStoreType};
use crate::keyval::{StoreInput, StoreName, StoreValue};
use crate::metadata::MetadataKey;
use crate::predicate::PredicateCondition;
use crate::similarity::{Algorithm, NonLinearAlgorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::num::NonZeroUsize;

use crate::bincode::{BinCodeSerAndDeser, BinCodeSerAndDeserQuery};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AIQuery {
    CreateStore {
        r#type: AIStoreType,
        store: StoreName,
        model: AIModel,
        predicates: HashSet<MetadataKey>,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
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
    CreatePredIndex {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
    },
    DropPredIndex {
        store: StoreName,
        predicates: HashSet<MetadataKey>,
        error_if_not_exists: bool,
    },
    Set {
        store: StoreName,
        inputs: Vec<(StoreInput, StoreValue)>,
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
    PurgeStores,
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
}

impl BinCodeSerAndDeser for AIServerQuery {}

impl BinCodeSerAndDeserQuery for AIServerQuery {
    type Inner = Vec<AIQuery>;

    fn into_inner(self) -> Self::Inner {
        self.queries
    }
}
