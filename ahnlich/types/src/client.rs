// This file is @generated by prost-build.
#[derive(PartialOrd, Ord, Eq, Clone, PartialEq, ::prost::Message)]
pub struct ConnectedClient {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
    #[prost(string, tag = "2")]
    pub time_connected: ::prost::alloc::string::String,
}
