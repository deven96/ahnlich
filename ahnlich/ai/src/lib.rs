pub mod cli;
pub mod engine;
pub mod error;
mod manager;
pub mod server;
#[cfg(test)]
mod tests;

// FIXME: Replace with grpc metadatakey
pub(crate) static AHNLICH_AI_RESERVED_META_KEY: &str = "_ahnlich_input_key";
// Sequential index for OneToMany models (e.g., 0, 1, 2... for multiple outputs per input)
pub(crate) static AHNLICH_AI_ONE_TO_MANY_INDEX_META_KEY: &str = "_ahnlich_one_to_many_index";
