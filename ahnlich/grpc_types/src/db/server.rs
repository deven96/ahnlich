// This file is @generated by prost-build.
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct Unit {}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct Pong {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientList {
    #[prost(message, repeated, tag = "1")]
    pub clients: ::prost::alloc::vec::Vec<super::super::client::ConnectedClient>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreList {
    #[prost(message, repeated, tag = "1")]
    pub stores: ::prost::alloc::vec::Vec<StoreInfo>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InfoServer {
    #[prost(message, optional, tag = "1")]
    pub info: ::core::option::Option<super::super::shared::info::ServerInfo>,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct Set {
    #[prost(message, optional, tag = "1")]
    pub upsert: ::core::option::Option<super::super::shared::info::StoreUpsert>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Get {
    #[prost(message, repeated, tag = "1")]
    pub entries: ::prost::alloc::vec::Vec<super::super::keyval::StoreEntry>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSimNEntry {
    #[prost(message, optional, tag = "1")]
    pub key: ::core::option::Option<super::super::keyval::StoreKey>,
    #[prost(message, optional, tag = "2")]
    pub value: ::core::option::Option<super::super::keyval::StoreValue>,
    #[prost(message, optional, tag = "3")]
    pub similarity: ::core::option::Option<super::super::similarity::Similarity>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GetSimN {
    #[prost(message, repeated, tag = "1")]
    pub entries: ::prost::alloc::vec::Vec<GetSimNEntry>,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct Del {
    #[prost(uint64, tag = "1")]
    pub deleted_count: u64,
}
#[derive(Clone, Copy, PartialEq, ::prost::Message)]
pub struct CreateIndex {
    #[prost(uint64, tag = "1")]
    pub created_indexes: u64,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ServerResponse {
    #[prost(oneof = "server_response::Response", tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10")]
    pub response: ::core::option::Option<server_response::Response>,
}
/// Nested message and enum types in `ServerResponse`.
pub mod server_response {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Response {
        #[prost(message, tag = "1")]
        Unit(super::Unit),
        #[prost(message, tag = "2")]
        Pong(super::Pong),
        #[prost(message, tag = "3")]
        ClientList(super::ClientList),
        #[prost(message, tag = "4")]
        StoreList(super::StoreList),
        #[prost(message, tag = "5")]
        InfoServer(super::InfoServer),
        #[prost(message, tag = "6")]
        Set(super::Set),
        #[prost(message, tag = "7")]
        Get(super::Get),
        #[prost(message, tag = "8")]
        GetSimN(super::GetSimN),
        #[prost(message, tag = "9")]
        Del(super::Del),
        #[prost(message, tag = "10")]
        CreateIndex(super::CreateIndex),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StoreInfo {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(uint64, tag = "2")]
    pub len: u64,
    #[prost(uint64, tag = "3")]
    pub size_in_bytes: u64,
}
