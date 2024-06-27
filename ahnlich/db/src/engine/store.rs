use crate::errors::ServerError;

use super::super::algorithm::FindSimilarN;
use super::predicate::PredicateIndices;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::MetadataKey;
use ahnlich_types::predicate::PredicateCondition;
use ahnlich_types::server::StoreInfo;
use ahnlich_types::server::StoreUpsert;
use ahnlich_types::similarity::Algorithm;
use ahnlich_types::similarity::Similarity;
use flurry::HashMap as ConcurrentHashMap;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet as StdHashSet;
use std::mem::size_of_val;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
/// A hash of Store key, this is more preferable when passing around references as arrays can be
/// potentially larger
/// We should be only able to generate a store key id from a 1D vector except during tests

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
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
        // compute a fast blake hash of the vector to ensure it always gives us the same value
        // and use that as a reference to the vector
        let mut hasher = blake3::Hasher::new();
        for element in value.0.iter() {
            let bytes = element.to_ne_bytes();
            hasher.update(&bytes);
        }
        let result = hasher.finalize();
        Self(format!("{result}"))
    }
}

/// Contains all the stores that have been created in memory
#[derive(Debug)]
pub(crate) struct StoreHandler {
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    stores: Stores,
    pub write_flag: Arc<AtomicBool>,
}

pub type Stores = Arc<ConcurrentHashMap<StoreName, Arc<Store>>>;

impl StoreHandler {
    pub(crate) fn new(write_flag: Arc<AtomicBool>) -> Self {
        Self {
            stores: Arc::new(ConcurrentHashMap::new()),
            write_flag,
        }
    }

    pub(crate) fn get_stores(&self) -> Stores {
        self.stores.clone()
    }

    #[cfg(test)]
    pub fn write_flag(&self) -> Arc<AtomicBool> {
        self.write_flag.clone()
    }

    fn set_write_flag(&self) {
        let _ = self
            .write_flag
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    }

    pub(crate) fn use_snapshot(&mut self, stores_snapshot: Stores) {
        self.stores = stores_snapshot;
    }

    /// Returns a store using the store name, else returns an error
    #[tracing::instrument(skip(self))]
    fn get(&self, store_name: &StoreName) -> Result<Arc<Store>, ServerError> {
        let store = self
            .stores
            .get(store_name, &self.stores.guard())
            .cloned()
            .ok_or(ServerError::StoreNotFound(store_name.clone()))?;
        Ok(store)
    }

    /// Matches CREATEINDEX - reindexes a store with some predicate values
    #[tracing::instrument(skip(self))]
    pub(crate) fn create_index(
        &self,
        store_name: &StoreName,
        predicates: Vec<MetadataKey>,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        let created_predicates = store.create_index(predicates);
        if created_predicates > 0 {
            self.set_write_flag()
        }
        Ok(created_predicates)
    }

    /// Matches DELKEY - removes keys from a store
    #[tracing::instrument(skip(self))]
    pub(crate) fn del_key_in_store(
        &self,
        store_name: &StoreName,
        keys: Vec<StoreKey>,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        let deleted = store.delete_keys(keys)?;
        if deleted > 0 {
            self.set_write_flag();
        };
        Ok(deleted)
    }

    /// Matches DELPRED - removes keys from a store when value matches predicate
    #[tracing::instrument(skip(self))]
    pub(crate) fn del_pred_in_store(
        &self,
        store_name: &StoreName,
        condition: &PredicateCondition,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        let deleted = store.delete_matches(condition)?;
        if deleted > 0 {
            self.set_write_flag();
        };
        Ok(deleted)
    }

    /// Matches GETSIMN - gets all similar from a store that also match a predicate
    #[tracing::instrument(skip(self))]
    pub(crate) fn get_sim_in_store(
        &self,
        store_name: &StoreName,
        search_input: StoreKey,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
        condition: Option<PredicateCondition>,
    ) -> Result<Vec<(StoreKey, StoreValue, Similarity)>, ServerError> {
        let store = self.get(store_name)?;
        let store_dimension = store.dimension.get();
        let input_dimension = search_input.dimension();

        if input_dimension != store_dimension {
            return Err(ServerError::StoreDimensionMismatch {
                store_dimension,
                input_dimension,
            });
        }

        let filtered = if let Some(ref condition) = condition {
            store.get_matches(condition)?
        } else {
            store.get_all()
        };

        if filtered.is_empty() {
            return Ok(vec![]);
        }

        let filtered_iter = filtered.iter().map(|(key, _)| key);

        let similar_result = algorithm.find_similar_n(&search_input, filtered_iter, closest_n);
        let mut keys_to_value_map: StdHashMap<StoreKeyId, &StoreValue> = StdHashMap::from_iter(
            filtered
                .iter()
                .map(|(store_key, store_value)| (StoreKeyId::from(store_key), store_value)),
        );

        Ok(similar_result
            .into_iter()
            .flat_map(|(store_key, similarity)| {
                keys_to_value_map
                    .remove(&StoreKeyId::from(store_key))
                    .map(|value| (store_key.to_owned(), value.clone(), Similarity(similarity)))
            })
            .collect())
    }

    /// Matches GETPRED - gets all matching predicates from a store
    #[tracing::instrument(skip(self))]
    pub(crate) fn get_pred_in_store(
        &self,
        store_name: &StoreName,
        condition: &PredicateCondition,
    ) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        let store = self.get(store_name)?;
        store.get_matches(condition)
    }

    /// Matches GETKEY - gets all keys matching the inputs
    #[tracing::instrument(skip(self))]
    pub(crate) fn get_key_in_store(
        &self,
        store_name: &StoreName,
        keys: Vec<StoreKey>,
    ) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        let store = self.get(store_name)?;
        store.get_keys(keys)
    }

    /// Matches SET - adds new entries into a particular store
    #[tracing::instrument(skip(self))]
    pub(crate) fn set_in_store(
        &self,
        store_name: &StoreName,
        new: Vec<(StoreKey, StoreValue)>,
    ) -> Result<StoreUpsert, ServerError> {
        let store = self.get(store_name)?;
        let upsert = store.add(new)?;
        if upsert.modified() {
            self.set_write_flag();
        }
        Ok(upsert)
    }

    /// matches LISTSTORES - to return statistics of all stores
    #[tracing::instrument(skip(self))]
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

    /// Matches CREATESTORE - Creates a store if not exist, else return an error
    #[tracing::instrument(skip(self))]
    pub(crate) fn create_store(
        &self,
        store_name: StoreName,
        dimension: NonZeroUsize,
        predicates: Vec<MetadataKey>,
        error_if_exists: bool,
    ) -> Result<(), ServerError> {
        if self
            .stores
            .try_insert(
                store_name.clone(),
                Arc::new(Store::create(dimension, predicates)),
                &self.stores.guard(),
            )
            .is_err()
            && error_if_exists
        {
            return Err(ServerError::StoreAlreadyExists(store_name));
        }
        self.set_write_flag();
        Ok(())
    }

    /// Matches DROPINDEXPRED - Drops predicate index if exists, else returns an error
    #[tracing::instrument(skip(self))]
    pub(crate) fn drop_index_in_store(
        &self,
        store_name: &StoreName,
        predicates: Vec<MetadataKey>,
        error_if_not_exists: bool,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        let deleted = store.drop_predicates(predicates, error_if_not_exists)?;
        if deleted > 0 {
            self.set_write_flag();
        };
        Ok(deleted)
    }

    /// Matches DROPSTORE - Drops a store if exist, else returns an error
    #[tracing::instrument(skip(self))]
    pub(crate) fn drop_store(
        &self,
        store_name: StoreName,
        error_if_not_exists: bool,
    ) -> Result<usize, ServerError> {
        let pinned = self.stores.pin();
        let removed = pinned.remove(&store_name).is_some();
        if !removed && error_if_not_exists {
            return Err(ServerError::StoreNotFound(store_name));
        }
        let removed = if !removed {
            0
        } else {
            self.set_write_flag();
            1
        };
        Ok(removed)
    }
}

/// A Store is a single database containing multiple N*1 arrays where N is the dimension of the
/// store to which all arrays must conform
#[derive(Debug, Serialize, Deserialize)]
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

    #[tracing::instrument(skip(self))]
    fn drop_predicates(
        &self,
        predicates: Vec<MetadataKey>,
        error_if_not_exists: bool,
    ) -> Result<usize, ServerError> {
        self.predicate_indices
            .remove_predicates(predicates, error_if_not_exists)
    }

    #[tracing::instrument(skip_all)]
    fn delete(&self, keys: impl Iterator<Item = StoreKeyId>) -> usize {
        let keys: Vec<StoreKeyId> = keys.collect();
        let pinned = self.id_to_value.pin();
        let removed = keys.iter().flat_map(|k| pinned.remove(k));
        self.predicate_indices.remove_store_keys(&keys);
        removed.count()
    }

    /// filters input dimension to make sure it matches store dimension
    #[tracing::instrument(skip(self))]
    fn filter_dimension(&self, input: Vec<StoreKey>) -> Result<Vec<StoreKey>, ServerError> {
        input
            .into_iter()
            .map(|key| {
                let store_dimension = self.dimension.get();
                let input_dimension = key.dimension();
                if input_dimension != store_dimension {
                    return Err(ServerError::StoreDimensionMismatch {
                        store_dimension,
                        input_dimension,
                    });
                }
                Ok(key)
            })
            .collect()
    }

    /// Deletes a bunch of store keys from the store
    #[tracing::instrument(skip(self))]
    fn delete_keys(&self, del: Vec<StoreKey>) -> Result<usize, ServerError> {
        if del.is_empty() {
            return Ok(0);
        }
        let keys = self.filter_dimension(del)?;
        Ok(self.delete(keys.iter().map(From::from)))
    }

    /// Deletes a bunch of store keys from the store matching a specific predicate
    #[tracing::instrument(skip(self))]
    fn delete_matches(&self, condition: &PredicateCondition) -> Result<usize, ServerError> {
        let matches = self.predicate_indices.matches(condition)?.into_iter();
        Ok(self.delete(matches))
    }

    /// Gets a bunch of store keys from the store
    #[tracing::instrument(skip(self))]
    fn get_keys(&self, val: Vec<StoreKey>) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        if val.is_empty() {
            return Ok(vec![]);
        }
        // return error if dimensions do not match
        let keys = self.filter_dimension(val)?;
        Ok(self.get(keys.iter().map(From::from)))
    }

    /// Gets a bunch of store entries that matches a predicate condition
    #[tracing::instrument(skip(self))]
    fn get_matches(
        &self,
        condition: &PredicateCondition,
    ) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        let matches = self.predicate_indices.matches(condition)?.into_iter();
        Ok(self.get(matches))
    }

    #[tracing::instrument(skip_all)]
    fn get(&self, keys: impl Iterator<Item = StoreKeyId>) -> Vec<(StoreKey, StoreValue)> {
        let pinned = self.id_to_value.pin();
        keys.flat_map(|k| pinned.get(&k).cloned()).collect()
    }

    #[tracing::instrument(skip(self))]
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
    #[tracing::instrument(skip(self))]
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
            .map(check_bounds)
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

    #[tracing::instrument(skip(self))]
    fn create_index(&self, requested_predicates: Vec<MetadataKey>) -> usize {
        let current_predicates = self.predicate_indices.current_predicates();
        let new_predicates: Vec<_> = StdHashSet::from_iter(requested_predicates)
            .difference(&current_predicates)
            .cloned()
            .collect();
        let new_predicates_len = new_predicates.len();
        if !new_predicates.is_empty() {
            // get all the values and reindex
            let values = self
                .get_all()
                .into_iter()
                .map(|(k, v)| (StoreKeyId::from(&k), v))
                .collect();
            self.predicate_indices
                .add_predicates(new_predicates, Some(values));
        };
        new_predicates_len
    }

    /// Returns the number of key value pairs in the store
    #[tracing::instrument(skip(self))]
    fn len(&self) -> usize {
        self.id_to_value.pin().len()
    }

    /// TODO: Fix nested calculation of sizes using size_of_val
    #[tracing::instrument(skip(self))]
    fn size(&self) -> usize {
        size_of_val(&self)
            + size_of_val(&self.dimension)
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
    use crate::tests::*;
    use pretty_assertions::assert_eq;
    use std::num::NonZeroUsize;

    use super::*;
    use ahnlich_types::metadata::MetadataKey;
    use ahnlich_types::metadata::MetadataValue;
    use ahnlich_types::predicate::Predicate;
    use ndarray::array;
    use ndarray::Array1;
    use std::collections::HashMap as StdHashMap;

    #[test]
    fn test_compute_store_key_id_empty_vector() {
        let array: Array1<f32> = Array1::zeros(0);
        let store_key: StoreKeyId = (&StoreKey(array)).into();
        assert_eq!(
            store_key,
            StoreKeyId("af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262".into())
        );
    }

    #[test]
    fn test_compute_store_key_id_single_element_array() {
        let array = array![1.23];
        let store_key: StoreKeyId = (&StoreKey(array)).into();
        assert_eq!(
            store_key,
            StoreKeyId("ae69ac20168542c9058847862a41c0f24ecd5a935dfabb640bed7d591dd48ae8".into())
        );
    }

    #[test]
    fn test_compute_store_key_id_multiple_elements_array() {
        let array = array![1.22, 2.11, 3.22, 3.11];
        let store_key: StoreKeyId = (&StoreKey(array)).into();
        assert_eq!(
            store_key,
            StoreKeyId("c8d293fab65705ee27956818a9b02139fa002b71e4cd416fea055344a907db67".into())
        );
    }

    fn create_store_handler_no_loom(
        predicates: Vec<MetadataKey>,
        even_dimensions: Option<usize>,
        odd_dimensions: Option<usize>,
    ) -> Arc<StoreHandler> {
        let write_flag = Arc::new(AtomicBool::new(false));
        let handler = Arc::new(StoreHandler::new(write_flag));
        let handles = (0..3).map(|i| {
            let predicates = predicates.clone();
            let shared_handler = handler.clone();
            let handle = std::thread::spawn(move || {
                let (store_name, size) = if i % 2 == 0 {
                    ("Even", even_dimensions.unwrap_or(5))
                } else {
                    ("Odd", odd_dimensions.unwrap_or(3))
                };
                shared_handler.create_store(
                    StoreName(store_name.to_string()),
                    NonZeroUsize::new(size).unwrap(),
                    predicates,
                    true,
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
        let write_flag = Arc::new(AtomicBool::new(false));
        let handler = Arc::new(StoreHandler::new(write_flag));
        let handles = (0..3).map(|i| {
            let predicates = predicates.clone();
            let shared_handler = handler.clone();
            let handle = std::thread::spawn(move || {
                let (store_name, size) = if i % 2 == 0 { ("Even", 5) } else { ("Odd", 3) };
                shared_handler.create_store(
                    StoreName(store_name.to_string()),
                    NonZeroUsize::new(size).unwrap(),
                    predicates,
                    true,
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
    }

    #[test]
    fn test_create_and_set_in_store_fails() {
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
    }

    #[test]
    fn test_create_and_set_in_store_passes() {
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
    }

    #[test]
    fn test_add_index_in_store() {
        let handler =
            create_store_handler_no_loom(vec![MetadataKey::new("author".into())], None, None);
        let even_store = StoreName("Even".into());
        let input_arr_1 = array![0.1, 0.2, 0.3, 0.0, 0.0];
        let input_arr_2 = array![0.2, 0.3, 0.4, 0.0, 0.0];
        let input_arr_3 = array![0.3, 0.4, 0.4, 0.0, 0.0];
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey(input_arr_1.clone()),
                    StdHashMap::from_iter(vec![
                        (
                            MetadataKey::new("author".into()),
                            MetadataValue::new("Lex Luthor".into()),
                        ),
                        (
                            MetadataKey::new("planet".into()),
                            MetadataValue::new("earth".into()),
                        ),
                    ]),
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey(input_arr_2.clone()),
                    StdHashMap::from_iter(vec![
                        (
                            MetadataKey::new("author".into()),
                            MetadataValue::new("Clark Kent".into()),
                        ),
                        (
                            MetadataKey::new("planet".into()),
                            MetadataValue::new("krypton".into()),
                        ),
                    ]),
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey(input_arr_3.clone()),
                    StdHashMap::from_iter(vec![
                        (
                            MetadataKey::new("author".into()),
                            MetadataValue::new("General Zod".into()),
                        ),
                        (
                            MetadataKey::new("planet".into()),
                            MetadataValue::new("krypton".into()),
                        ),
                    ]),
                )],
            )
            .unwrap();
        let condition = &PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("author".into()),
            value: MetadataValue::new("Lex Luthor".into()),
        });
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 1);
        let condition = &PredicateCondition::Value(Predicate::NotEquals {
            key: MetadataKey::new("author".into()),
            value: MetadataValue::new("Lex Luthor".into()),
        });
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 2);
        let condition = &PredicateCondition::Value(Predicate::NotEquals {
            key: MetadataKey::new("author".into()),
            value: MetadataValue::new("Lex Luthor".into()),
        })
        .or(PredicateCondition::Value(Predicate::NotEquals {
            key: MetadataKey::new("planet".into()),
            value: MetadataValue::new("earth".into()),
        }));
        let res = handler.get_pred_in_store(&even_store, &condition);
        assert_eq!(
            res.unwrap_err(),
            ServerError::PredicateNotFound(MetadataKey::new("planet".into()))
        );
        handler
            .create_index(
                &even_store,
                vec![
                    MetadataKey::new("author".into()),
                    MetadataKey::new("planet".into()),
                ],
            )
            .unwrap();
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn test_get_key_in_store() {
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
    }

    #[test]
    fn test_get_pred_in_store() {
        let handler =
            create_store_handler_no_loom(vec![MetadataKey::new("rank".into())], None, None);
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
        let condition = &PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("rank".into()),
            value: MetadataValue::new("Hokage".into()),
        });
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert!(res.is_empty());
        let condition = &PredicateCondition::Value(Predicate::NotEquals {
            key: MetadataKey::new("rank".into()),
            value: MetadataValue::new("Hokage".into()),
        });
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 2);
        let condition = &PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("rank".into()),
            value: MetadataValue::new("Joinin".into()),
        });
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_get_store_info() {
        let handler =
            create_store_handler_no_loom(vec![MetadataKey::new("rank".into())], None, None);
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
                    size_in_bytes: 2104,
                },
                StoreInfo {
                    name: even_store,
                    len: 0,
                    size_in_bytes: 1736,
                },
            ])
        )
    }

    #[test]
    fn test_get_sim_in_store_with_predicate() {
        let vectors = word_to_vector();

        let input_arr_1 = vectors.get(MOST_SIMILAR[0]).unwrap();
        let input_arr_2 = vectors.get(MOST_SIMILAR[1]).unwrap();
        let input_arr_3 = vectors.get(MOST_SIMILAR[2]).unwrap();

        let handler = create_store_handler_no_loom(
            vec![MetadataKey::new("rank".into())],
            Some(input_arr_1.0.len()),
            Some(input_arr_1.0.len()),
        );
        let even_store = StoreName("Even".into());
        handler
            .set_in_store(
                &even_store,
                vec![(
                    input_arr_1.clone(),
                    StdHashMap::from_iter(vec![(
                        MetadataKey::new("rank".into()),
                        MetadataValue::new("Chunin".into()),
                    )]),
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    input_arr_2.clone(),
                    StdHashMap::from_iter(vec![(
                        MetadataKey::new("rank".into()),
                        MetadataValue::new("Chunin".into()),
                    )]),
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    input_arr_3.clone(),
                    StdHashMap::from_iter(vec![(
                        MetadataKey::new("rank".into()),
                        MetadataValue::new("Genin".into()),
                    )]),
                )],
            )
            .unwrap();
        let condition = &PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("rank".into()),
            value: MetadataValue::new("Chunin".into()),
        });
        let search_input = StoreKey(vectors.get(SEACH_TEXT).unwrap().0.clone());
        let algorithm = Algorithm::CosineSimilarity;

        let closest_n = NonZeroUsize::new(3).unwrap();
        let res = handler
            .get_sim_in_store(
                &even_store,
                search_input.clone(),
                closest_n,
                algorithm,
                Some(condition.clone()),
            )
            .unwrap();
        assert_eq!(res.len(), 2);

        let closest_n = NonZeroUsize::new(1).unwrap();
        let res = handler
            .get_sim_in_store(
                &even_store,
                search_input.clone(),
                closest_n,
                algorithm,
                None,
            )
            .unwrap();
        assert_eq!(res.len(), 1);
        assert!(res[0].0 == *vectors.get(MOST_SIMILAR[0]).unwrap());

        let condition = &PredicateCondition::Value(Predicate::NotEquals {
            key: MetadataKey::new("rank".into()),
            value: MetadataValue::new("Chunin".into()),
        });
        let closest_n = NonZeroUsize::new(3).unwrap();
        let res = handler
            .get_sim_in_store(
                &even_store,
                search_input.clone(),
                closest_n,
                algorithm,
                Some(condition.clone()),
            )
            .unwrap();
        assert_eq!(res.len(), 1);

        // Add more items storekeys into the store for processing.
        //
        let meta_data_key = MetadataKey::new("english".into());
        let store_values = vectors
            .iter()
            .filter(|(sentence, _)| SEACH_TEXT != *sentence)
            .map(|(sentence, store_key)| {
                let value: StdHashMap<MetadataKey, MetadataValue> = StdHashMap::from_iter(vec![(
                    meta_data_key.clone(),
                    MetadataValue::new(sentence.into()),
                )]);
                (store_key.clone(), value)
            })
            .collect();
        handler.set_in_store(&even_store, store_values).unwrap();
        let res = handler
            .get_sim_in_store(
                &even_store,
                search_input.clone(),
                closest_n,
                Algorithm::EuclideanDistance,
                None,
            )
            .unwrap();

        assert_eq!(res.len(), 3);

        assert_eq!(
            res[0].1.get(&meta_data_key).cloned().unwrap(),
            MetadataValue::new(MOST_SIMILAR[0].into())
        );
        assert_eq!(
            res[1].1.get(&meta_data_key).cloned().unwrap(),
            MetadataValue::new(MOST_SIMILAR[1].into())
        );
        assert_eq!(
            res[2].1.get(&meta_data_key).cloned().unwrap(),
            MetadataValue::new(MOST_SIMILAR[2].into())
        );
    }
}
