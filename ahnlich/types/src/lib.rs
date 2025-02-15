pub mod ai;
pub mod bincode;
pub mod client;
pub mod db;
pub mod keyval;
pub mod metadata;
pub mod predicate;
pub mod query_builders;
pub mod similarity;
pub mod version;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServerType {
    Database,
    AI,
}
