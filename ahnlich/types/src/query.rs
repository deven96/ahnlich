use std::num::NonZeroU32;

use crate::keyval::StoreKey;
use crate::metadata::MetadataKey;

/// All possible queries for the server to respond to
#[derive(Debug, Clone)]
pub enum Query {
    Connect,
    Disconnect,
    Shutdown {
        reason: Option<String>,
    },
    Create {
        store: String,
        dimension: NonZeroU32,
        create_predicates: Vec<MetadataKey>,
    },
    GetKey {
        key: StoreKey,
    },
}
