// This file is @generated by prost-build.
#[derive(Eq, Hash, Ord, PartialOrd, Clone, PartialEq, ::prost::Message)]
pub struct StoreName {
    #[prost(string, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreKey {
    #[prost(float, repeated, tag = "1")]
    pub key: ::prost::alloc::vec::Vec<f32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreInput {
    #[prost(oneof = "store_input::Value", tags = "2, 3")]
    pub value: ::core::option::Option<store_input::Value>,
}
/// Nested message and enum types in `StoreInput`.
pub mod store_input {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Value {
        #[prost(string, tag = "2")]
        RawString(::prost::alloc::string::String),
        #[prost(bytes, tag = "3")]
        Image(::prost::alloc::vec::Vec<u8>),
    }
}
/// A single entry for a store containing embedding key and its associated metadata values.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DbStoreEntry {
    #[prost(message, optional, tag = "1")]
    pub key: ::core::option::Option<StoreKey>,
    #[prost(message, optional, tag = "2")]
    pub value: ::core::option::Option<StoreValue>,
}
/// A single entry for a store containing raw key and its associated metadata values.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AiStoreEntry {
    #[prost(message, optional, tag = "1")]
    pub key: ::core::option::Option<StoreInput>,
    #[prost(message, optional, tag = "2")]
    pub value: ::core::option::Option<StoreValue>,
}
#[derive(serde::Serialize, serde::Deserialize, Clone, PartialEq, ::prost::Message)]
pub struct StoreValue {
    #[prost(map = "string, message", tag = "1")]
    pub value:
        ::std::collections::HashMap<::prost::alloc::string::String, super::metadata::MetadataValue>,
}
