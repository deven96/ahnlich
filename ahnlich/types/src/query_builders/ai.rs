use std::{collections::HashSet, num::NonZeroUsize};

use crate::{
    ai::{AIModel, ExecutionProvider, PreprocessAction},
    keyval::{StoreInput, StoreName, StoreValue},
    metadata::MetadataKey,
    predicate::PredicateCondition,
    similarity::{Algorithm, NonLinearAlgorithm},
};
use typed_builder::TypedBuilder;

#[derive(TypedBuilder)]
pub struct CreateStoreParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    #[builder(default = AIModel::AllMiniLML6V2)]
    pub query_model: AIModel,

    #[builder(default = AIModel::AllMiniLML6V2)]
    pub index_model: AIModel,

    #[builder(default = HashSet::new())]
    pub predicates: HashSet<MetadataKey>,

    #[builder(default = HashSet::new())]
    pub non_linear_indices: HashSet<NonLinearAlgorithm>,

    #[builder(default = true)]
    pub error_if_exists: bool,

    #[builder(default = true)]
    pub store_original: bool,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct GetPredParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,
    pub condition: PredicateCondition,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct GetSimNParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,
    pub search_input: StoreInput,

    #[builder(default = None)]
    pub condition: Option<PredicateCondition>,

    #[builder(setter(into, transform = |n: usize| NonZeroUsize::new(n).unwrap()),default=NonZeroUsize::new(1).unwrap())]
    pub closest_n: NonZeroUsize,

    #[builder(default=Algorithm::CosineSimilarity)]
    pub algorithm: Algorithm,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
    #[builder(default = PreprocessAction::NoPreprocessing)]
    pub preprocess_action: PreprocessAction,

    #[builder(default=None)]
    pub execution_provider: Option<ExecutionProvider>,
}

#[derive(TypedBuilder)]
pub struct CreatePredIndexParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    pub predicates: HashSet<MetadataKey>,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct CreateNonLinearAlgorithmIndexParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    #[builder(default = HashSet::from_iter([NonLinearAlgorithm::KDTree]))]
    pub non_linear_indices: HashSet<NonLinearAlgorithm>,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct DropPredIndexParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    pub predicates: HashSet<MetadataKey>,

    #[builder(default = true)]
    pub error_if_not_exists: bool,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct SetParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    pub inputs: Vec<(StoreInput, StoreValue)>,

    #[builder(default = PreprocessAction::ModelPreprocessing)]
    pub preprocess_action: PreprocessAction,

    #[builder(default = None)]
    pub tracing_id: Option<String>,

    #[builder(default=None)]
    pub execution_provider: Option<ExecutionProvider>,
}

#[derive(TypedBuilder)]
pub struct DelKeyParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    pub key: StoreInput,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct DropStoreParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    #[builder(default = true)]
    pub error_if_not_exists: bool,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}
