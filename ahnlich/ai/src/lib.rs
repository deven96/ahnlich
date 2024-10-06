use ahnlich_types::metadata::MetadataKey;

pub mod cli;
pub mod engine;
pub mod error;
mod manager;
pub mod server;
use once_cell::sync::Lazy;
#[cfg(test)]
mod tests;

pub(crate) static AHNLICH_AI_RESERVED_META_KEY: Lazy<MetadataKey> =
    Lazy::new(|| MetadataKey::new(String::from("_ahnlich_input_key")));
