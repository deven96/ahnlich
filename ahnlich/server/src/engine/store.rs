use crate::errors::ServerError;

use super::predicate::PredicateIndices;
use flurry::HashMap as ConcurrentHashMap;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet as StdHashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use types::keyval::StoreKey;
use types::keyval::StoreName;
use types::keyval::StoreValue;
use types::metadata::MetadataKey;
use types::predicate::PredicateCondition;

/// A hash of Store key, this is more preferable when passing around references as arrays can be
/// potentially larger
/// We should be only able to generate a store key id from a 1D vector except during tests

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub(crate) struct StoreKeyId(String);

#[cfg(test)]
impl From<String> for StoreKeyId {
    fn from(value: String) -> Self {
        Self(value)
    }
}

#[cfg(test)]
impl From<&str> for StoreKeyId {
    fn from(value: &str) -> Self {
        Self(value.into())
    }
}

impl From<&StoreKey> for StoreKeyId {
    fn from(value: &StoreKey) -> Self {
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

/// StoreInfo just shows store name, size and length
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StoreInfo {
    pub name: StoreName,
    pub len: usize,
    pub size_in_bytes: usize,
}

/// StoreUpsert shows how many entries were inserted and updated during a store add call
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct StoreUpsert {
    pub inserted: usize,
    pub updated: usize,
}

/// Contains all the stores that have been created in memory
#[derive(Debug)]
pub(crate) struct StoreHandler {
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    stores: ConcurrentHashMap<StoreName, Arc<Store>>,
}

impl StoreHandler {
    pub(crate) fn new() -> Self {
        Self {
            stores: ConcurrentHashMap::new(),
        }
    }

    /// Returns a store using the store name, else returns an error
    fn get(&self, store_name: &StoreName) -> Result<Arc<Store>, ServerError> {
        let store = self
            .stores
            .get(store_name, &self.stores.guard())
            .cloned()
            .ok_or(ServerError::StoreNotFound(store_name.clone()))?;
        Ok(store)
    }

    /// Matches DELKEY - removes keys from a store
    pub(crate) fn del_key_in_store(
        &self,
        store_name: StoreName,
        keys: Vec<StoreKey>,
    ) -> Result<usize, ServerError> {
        let store = self.get(&store_name)?;
        Ok(store.delete_keys(keys))
    }

    /// Matches DELPRED - removes keys from a store when value matches predicate
    pub(crate) fn del_pred_in_store(
        &self,
        store_name: StoreName,
        condition: &PredicateCondition,
    ) -> Result<usize, ServerError> {
        let store = self.get(&store_name)?;
        Ok(store.delete_matches(condition))
    }

    /// Matches SET - adds new entries into a particular store
    pub(crate) fn set_in_store(
        &self,
        store_name: StoreName,
        new: StdHashMap<StoreKey, StoreValue>,
    ) -> Result<StoreUpsert, ServerError> {
        let store = self.get(&store_name)?;
        store.add(new)
    }

    /// matches LISTSTORES - to return statistics of all stores
    pub(crate) fn list_stores(&self) -> StdHashSet<StoreInfo> {
        self.stores
            .iter(&self.stores.guard())
            .map(|(store_name, store)| StoreInfo {
                name: store_name.clone(),
                len: store.len(),
                size_in_bytes: std::mem::size_of_val(store),
            })
            .collect()
    }

    /// Matches CREATE - Creates a store if not exist, else return an error
    pub(crate) fn create_store(
        &self,
        store_name: StoreName,
        dimension: NonZeroUsize,
        predicates: Vec<MetadataKey>,
    ) -> Result<(), ServerError> {
        if let Err(_) = self.stores.try_insert(
            store_name.clone(),
            Arc::new(Store::create(dimension, predicates)),
            &self.stores.guard(),
        ) {
            return Err(ServerError::StoreAlreadyExists(store_name));
        }
        Ok(())
    }
}

/// A Store is a single database containing multiple N*1 arrays where N is the dimension of the
/// store to which all arrays must conform
#[derive(Debug)]
pub(crate) struct Store {
    dimension: NonZeroUsize,
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    id_to_value: ConcurrentHashMap<StoreKeyId, (StoreKey, StoreValue)>,
    /// Indices to filter for the store
    predicate_indices: PredicateIndices,
}

impl Store {
    /// Creates a new empty store
    fn create(dimension: NonZeroUsize, predicates: Vec<MetadataKey>) -> Self {
        Self {
            dimension,
            id_to_value: ConcurrentHashMap::new(),
            predicate_indices: PredicateIndices::init(predicates),
        }
    }

    fn delete(&self, keys: impl Iterator<Item = StoreKeyId>) -> usize {
        let keys: Vec<StoreKeyId> = keys.collect();
        let pinned = self.id_to_value.pin();
        let removed: Vec<_> = keys.iter().flat_map(|k| pinned.remove(k)).collect();
        self.predicate_indices.remove_store_keys(&keys);
        removed.len()
    }

    /// Deletes a bunch of store keys from the store
    fn delete_keys(&self, del: Vec<StoreKey>) -> usize {
        let keys = del.iter().map(From::from);
        self.delete(keys)
    }

    /// Deletes a bunch of store keys from the store matching a specific predicate
    fn delete_matches(&self, condition: &PredicateCondition) -> usize {
        let matches = self.predicate_indices.matches(condition).into_iter();
        self.delete(matches)
    }

    /// Adds a bunch of entries into the store if they match the dimensions
    /// Returns the len of values added, if a value already existed it is updated but not counted
    /// as a new insert
    fn add(&self, new: StdHashMap<StoreKey, StoreValue>) -> Result<StoreUpsert, ServerError> {
        let store_dimension: usize = self.dimension.into();
        let check_bounds = |(store_key, store_val): (StoreKey, StoreValue)| -> Result<(StoreKeyId, (StoreKey, StoreValue)), ServerError> {
            let input_dimension = store_key.0.len();
            if input_dimension != store_dimension {
                Err(ServerError::StoreDimensionMismatch { store_dimension, input_dimension  })
            } else {
                Ok(((&store_key).into(), (store_key, store_val)))
            }
        };
        let res: Vec<(StoreKeyId, (StoreKey, StoreValue))> = new
            .into_iter()
            .map(|item| check_bounds(item))
            .collect::<Result<_, _>>()?;
        let predicate_insert = res
            .iter()
            .map(|(k, (_, v))| (k.clone(), v.clone()))
            .collect();
        let pinned = self.id_to_value.pin();
        let (mut inserted, mut updated) = (0, 0);
        for (key, val) in res {
            if pinned.insert(key, val).is_none() {
                inserted += 1
            } else {
                updated += 1
            }
        }
        self.predicate_indices.add(predicate_insert);
        Ok(StoreUpsert { inserted, updated })
    }

    /// Returns the number of key value pairs in the store
    fn len(&self) -> usize {
        self.id_to_value.pin().len()
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::*;
    use ndarray::array;
    use ndarray::Array1;

    #[test]
    fn test_compute_store_key_id_empty_vector() {
        let array: Array1<f32> = Array1::zeros(0);
        let store_key: StoreKeyId = (&StoreKey(array)).into();
        assert_eq!(
            store_key,
            StoreKeyId("e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855".into())
        );
    }

    #[test]
    fn test_compute_store_key_id_single_element_array() {
        let array = array![1.23];
        let store_key: StoreKeyId = (&StoreKey(array)).into();
        assert_eq!(
            store_key,
            StoreKeyId("b2d6f6f0d78e1e5c6b4d42226c1c42105ea241d2642d6f96f69141788a1d16db".into())
        );
    }

    #[test]
    fn test_compute_store_key_id_multiple_elements_array() {
        let array = array![1.22, 2.11, 3.22, 3.11];
        let store_key: StoreKeyId = (&StoreKey(array)).into();
        assert_eq!(
            store_key,
            StoreKeyId("1cb232f8e9e23d1576db3d7d1b93a15922263b31b6bf83c57d6b9b0ce913c1bf".into())
        );
    }

    fn create_store_handler() -> (
        Arc<StoreHandler>,
        Vec<Result<(), ServerError>>,
        Vec<Result<(), ServerError>>,
    ) {
        let handler = Arc::new(StoreHandler::new());
        let handles = (0..3).map(|i| {
            let shared_handler = handler.clone();
            let handle = loom::thread::spawn(move || {
                let (store_name, size) = if i % 2 == 0 {
                    ("Even", 1024)
                } else {
                    ("Odd", 2048)
                };
                shared_handler.create_store(
                    StoreName(store_name.to_string()),
                    NonZeroUsize::new(size).unwrap(),
                    vec![],
                )
            });
            handle
        });
        let (oks, errs): (Vec<_>, Vec<_>) = handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .partition(Result::is_ok);

        (handler, oks, errs)
    }

    #[test]
    fn test_get_and_create_store_handler() {
        loom::model(|| {
            let (handler, oks, errs) = create_store_handler();
            assert_eq!(oks.len(), 2);
            assert_eq!(
                errs,
                vec![Err(ServerError::StoreAlreadyExists(StoreName(
                    "Even".to_string()
                )))]
            );
            // test out a store that does not exist
            let fake_store = StoreName("Random".to_string());
            assert_eq!(
                handler.get(&fake_store).unwrap_err(),
                ServerError::StoreNotFound(fake_store)
            );
        })
    }

    #[test]
    fn test_get_store_info() {
        loom::model(|| {
            let (handler, oks, errs) = create_store_handler();
        })
    }
}
