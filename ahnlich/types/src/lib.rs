pub mod bincode;
pub mod keyval;
pub mod metadata;
pub mod predicate;
pub mod query;
pub mod server;
pub mod similarity;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
