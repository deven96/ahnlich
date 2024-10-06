use super::{AIModel, PreprocessAction};
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
        store: StoreName,
        query_model: AIModel,
        index_model: AIModel,
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
        inputs: Vec<(StoreInput, StoreValue)>,
        preprocess_action: PreprocessAction,
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
    trace_id: Option<String>,
}

impl AIServerQuery {
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

    pub fn push(&mut self, entry: AIQuery) {
        self.queries.push(entry)
    }

    pub fn from_queries(queries: &[AIQuery]) -> Self {
        Self {
            queries: queries.to_vec(),
            trace_id: None,
        }
    }
}

impl BinCodeSerAndDeser for AIServerQuery {}

impl BinCodeSerAndDeserQuery for AIServerQuery {
    type Inner = Vec<AIQuery>;

    fn into_inner(self) -> Self::Inner {
        self.queries
    }
    fn get_traceparent(&self) -> Option<String> {
        self.trace_id.clone()
    }
}
