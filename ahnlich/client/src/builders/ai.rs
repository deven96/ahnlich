use std::{collections::HashSet, num::NonZeroUsize};
use typed_builder::TypedBuilder;

use ahnlich_types::{
    ai::{AIModel, PreprocessAction},
    keyval::{StoreInput, StoreName, StoreValue},
    metadata::MetadataKey,
    predicate::PredicateCondition,
    similarity::{Algorithm, NonLinearAlgorithm},
};

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

    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct GetPredParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,
    pub condition: PredicateCondition,
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct GetSimNParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,
    pub search_input: StoreInput,
    pub condition: Option<PredicateCondition>,

    #[builder(default=NonZeroUsize::new(1).unwrap())]
    pub closest_n: NonZeroUsize,

    #[builder(default=Algorithm::CosineSimilarity)]
    pub algorithm: Algorithm,

    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct CreatePredIndexParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    //#[builder(default = HashSet::new())]
    pub predicates: HashSet<MetadataKey>,

    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct CreateNonLinearAlgorithmIndexParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    #[builder(default = HashSet::from_iter(&[NonLinearAlgorithm::KDTree]))]
    pub non_linear_indices: HashSet<NonLinearAlgorithm>,

    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct DropPredIndexParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    //#[builder(default = HashSet::new())]
    pub predicates: HashSet<MetadataKey>,

    #[builder(default = true)]
    pub error_if_not_exists: bool,

    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct SetParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    pub inputs: Vec<(StoreInput, StoreValue)>,

    #[builder(default = PreprocessAction::NoPreprocessing)]
    pub preprocess_action: PreprocessAction,

    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct DelKeyParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    pub key: StoreInput,
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct DropStoreParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    #[builder(default = true)]
    pub error_if_not_exists: bool,

    pub tracing_id: Option<String>,
}
