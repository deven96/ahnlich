use serde::Deserialize;
use serde::Serialize;

pub mod ai;
pub mod db;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ServerType {
    Database,
    AI,
}
