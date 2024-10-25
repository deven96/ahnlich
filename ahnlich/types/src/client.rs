use serde::Deserialize;
use serde::Serialize;
use std::hash::Hash;
use std::hash::Hasher;
use std::time::SystemTime;

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialOrd, Ord)]
pub struct ConnectedClient {
    pub address: String,
    // NOTE: We are using System specific time so the time marked by clients cannot be relied on to
    // be monotonic and the size depends on operating system
    pub time_connected: SystemTime,
}

// NOTE: ConnectedClient should be unique purely by address assuming we are not doing any TCP magic
// to allow port reuse
impl Hash for ConnectedClient {
    fn hash<H>(&self, state: &mut H)
    where
        H: Hasher,
    {
        self.address.hash(state)
    }
}

impl PartialEq for ConnectedClient {
    fn eq(&self, other: &Self) -> bool {
        self.address.eq(&other.address)
    }
}
