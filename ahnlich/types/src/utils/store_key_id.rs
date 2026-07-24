use ahash::RandomState;
use serde::{Deserialize, Serialize};
use std::hash::{BuildHasher, Hash, Hasher};

/// A hash of Store key, this is more preferable when passing around references as arrays can be
/// potentially larger
/// We should be only able to generate a store key id from a 1D vector except during tests
#[derive(Debug, Clone, Copy, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreKeyId(pub u64);

#[cfg(test)]
impl From<u64> for StoreKeyId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<&crate::keyval::StoreKey> for StoreKeyId {
    fn from(value: &crate::keyval::StoreKey) -> Self {
        Self(hash_f32_vec(&value.key))
    }
}

/// Compute a fast ahash of the vector to ensure it always gives us the same value
/// and use that as a reference to the vector.
/// Fixed seed so StoreKeyId is deterministic across restarts and platforms.
/// AHasher::default() is randomly seeded per-process (DoS protection for maps),
/// which would break snapshot/persistence of StoreKeyIds across restarts.
pub fn hash_f32_vec(vec: &[f32]) -> u64 {
    let mut hasher = RandomState::with_seeds(0, 0, 0, 0).build_hasher();
    for element in vec.iter() {
        hasher.write_u32(element.to_bits());
    }
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_deterministic() {
        let vec = vec![1.0, 2.0, 3.0];
        let hash1 = hash_f32_vec(&vec);
        let hash2 = hash_f32_vec(&vec);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_different_vectors_different_hashes() {
        let vec1 = vec![1.0, 2.0, 3.0];
        let vec2 = vec![1.0, 2.0, 4.0];
        let hash1 = hash_f32_vec(&vec1);
        let hash2 = hash_f32_vec(&vec2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_store_key_id_from_store_key() {
        let key = crate::keyval::StoreKey {
            key: vec![1.0, 2.0, 3.0],
        };
        let id1 = StoreKeyId::from(&key);
        let id2 = StoreKeyId::from(&key);
        assert_eq!(id1, id2);
    }
}
