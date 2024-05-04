use super::predicate::PredicateIndices;
use flurry::HashMap as ConcurrentHashMap;
use flurry::HashSet as ConcurrentHashSet;
use ndarray::Array1;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap as StdHashMap;
use std::num::NonZeroU32;
use types::metadata::MetadataKey;
use types::metadata::MetadataValue;

/// A store key is always an f32 one dimensional array
#[derive(Debug)]
struct StoreKey(Array1<f32>);

/// A hash of Store key, this is more preferable when passing around references as arrays can be
/// potentially larger
/// We should be only able to generate a store key id from a 1D vector except during tests
#[cfg(test)]
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) struct StoreKeyId(pub String);
#[cfg(not(test))]
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) struct StoreKeyId(String);

impl From<StoreKey> for StoreKeyId {
    fn from(value: StoreKey) -> Self {
        // compute a standard SHA256 hash of the vector to ensure it always gives us the same value
        // and use that as a reference to the vector
        let mut hasher = Sha256::new();
        for element in value.0.iter() {
            let bytes = element.to_ne_bytes();
            hasher.update(bytes);
        }
        let result = hasher.finalize();
        // Convert the hash bytes to a hexadecimal string
        let hash_string = result
            .iter()
            .map(|byte| format!("{:02x}", byte))
            .collect::<String>();
        Self(hash_string)
    }
}

/// A store value for now is a simple key value pair of strings
pub(crate) type StoreValue = StdHashMap<MetadataKey, MetadataValue>;

/// A Store is a single database containing multiple N*1 arrays where N is the dimension of the
/// store to which all arrays must conform
#[derive(Debug)]
struct Store {
    dimension: NonZeroU32,
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    id_to_value: ConcurrentHashMap<StoreKeyId, StoreValue>,
    /// We store keys separately as we can always compute their hash and find the StoreValue from
    /// there
    keys: ConcurrentHashSet<StoreKey>,
    /// Indices to filter for the store
    predicate_indices: PredicateIndices,
}

impl Store {
    /// Creates a new empty store
    fn create(dimension: NonZeroU32, predicates: Vec<MetadataKey>) -> Self {
        Self {
            dimension,
            id_to_value: ConcurrentHashMap::new(),
            keys: ConcurrentHashSet::new(),
            predicate_indices: PredicateIndices::init(predicates),
        }
    }

    /// Returns the number of key value pairs in the store
    fn len(&self) -> usize {
        self.id_to_value.pin().len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::array;

    #[test]
    fn test_compute_store_key_id_empty_vector() {
        let array: Array1<f32> = Array1::zeros(0);
        let store_key: StoreKeyId = StoreKey(array).into();
        assert_eq!(
            store_key,
            StoreKeyId("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into())
        );
    }

    #[test]
    fn test_compute_store_key_id_single_element_array() {
        let array = array![1.23];
        let store_key: StoreKeyId = StoreKey(array).into();
        assert_eq!(
            store_key,
            StoreKeyId("b2d6f6f0d78e1e5c6b4d42226c1c42105ea241d2642d6f96f69141788a1d16db".into())
        );
    }

    #[test]
    fn test_compute_store_key_id_multiple_elements_array() {
        let array = array![1.22, 2.11, 3.22, 3.11];
        let store_key: StoreKeyId = StoreKey(array).into();
        assert_eq!(
            store_key,
            StoreKeyId("1cb232f8e9e23d1576db3d7d1b93a15922263b31b6bf83c57d6b9b0ce913c1bf".into())
        );
    }
}
