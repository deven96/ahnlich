//! Shared snapshot serialization helpers.
//!
//! Both Raft state-machine snapshots (in `ahnlich-replication`) and the
//! standalone [`crate::persistence::Persistence`] task share these
//! serialize/deserialize routines so that the on-disk format is decided in
//! exactly one place. If the format ever changes, both code paths move
//! together.
//!
//! Milestone 1 introduces the helpers without changing the persistence task,
//! that refactor lands in Milestone 6 alongside the cluster-mode persistence
//! disable. Until then, [`Persistence`](crate::persistence::Persistence) keeps
//! its existing `serde_json::to_writer` / `from_reader` calls so this is a
//! pure addition.

use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("snapshot serde error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Encode a snapshot to a self-describing byte buffer. The current format is
/// `serde_json`, chosen for parity with the existing persistence task.
pub fn serialize_snapshot<T: Serialize>(state: &T) -> Result<Vec<u8>, SnapshotError> {
    Ok(serde_json::to_vec(state)?)
}

/// Decode a snapshot produced by [`serialize_snapshot`].
pub fn deserialize_snapshot<T: DeserializeOwned>(data: &[u8]) -> Result<T, SnapshotError> {
    Ok(serde_json::from_slice(data)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Sample {
        name: String,
        counts: HashMap<String, u32>,
        flags: Vec<bool>,
    }

    #[test]
    fn round_trip_preserves_value() {
        let mut counts = HashMap::new();
        counts.insert("alpha".to_string(), 1);
        counts.insert("beta".to_string(), 2);
        let original = Sample {
            name: "snap".into(),
            counts,
            flags: vec![true, false, true],
        };

        let encoded = serialize_snapshot(&original).expect("serialize");
        let decoded: Sample = deserialize_snapshot(&encoded).expect("deserialize");
        assert_eq!(decoded, original);
    }

    #[test]
    fn deserialize_rejects_garbage() {
        let bad: &[u8] = b"\xff\xff not json";
        let err = deserialize_snapshot::<Sample>(bad).unwrap_err();
        assert!(matches!(err, SnapshotError::Serde(_)));
    }
}
