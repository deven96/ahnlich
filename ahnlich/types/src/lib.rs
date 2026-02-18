pub mod ai;
pub mod algorithm;
pub mod client;
pub mod db;
pub mod keyval;
pub mod metadata;
pub mod predicates;
pub mod server_types;
pub mod services;
pub mod shared;
pub mod similarity;
pub mod utils;
pub mod version;
/// Binary file descriptor set for all Ahnlich proto services.
/// Used to register gRPC server reflection via `tonic-reflection`.
pub const FILE_DESCRIPTOR_SET: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/ahnlich_descriptor.bin"));
