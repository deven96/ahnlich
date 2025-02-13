// This file is @generated by prost-build.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateStore {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(enumeration = "super::models::AiModel", tag = "2")]
    pub query_model: i32,
    #[prost(enumeration = "super::models::AiModel", tag = "3")]
    pub index_model: i32,
    /// Validation will enforce uniqueness.
    #[prost(string, repeated, tag = "4")]
    pub predicates: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(
        enumeration = "super::super::algorithm::nonlinear::NonLinearAlgorithm",
        repeated,
        tag = "5"
    )]
    pub non_linear_indices: ::prost::alloc::vec::Vec<i32>,
    #[prost(bool, tag = "6")]
    pub error_if_exists: bool,
    #[prost(bool, tag = "7")]
    pub store_original: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetPred {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub condition: ::core::option::Option<super::super::predicates::PredicateCondition>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSimN {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub search_input: ::core::option::Option<super::super::keyval::StoreInput>,
    /// Optional: Use `oneof` if you want explicit nullability.
    #[prost(message, optional, tag = "3")]
    pub condition: ::core::option::Option<super::super::predicates::PredicateCondition>,
    /// NonZeroUsize isn't directly supported, enforce via validation.
    #[prost(uint32, tag = "4")]
    pub closest_n: u32,
    #[prost(
        enumeration = "super::super::algorithm::algorithms::Algorithm",
        tag = "5"
    )]
    pub algorithm: i32,
    #[prost(enumeration = "super::preprocess::PreprocessAction", tag = "6")]
    pub preprocess_action: i32,
    /// Optional: Again, use `oneof` if explicit null is needed.
    #[prost(
        enumeration = "super::execution_provider::ExecutionProvider",
        optional,
        tag = "7"
    )]
    pub execution_provider: ::core::option::Option<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreatePredIndex {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub predicates: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateNonLinearAlgorithmIndex {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(
        enumeration = "super::super::algorithm::nonlinear::NonLinearAlgorithm",
        repeated,
        tag = "2"
    )]
    pub non_linear_indices: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropPredIndex {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(string, repeated, tag = "2")]
    pub predicates: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(bool, tag = "3")]
    pub error_if_not_exists: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropNonLinearAlgorithmIndex {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(
        enumeration = "super::super::algorithm::nonlinear::NonLinearAlgorithm",
        repeated,
        tag = "2"
    )]
    pub non_linear_indices: ::prost::alloc::vec::Vec<i32>,
    #[prost(bool, tag = "3")]
    pub error_if_not_exists: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelKey {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub key: ::core::option::Option<super::super::keyval::StoreInput>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DropStore {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(bool, tag = "2")]
    pub error_if_not_exists: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetKey {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    /// StoreKey is imported from keyval.proto.
    #[prost(message, repeated, tag = "2")]
    pub keys: ::prost::alloc::vec::Vec<super::super::keyval::StoreInput>,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct InfoServer {}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct ListClients {}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct ListStores {}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct PurgeStores {}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct Ping {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Set {
    #[prost(string, tag = "1")]
    pub store: ::prost::alloc::string::String,
    #[prost(message, repeated, tag = "2")]
    pub inputs: ::prost::alloc::vec::Vec<StoreEntry>,
    #[prost(enumeration = "super::preprocess::PreprocessAction", tag = "3")]
    pub preprocess_action: i32,
    #[prost(
        enumeration = "super::execution_provider::ExecutionProvider",
        optional,
        tag = "4"
    )]
    pub execution_provider: ::core::option::Option<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreEntry {
    #[prost(message, optional, tag = "1")]
    pub key: ::core::option::Option<super::super::keyval::StoreInput>,
    /// MetadataKey is a string.
    #[prost(map = "string, message", tag = "2")]
    pub value: ::std::collections::HashMap<
        ::prost::alloc::string::String,
        super::super::metadata::MetadataValue,
    >,
}
