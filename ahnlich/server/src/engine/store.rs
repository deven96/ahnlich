use crate::errors::ServerError;

use super::super::algorithm::FindSimilarN;
use super::predicate::PredicateIndices;
use flurry::HashMap as ConcurrentHashMap;
use sha2::Digest;
use sha2::Sha256;
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet as StdHashSet;
use std::mem::size_of_val;
use std::num::NonZeroUsize;
use std::sync::Arc;
use types::keyval::SearchInput;
use types::keyval::StoreKey;
use types::keyval::StoreName;
use types::keyval::StoreValue;
use types::metadata::MetadataKey;
use types::predicate::PredicateCondition;
use types::similarity::Algorithm;
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
        store_name: &StoreName,
        keys: Vec<StoreKey>,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        Ok(store.delete_keys(keys))
    }

    /// Matches DELPRED - removes keys from a store when value matches predicate
    pub(crate) fn del_pred_in_store(
        &self,
        store_name: &StoreName,
        condition: &PredicateCondition,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        Ok(store.delete_matches(condition))
    }

    /// Matches GETSIMN - gets all similar from a store that also match a predicate
    pub(crate) fn get_sim_in_store(
        &self,
        store_name: &StoreName,
        search_input: SearchInput,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
        condition: Option<PredicateCondition>,
    ) -> Result<Vec<(StoreKey, StoreValue, f64)>, ServerError> {
        let store = self.get(store_name)?;

        let filtered = if let Some(ref condition) = condition {
            let output = self.get_pred_in_store(store_name, condition)?;
            output
        } else {
            store.get_all()
        };

        if filtered.is_empty() {
            return Ok(vec![]);
        }

        let filtered_iter = filtered.iter().map(|(key, _)| key);

        let similar_result = algorithm.find_similar_n(&search_input, filtered_iter, closest_n);
        let mut keys_to_value_map: StdHashMap<StoreKeyId, StoreValue> =
            StdHashMap::from_iter(filtered.iter().map(|(store_key, store_value)| {
                (StoreKeyId::from(store_key), store_value.clone())
            }));

        Ok(similar_result
            .into_iter()
            .flat_map(|(store_key, similarity)| {
                keys_to_value_map
                    .remove(&StoreKeyId::from(store_key))
                    .map(|value| (store_key.clone(), value, similarity))
            })
            .collect())
    }

    /// Matches GETPRED - gets all matching predicates from a store
    pub(crate) fn get_pred_in_store(
        &self,
        store_name: &StoreName,
        condition: &PredicateCondition,
    ) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        let store = self.get(store_name)?;
        Ok(store.get_matches(condition))
    }

    /// Matches GETKEY - gets all keys matching the inputs
    pub(crate) fn get_key_in_store(
        &self,
        store_name: &StoreName,
        keys: Vec<StoreKey>,
    ) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        let store = self.get(store_name)?;
        Ok(store.get_keys(keys))
    }

    /// Matches SET - adds new entries into a particular store
    pub(crate) fn set_in_store(
        &self,
        store_name: &StoreName,
        new: Vec<(StoreKey, StoreValue)>,
    ) -> Result<StoreUpsert, ServerError> {
        let store = self.get(store_name)?;
        store.add(new)
    }

    /// matches LISTSTORES - to return statistics of all stores
    pub(crate) fn list_stores(&self) -> StdHashSet<StoreInfo> {
        self.stores
            .iter(&self.stores.guard())
            .map(|(store_name, store)| StoreInfo {
                name: store_name.clone(),
                len: store.len(),
                size_in_bytes: store.size(),
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

    /// Matches DROPSTORE - Drops a store if exist, else returns an error
    pub(crate) fn drop_store(&self, store_name: StoreName) -> Result<(), ServerError> {
        let pinned = self.stores.pin();
        pinned
            .remove(&store_name)
            .ok_or(ServerError::StoreNotFound(store_name))?;
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
        if del.is_empty() {
            return 0;
        }
        let keys = del.iter().map(From::from);
        self.delete(keys)
    }

    /// Deletes a bunch of store keys from the store matching a specific predicate
    fn delete_matches(&self, condition: &PredicateCondition) -> usize {
        let matches = self.predicate_indices.matches(condition).into_iter();
        self.delete(matches)
    }

    /// Gets a bunch of store keys from the store
    fn get_keys(&self, val: Vec<StoreKey>) -> Vec<(StoreKey, StoreValue)> {
        if val.is_empty() {
            return vec![];
        }
        let keys = val.iter().map(From::from);
        self.get(keys)
    }

    /// Gets a bunch of store entries that matches a predicate condition
    fn get_matches(&self, condition: &PredicateCondition) -> Vec<(StoreKey, StoreValue)> {
        let matches = self.predicate_indices.matches(condition).into_iter();
        self.get(matches)
    }

    fn get(&self, keys: impl Iterator<Item = StoreKeyId>) -> Vec<(StoreKey, StoreValue)> {
        let pinned = self.id_to_value.pin();
        keys.flat_map(|k| pinned.get(&k).cloned()).collect()
    }
    fn get_all(&self) -> Vec<(StoreKey, StoreValue)> {
        let pinned = self.id_to_value.pin();
        pinned
            .into_iter()
            .map(|(_key, (store_key, store_value))| (store_key.clone(), store_value.clone()))
            .collect()
    }

    /// Adds a bunch of entries into the store if they match the dimensions
    /// Returns the len of values added, if a value already existed it is updated but not counted
    /// as a new insert
    fn add(&self, new: Vec<(StoreKey, StoreValue)>) -> Result<StoreUpsert, ServerError> {
        if new.is_empty() {
            return Ok(StoreUpsert {
                inserted: 0,
                updated: 0,
            });
        }
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

    /// TODO: Fix nested calculation of sizes using size_of_val
    fn size(&self) -> usize {
        size_of_val(&self)
            + size_of_val(&self.id_to_value)
            + self
                .id_to_value
                .iter(&self.id_to_value.guard())
                .map(|(k, v)| {
                    size_of_val(k)
                        + size_of_val(&v.0)
                        + v.1
                            .iter()
                            .map(|(inner_k, inner_val)| {
                                size_of_val(inner_k) + size_of_val(inner_val)
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.predicate_indices.size()
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::*;
    use ndarray::array;
    use ndarray::Array1;
    use std::collections::HashMap as StdHashMap;
    use types::metadata::MetadataKey;
    use types::metadata::MetadataValue;
    use types::predicate::Predicate;
    use types::predicate::PredicateOp;

    #[test]
    fn test_compute_store_key_id_empty_vector() {
        let array: Array1<f64> = Array1::zeros(0);
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
            StoreKeyId("5658eb3a4999cac5729dd21f0e5e36587bf7e37aef139093432405a22a430046".into())
        );
    }

    #[test]
    fn test_compute_store_key_id_multiple_elements_array() {
        let array = array![1.22, 2.11, 3.22, 3.11];
        let store_key: StoreKeyId = (&StoreKey(array)).into();
        assert_eq!(
            store_key,
            StoreKeyId("c0c5df8fe8939c0c73446b33500a14f7512514b143ead3655f36f8947eb9898b".into())
        );
    }

    fn create_store_handler_no_loom(predicates: Vec<MetadataKey>) -> Arc<StoreHandler> {
        let handler = Arc::new(StoreHandler::new());
        let handles = (0..3).map(|i| {
            let predicates = predicates.clone();
            let shared_handler = handler.clone();
            let handle = std::thread::spawn(move || {
                let (store_name, size) = if i % 2 == 0 { ("Even", 5) } else { ("Odd", 3) };
                shared_handler.create_store(
                    StoreName(store_name.to_string()),
                    NonZeroUsize::new(size).unwrap(),
                    predicates,
                )
            });
            handle
        });
        for handle in handles {
            let _ = handle.join().unwrap();
        }
        handler
    }

    fn create_store_handler(
        predicates: Vec<MetadataKey>,
    ) -> (
        Arc<StoreHandler>,
        Vec<Result<(), ServerError>>,
        Vec<Result<(), ServerError>>,
    ) {
        let handler = Arc::new(StoreHandler::new());
        let handles = (0..3).map(|i| {
            let predicates = predicates.clone();
            let shared_handler = handler.clone();
            let handle = loom::thread::spawn(move || {
                let (store_name, size) = if i % 2 == 0 { ("Even", 5) } else { ("Odd", 3) };
                shared_handler.create_store(
                    StoreName(store_name.to_string()),
                    NonZeroUsize::new(size).unwrap(),
                    predicates,
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
            let (handler, oks, errs) = create_store_handler(vec![]);
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
    fn test_create_and_set_in_store_fails() {
        loom::model(|| {
            let (handler, _oks, _errs) = create_store_handler(vec![]);
            let even_store = StoreName("Even".into());
            let fake_store = StoreName("Fake".into());
            // set in nonexistent store should fail
            assert_eq!(
                handler.set_in_store(&fake_store, vec![]).unwrap_err(),
                ServerError::StoreNotFound(fake_store)
            );
            // set in store with wrong dimensions should fail
            assert_eq!(
                handler
                    .set_in_store(
                        &even_store,
                        vec![(
                            StoreKey(array![0.33, 0.44, 0.5]),
                            StdHashMap::from_iter(vec![(
                                MetadataKey::new("author".into()),
                                MetadataValue::new("Vincent".into()),
                            ),])
                        ),]
                    )
                    .unwrap_err(),
                ServerError::StoreDimensionMismatch {
                    store_dimension: 5,
                    input_dimension: 3
                }
            );
        })
    }

    #[test]
    fn test_create_and_set_in_store_passes() {
        loom::model(|| {
            let (handler, _oks, _errs) = create_store_handler(vec![]);
            let odd_store = StoreName("Odd".into());
            let input_arr = array![0.1, 0.2, 0.3];
            let ret = handler
                .set_in_store(
                    &odd_store,
                    vec![(
                        StoreKey(input_arr.clone()),
                        StdHashMap::from_iter(vec![(
                            MetadataKey::new("author".into()),
                            MetadataValue::new("Lex Luthor".into()),
                        )]),
                    )],
                )
                .unwrap();
            assert_eq!(
                ret,
                StoreUpsert {
                    inserted: 1,
                    updated: 0,
                }
            );
            let ret = handler
                .set_in_store(
                    &odd_store,
                    vec![(
                        StoreKey(input_arr.clone()),
                        StdHashMap::from_iter(vec![(
                            MetadataKey::new("author".into()),
                            MetadataValue::new("Clark Kent".into()),
                        )]),
                    )],
                )
                .unwrap();
            assert_eq!(
                ret,
                StoreUpsert {
                    inserted: 0,
                    updated: 1,
                }
            );
        })
    }

    #[test]
    fn test_get_key_in_store() {
        loom::model(|| {
            let (handler, _oks, _errs) = create_store_handler(vec![]);
            let odd_store = StoreName("Odd".into());
            let fake_store = StoreName("Fakest".into());
            let input_arr_1 = array![0.1, 0.2, 0.3];
            let input_arr_2 = array![0.2, 0.3, 0.4];
            handler
                .set_in_store(
                    &odd_store,
                    vec![(
                        StoreKey(input_arr_1.clone()),
                        StdHashMap::from_iter(vec![(
                            MetadataKey::new("author".into()),
                            MetadataValue::new("Lex Luthor".into()),
                        )]),
                    )],
                )
                .unwrap();
            handler
                .set_in_store(
                    &odd_store,
                    vec![(
                        StoreKey(input_arr_2.clone()),
                        StdHashMap::from_iter(vec![(
                            MetadataKey::new("author".into()),
                            MetadataValue::new("Clark Kent".into()),
                        )]),
                    )],
                )
                .unwrap();
            assert_eq!(
                handler.get_key_in_store(&fake_store, vec![]).unwrap_err(),
                ServerError::StoreNotFound(fake_store)
            );
            let ret = handler
                .get_key_in_store(
                    &odd_store,
                    vec![StoreKey(input_arr_1), StoreKey(input_arr_2)],
                )
                .unwrap();
            assert_eq!(ret.len(), 2);
            assert_eq!(
                ret[0]
                    .1
                    .get(&MetadataKey::new("author".into()))
                    .cloned()
                    .unwrap(),
                MetadataValue::new("Lex Luthor".into())
            );
            assert_eq!(
                ret[1]
                    .1
                    .get(&MetadataKey::new("author".into()))
                    .cloned()
                    .unwrap(),
                MetadataValue::new("Clark Kent".into())
            );
        })
    }

    #[test]
    fn test_get_pred_in_store() {
        let handler = create_store_handler_no_loom(vec![MetadataKey::new("rank".into())]);
        let even_store = StoreName("Even".into());
        let input_arr_1 = array![0.1, 0.2, 0.3, 0.4, 0.5];
        let input_arr_2 = array![0.2, 0.3, 0.4, 0.5, 0.6];
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey(input_arr_1.clone()),
                    StdHashMap::from_iter(vec![(
                        MetadataKey::new("rank".into()),
                        MetadataValue::new("Joinin".into()),
                    )]),
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey(input_arr_2.clone()),
                    StdHashMap::from_iter(vec![(
                        MetadataKey::new("rank".into()),
                        MetadataValue::new("Genin".into()),
                    )]),
                )],
            )
            .unwrap();
        let condition = &PredicateCondition::Value(Predicate {
            key: MetadataKey::new("rank".into()),
            value: MetadataValue::new("Hokage".into()),
            op: PredicateOp::Equals,
        });
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert!(res.is_empty());
        let condition = &PredicateCondition::Value(Predicate {
            key: MetadataKey::new("rank".into()),
            value: MetadataValue::new("Hokage".into()),
            op: PredicateOp::NotEquals,
        });
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 2);
        let condition = &PredicateCondition::Value(Predicate {
            key: MetadataKey::new("rank".into()),
            value: MetadataValue::new("Joinin".into()),
            op: PredicateOp::Equals,
        });
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_get_store_info() {
        let handler = create_store_handler_no_loom(vec![MetadataKey::new("rank".into())]);
        let odd_store = StoreName("Odd".into());
        let even_store = StoreName("Even".into());
        let input_arr_1 = array![0.1, 0.2, 0.3];
        let input_arr_2 = array![0.2, 0.3, 0.4];
        handler
            .set_in_store(
                &odd_store,
                vec![(
                    StoreKey(input_arr_1.clone()),
                    StdHashMap::from_iter(vec![(
                        MetadataKey::new("rank".into()),
                        MetadataValue::new("Joinin".into()),
                    )]),
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &odd_store,
                vec![(
                    StoreKey(input_arr_2.clone()),
                    StdHashMap::from_iter(vec![(
                        MetadataKey::new("rank".into()),
                        MetadataValue::new("Genin".into()),
                    )]),
                )],
            )
            .unwrap();
        let stores = handler.list_stores();
        assert_eq!(
            stores,
            StdHashSet::from_iter([
                StoreInfo {
                    name: odd_store,
                    len: 2,
                    size_in_bytes: 2096,
                },
                StoreInfo {
                    name: even_store,
                    len: 0,
                    size_in_bytes: 1728,
                },
            ])
        )
    }
}
