use std::collections::HashSet;
use std::num::NonZeroUsize;
use typed_builder::TypedBuilder;

use ahnlich_types::{
    keyval::{StoreKey, StoreName, StoreValue},
    metadata::MetadataKey,
    predicate::PredicateCondition,
    similarity::{Algorithm, NonLinearAlgorithm},
};

#[derive(TypedBuilder)]
pub struct CreateStoreParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    #[builder(setter(into, transform = |n: usize| NonZeroUsize::new(n).unwrap()))]
    pub dimension: NonZeroUsize,

    #[builder(default = HashSet::new())]
    pub create_predicates: HashSet<MetadataKey>,

    #[builder(default = HashSet::new())]
    pub non_linear_indices: HashSet<NonLinearAlgorithm>,

    #[builder(default = true)]
    pub error_if_exists: bool,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct GetKeyParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,
    pub keys: Vec<StoreKey>,
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
    pub search_input: StoreKey,

    #[builder(setter(into, transform = |n: usize| NonZeroUsize::new(n).unwrap()),default=NonZeroUsize::new(1).unwrap())]
    pub closest_n: NonZeroUsize,

    #[builder(default=Algorithm::CosineSimilarity)]
    pub algorithm: Algorithm,

    #[builder(default = None)]
    pub condition: Option<PredicateCondition>,
    #[builder(default = None)]
    pub tracing_id: Option<String>,
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
pub struct DropNonLinearAlgorithmIndexParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    pub non_linear_indices: HashSet<NonLinearAlgorithm>,

    #[builder(default = true)]
    pub error_if_not_exists: bool,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct SetParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,
    pub inputs: Vec<(StoreKey, StoreValue)>,
    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct DelKeyParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    pub keys: Vec<StoreKey>,

    #[builder(default = None)]
    pub tracing_id: Option<String>,
}

#[derive(TypedBuilder)]
pub struct DelPredParams {
    #[builder(setter(into, transform = |s: String| StoreName(s)))]
    pub store: StoreName,

    pub condition: PredicateCondition,

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
