use crate::errors::ServerError;
use rayon::prelude::*;

use super::super::algorithm::non_linear::NonLinearAlgorithmIndices;
use super::super::algorithm::{AlgorithmByType, FindSimilarN};
use super::predicate::PredicateIndices;
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use ahnlich_types::db::server::StoreInfo;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::keyval::{StoreKey, StoreValue};
use ahnlich_types::predicates::{
    self, predicate::Kind as PredicateKind, Predicate, PredicateCondition,
};
use ahnlich_types::shared::info::StoreUpsert;
use ahnlich_types::similarity::Similarity;
use papaya::HashMap as ConcurrentHashMap;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet as StdHashSet;
use std::mem::size_of_val;
use std::num::NonZeroUsize;
use std::sync::atomic::Ordering;
use std::sync::atomic::{AtomicBool, AtomicUsize};
use std::sync::Arc;
use utils::persistence::AhnlichPersistenceUtils;
/// A hash of Store key, this is more preferable when passing around references as arrays can be
/// potentially larger
/// We should be only able to generate a store key id from a 1D vector except during tests

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
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
        for element in value.key.iter() {
            let bytes = element.to_ne_bytes();
            hasher.update(&bytes);
        }
        let result = hasher.finalize();
        Self(format!("{result}"))
    }
}

/// Contains all the stores that have been created in memory
#[derive(Debug)]
pub struct StoreHandler {
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    stores: Stores,
    pub write_flag: Arc<AtomicBool>,
}

impl AhnlichPersistenceUtils for StoreHandler {
    type PersistenceObject = Stores;

    #[tracing::instrument(skip_all)]
    fn write_flag(&self) -> Arc<AtomicBool> {
        self.write_flag.clone()
    }

    #[tracing::instrument(skip(self))]
    fn get_snapshot(&self) -> Self::PersistenceObject {
        self.stores.clone()
    }
}

pub type Stores = Arc<ConcurrentHashMap<StoreName, Arc<Store>>>;

impl StoreHandler {
    pub fn new(write_flag: Arc<AtomicBool>) -> Self {
        Self {
            stores: Arc::new(ConcurrentHashMap::new()),
            write_flag,
        }
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn get_stores(&self) -> Stores {
        self.stores.clone()
    }

    #[cfg(test)]
    pub fn write_flag(&self) -> Arc<AtomicBool> {
        self.write_flag.clone()
    }

    #[tracing::instrument(skip(self))]
    fn set_write_flag(&self) {
        let _ = self
            .write_flag
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    }

    #[tracing::instrument(skip(self))]
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

    /// Matches CREATEPREDINDEX - reindexes a store with some predicate values
    #[tracing::instrument(skip(self))]
    pub(crate) fn create_pred_index(
        &self,
        store_name: &StoreName,
        // TODO: create grpc datatype for metadata key
        predicates: Vec<String>,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        let created_predicates = store.create_pred_index(predicates);
        if created_predicates > 0 {
            self.set_write_flag()
        }
        Ok(created_predicates)
    }

    /// Matches CREATENONLINEARALGORITHMINDEX - reindexes a store with some non linear algorithms
    #[tracing::instrument(skip(self))]
    pub(crate) fn create_non_linear_algorithm_index(
        &self,
        store_name: &StoreName,
        non_linear_indices: StdHashSet<NonLinearAlgorithm>,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        let created_predicates = store.create_non_linear_algorithm_index(non_linear_indices);
        if created_predicates > 0 {
            self.set_write_flag()
        }
        Ok(created_predicates)
    }

    /// Matches DELKEY - removes keys from a store
    #[tracing::instrument(skip(self, keys), fields(keys_length=keys.len()))]
    pub(crate) fn del_key_in_store(
        &self,
        store_name: &StoreName,
        keys: Vec<StoreKey>,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        let deleted = store.delete_keys(keys.clone())?;
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
    pub fn get_sim_in_store(
        &self,
        store_name: &StoreName,
        search_input: StoreKey,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
        condition: Option<PredicateCondition>,
    ) -> Result<Vec<(StoreKey, StoreValue, Similarity)>, ServerError> {
        let store = self.get(store_name)?;
        let store_dimension = store.dimension.get();
        let input_dimension = search_input.key.len();

        if input_dimension != store_dimension {
            return Err(ServerError::StoreDimensionMismatch {
                store_dimension,
                input_dimension,
            });
        }

        let (filtered, used_all) = if let Some(ref condition) = condition {
            (store.get_matches(condition)?, false)
        } else {
            (store.get_all(), true)
        };

        // early stopping: predicate filters everything out so no need to search
        if filtered.is_empty() {
            return Ok(vec![]);
        }

        let filtered_iter = filtered.par_iter().map(|(key, _)| key);

        let algorithm_by_type: AlgorithmByType = algorithm.into();
        let similar_result = match algorithm_by_type {
            AlgorithmByType::Linear(linear_algo) => {
                linear_algo.find_similar_n(&search_input, filtered_iter, used_all, closest_n)
            }
            AlgorithmByType::NonLinear(non_linear_algo) => {
                let non_linear_indices = store.non_linear_indices.algorithm_to_index.pin();
                let non_linear_index_with_algo = non_linear_indices
                    .get(&non_linear_algo)
                    .ok_or(ServerError::NonLinearIndexNotFound(non_linear_algo))?;
                non_linear_index_with_algo.find_similar_n(
                    &search_input,
                    filtered_iter,
                    used_all,
                    closest_n,
                )
            }
        };

        let mut keys_to_value_map: StdHashMap<StoreKeyId, &StoreValue> = StdHashMap::from_iter(
            filtered
                .iter()
                .map(|(store_key, store_value)| (StoreKeyId::from(store_key), store_value)),
        );

        Ok(similar_result
            .into_iter()
            .flat_map(|(store_key, similarity)| {
                keys_to_value_map
                    .remove(&StoreKeyId::from(&store_key))
                    .map(|value| (store_key, value.clone(), Similarity { value: similarity }))
            })
            .collect())
    }

    /// Matches GETPRED - gets all matching predicates from a store
    #[tracing::instrument(skip(self, condition))]
    pub(crate) fn get_pred_in_store(
        &self,
        store_name: &StoreName,
        condition: &PredicateCondition,
    ) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        let store = self.get(store_name)?;
        store.get_matches(condition)
    }

    /// Matches GETKEY - gets all keys matching the inputs
    #[tracing::instrument(skip(self, keys), fields(key_length=keys.len()))]
    pub(crate) fn get_key_in_store(
        &self,
        store_name: &StoreName,
        keys: Vec<StoreKey>,
    ) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        let store = self.get(store_name)?;
        store.get_keys(keys)
    }

    /// Matches SET - adds new entries into a particular store
    #[tracing::instrument(skip(self, new), fields(entries_length=new.len()))]
    pub fn set_in_store(
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
                name: store_name.clone().value,
                len: store.len() as u64,
                size_in_bytes: store.size() as u64,
            })
            .collect()
    }

    /// Matches CREATESTORE - Creates a store if not exist, else return an error
    #[tracing::instrument(skip(self))]
    pub fn create_store(
        &self,
        store_name: StoreName,
        dimension: NonZeroUsize,
        // FIXME: update metadata key with grpc type key
        predicates: Vec<String>,
        non_linear_indices: StdHashSet<NonLinearAlgorithm>,
        error_if_exists: bool,
    ) -> Result<(), ServerError> {
        if self
            .stores
            .try_insert(
                store_name.clone(),
                Arc::new(Store::create(dimension, predicates, non_linear_indices)),
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

    /// Matches DROPPREDINDEX - Drops predicate index if exists, else returns an error
    #[tracing::instrument(skip(self))]
    pub(crate) fn drop_pred_index_in_store(
        &self,
        store_name: &StoreName,
        predicates: Vec<String>,
        error_if_not_exists: bool,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        let deleted = store.drop_predicates(predicates, error_if_not_exists)?;
        if deleted > 0 {
            self.set_write_flag();
        };
        Ok(deleted)
    }

    /// Matches DROPNONLINEARALGORITHMINDEX - Drops non linear algorithm if it exists, else returns
    /// an error
    #[tracing::instrument(skip(self))]
    pub(crate) fn drop_non_linear_algorithm_index(
        &self,
        store_name: &StoreName,
        non_linear_indices: StdHashSet<NonLinearAlgorithm>,
        error_if_not_exists: bool,
    ) -> Result<usize, ServerError> {
        let store = self.get(store_name)?;
        let deleted = store
            .non_linear_indices
            .remove_indices(non_linear_indices, error_if_not_exists)?;
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
pub struct Store {
    dimension: NonZeroUsize,
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    id_to_value: ConcurrentHashMap<StoreKeyId, (StoreKey, StoreValue)>,
    /// Indices to filter for the store
    predicate_indices: PredicateIndices,
    /// Non linear Indices
    non_linear_indices: NonLinearAlgorithmIndices,
}

impl Store {
    /// Creates a new empty store
    pub(super) fn create(
        dimension: NonZeroUsize,
        predicates: Vec<String>,
        non_linear_indices: StdHashSet<NonLinearAlgorithm>,
    ) -> Self {
        Self {
            dimension,
            id_to_value: ConcurrentHashMap::new(),
            predicate_indices: PredicateIndices::init(predicates),
            non_linear_indices: NonLinearAlgorithmIndices::create(non_linear_indices, dimension),
        }
    }

    #[tracing::instrument(skip(self))]
    fn drop_predicates(
        &self,
        predicates: Vec<String>,
        error_if_not_exists: bool,
    ) -> Result<usize, ServerError> {
        self.predicate_indices
            .remove_predicates(predicates, error_if_not_exists)
    }

    #[tracing::instrument(skip_all)]
    fn delete(&self, keys: impl Iterator<Item = StoreKeyId>) -> usize {
        let keys: Vec<StoreKeyId> = keys.collect();
        let pinned = self.id_to_value.pin();
        let removed = keys
            .iter()
            .flat_map(|k| pinned.remove(k))
            .map(|(k, _)| k.key.clone())
            .collect::<Vec<_>>();
        self.predicate_indices.remove_store_keys(&keys);
        self.non_linear_indices.delete(&removed);
        removed.len()
    }

    /// filters input dimension to make sure it matches store dimension
    #[tracing::instrument(skip(self, input), fields(input_length=input.len()))]
    fn filter_dimension(&self, input: Vec<StoreKey>) -> Result<Vec<StoreKey>, ServerError> {
        input
            .into_par_iter()
            .map(|key| {
                let store_dimension = self.dimension.get();
                let input_dimension = key.key.len();
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
    #[tracing::instrument(skip(self, del), fields(key_length=del.len()))]
    fn delete_keys(&self, del: Vec<StoreKey>) -> Result<usize, ServerError> {
        if del.is_empty() {
            return Ok(0);
        }
        let keys = self.filter_dimension(del)?;
        let res = self.delete(keys.iter().map(From::from));
        Ok(res)
    }

    /// Deletes a bunch of store keys from the store matching a specific predicate
    #[tracing::instrument(skip(self))]
    fn delete_matches(&self, condition: &PredicateCondition) -> Result<usize, ServerError> {
        let matches = self.predicate_indices.matches(condition, self)?.into_iter();
        Ok(self.delete(matches))
    }

    /// Gets a bunch of store keys from the store
    #[tracing::instrument(skip(self, val), fields(key_length=val.len()))]
    fn get_keys(&self, val: Vec<StoreKey>) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        if val.is_empty() {
            return Ok(vec![]);
        }
        // return error if dimensions do not match
        let keys = self.filter_dimension(val)?;
        Ok(self.get(keys.iter().map(From::from)))
    }

    /// Gets a bunch of store entries that matches a predicate condition
    #[tracing::instrument(skip_all)]
    fn get_matches(
        &self,
        condition: &PredicateCondition,
    ) -> Result<Vec<(StoreKey, StoreValue)>, ServerError> {
        let matches = self.predicate_indices.matches(condition, self)?.into_iter();
        Ok(self.get(matches))
    }

    /// Used whenever there is no found predicate and so we search directly within store
    #[tracing::instrument(skip(self))]
    pub(super) fn get_match_without_predicate(
        &self,
        predicate: &Predicate,
    ) -> Result<StdHashSet<StoreKeyId>, ServerError> {
        let store_val_pinned = self.id_to_value.pin();
        let res = match &predicate.kind {
            Some(PredicateKind::Equals(predicates::Equals { key, value })) => store_val_pinned
                .into_iter()
                .filter(|(_, (_, store_value))| {
                    let metadata_value = store_value.value.get(key);

                    metadata_value.eq(&value.as_ref())
                })
                .map(|(k, _)| k.clone())
                .collect(),
            Some(PredicateKind::NotEquals(predicates::NotEquals { key, value })) => {
                store_val_pinned
                    .into_iter()
                    .filter(|(_, (_, store_value))| {
                        let metdata_value = store_value.value.get(key);
                        !metdata_value.eq(&value.as_ref())
                    })
                    .map(|(k, _)| k.clone())
                    .collect()
            }
            Some(PredicateKind::In(predicates::In { key, values })) => store_val_pinned
                .into_iter()
                .filter(|(_, (_, store_value))| {
                    store_value
                        .value
                        .get(key)
                        .map(|v| values.contains(v))
                        .unwrap_or(false)
                })
                .map(|(k, _)| k.clone())
                .collect(),
            Some(PredicateKind::NotIn(predicates::NotIn { key, values })) => store_val_pinned
                .into_iter()
                .filter(|(_, (_, store_value))| {
                    store_value
                        .value
                        .get(key)
                        .map(|v| !values.contains(v))
                        .unwrap_or(true)
                })
                .map(|(k, _)| k.clone())
                .collect(),

            None => unreachable!(),
        };
        Ok(res)
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
    #[tracing::instrument(skip(self, new), fields(entry_length=new.len()))]
    fn add(&self, new: Vec<(StoreKey, StoreValue)>) -> Result<StoreUpsert, ServerError> {
        if new.is_empty() {
            return Ok(StoreUpsert {
                inserted: 0,
                updated: 0,
            });
        }
        let store_dimension: usize = self.dimension.into();
        let check_bounds = |(store_key, store_val): &(StoreKey, StoreValue)| -> Result<(StoreKeyId, (StoreKey, StoreValue)), ServerError> {
            let input_dimension = store_key.key.len();
            if input_dimension != store_dimension {
                Err(ServerError::StoreDimensionMismatch { store_dimension, input_dimension  })
            } else {
                Ok(((store_key).into(), (store_key.to_owned(), store_val.to_owned())))
            }
        };
        let res: Vec<(StoreKeyId, (StoreKey, StoreValue))> =
            new.par_iter().map(check_bounds).collect::<Result<_, _>>()?;
        let predicate_insert = res
            .par_iter()
            .map(|(k, (_, v))| (k.clone(), v.clone()))
            .collect();
        let inserted = AtomicUsize::new(0);
        let updated = AtomicUsize::new(0);
        let inserted_keys = res
            .into_par_iter()
            .flat_map_iter(|(k, v)| {
                let pinned = self.id_to_value.pin();
                if pinned.insert(k, v.clone()).is_some() {
                    updated.fetch_add(1, Ordering::SeqCst);
                } else {
                    inserted.fetch_add(1, Ordering::SeqCst);
                    return Some(v.0.key);
                }
                None
            })
            .collect();
        self.predicate_indices.add(predicate_insert);
        if !self.non_linear_indices.is_empty() {
            self.non_linear_indices.insert(inserted_keys);
        }
        Ok(StoreUpsert {
            inserted: inserted.into_inner() as u64,
            updated: updated.into_inner() as u64,
        })
    }

    #[tracing::instrument(skip(self))]
    fn create_pred_index(&self, requested_predicates: Vec<String>) -> usize {
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

    #[tracing::instrument(skip(self))]
    fn create_non_linear_algorithm_index(
        &self,
        non_linear_indices: StdHashSet<NonLinearAlgorithm>,
    ) -> usize {
        let current_keys = self.non_linear_indices.current_keys();
        let new_predicates: StdHashSet<_> = non_linear_indices
            .difference(&current_keys)
            .copied()
            .collect();
        let new_predicates_len = new_predicates.len();
        if !new_predicates.is_empty() {
            // get all the values and reindex
            let values: Vec<_> = self.get_all().into_iter().map(|(k, _)| k.key).collect();
            self.non_linear_indices
                .insert_indices(new_predicates, &values, self.dimension);
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
                            .value
                            .iter()
                            .map(|(inner_k, inner_val)| {
                                size_of_val(inner_k) + size_of_val(inner_val)
                            })
                            .sum::<usize>()
                })
                .sum::<usize>()
            + self.predicate_indices.size()
            + self.non_linear_indices.size()
    }
}

#[cfg(test)]
mod tests {
    use crate::tests::*;
    use pretty_assertions::assert_eq;
    use std::num::NonZeroUsize;

    use super::*;
    use ahnlich_types::metadata::MetadataValue;
    use ahnlich_types::predicates::{
        self, predicate::Kind as PredicateKind,
        predicate_condition::Kind as PredicateConditionKind, Predicate, PredicateCondition,
    };
    use std::collections::HashMap as StdHashMap;

    #[test]
    fn test_compute_store_key_id_empty_vector() {
        let array: Vec<f32> = vec![];
        let store_key: StoreKeyId = (&StoreKey { key: array }).into();
        assert_eq!(
            store_key,
            StoreKeyId("af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262".into())
        );
    }

    #[test]
    fn test_compute_store_key_id_single_element_array() {
        let array = vec![1.23];
        let store_key: StoreKeyId = (&StoreKey { key: array }).into();
        assert_eq!(
            store_key,
            StoreKeyId("ae69ac20168542c9058847862a41c0f24ecd5a935dfabb640bed7d591dd48ae8".into())
        );
    }

    #[test]
    fn test_compute_store_key_id_multiple_elements_array() {
        let array = vec![1.22, 2.11, 3.22, 3.11];
        let store_key: StoreKeyId = (&StoreKey { key: array }).into();
        assert_eq!(
            store_key,
            StoreKeyId("c8d293fab65705ee27956818a9b02139fa002b71e4cd416fea055344a907db67".into())
        );
    }

    fn create_store_handler_no_loom(
        predicates: Vec<String>,
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
                    StoreName {
                        value: store_name.to_string(),
                    },
                    NonZeroUsize::new(size).unwrap(),
                    predicates,
                    StdHashSet::new(),
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
        predicates: Vec<String>,
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
                    StoreName {
                        value: store_name.to_string(),
                    },
                    NonZeroUsize::new(size).unwrap(),
                    predicates,
                    StdHashSet::new(),
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
            vec![Err(ServerError::StoreAlreadyExists(StoreName {
                value: "Even".to_string()
            }))]
        );
        // test out a store that does not exist
        let fake_store = StoreName {
            value: "Random".to_string(),
        };
        assert_eq!(
            handler.get(&fake_store).unwrap_err(),
            ServerError::StoreNotFound(fake_store)
        );
    }

    #[test]
    fn test_create_and_set_in_store_fails() {
        let (handler, _oks, _errs) = create_store_handler(vec![]);
        let even_store = StoreName {
            value: "Even".into(),
        };
        let fake_store = StoreName {
            value: "Fake".into(),
        };
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
                        StoreKey {
                            key: vec![0.33, 0.44, 0.5]
                        },
                        StoreValue {
                            value: StdHashMap::from_iter(vec![(
                                "author".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "Vincent".to_string()
                                        )
                                    )
                                },
                            ),])
                        }
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
        let odd_store = StoreName {
            value: "Odd".into(),
        };
        let input_arr = vec![0.1, 0.2, 0.3];
        let ret = handler
            .set_in_store(
                &odd_store,
                vec![(
                    StoreKey {
                        key: input_arr.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "author".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Lex Luthor".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
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
                    StoreKey {
                        key: input_arr.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "author".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Clark Kent".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
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
        let handler = create_store_handler_no_loom(vec!["author".to_string()], None, None);
        let even_store = StoreName {
            value: "Even".to_string(),
        };
        let input_arr_1 = vec![0.1, 0.2, 0.3, 0.0, 0.0];
        let input_arr_2 = vec![0.2, 0.3, 0.4, 0.0, 0.0];
        let input_arr_3 = vec![0.3, 0.4, 0.4, 0.0, 0.0];
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey {
                        key: input_arr_1.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![
                            (
                                "author".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "Lex Luthor".to_string(),
                                        ),
                                    ),
                                },
                            ),
                            (
                                "planet".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "earth".to_string(),
                                        ),
                                    ),
                                },
                            ),
                        ]),
                    },
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey {
                        key: input_arr_2.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![
                            (
                                "author".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "Clark Kent Luthor".to_string(),
                                        ),
                                    ),
                                },
                            ),
                            (
                                "planet".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "krypton".to_string(),
                                        ),
                                    ),
                                },
                            ),
                        ]),
                    },
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey {
                        key: input_arr_3.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![
                            (
                                "author".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "General Zof".to_string(),
                                        ),
                                    ),
                                },
                            ),
                            (
                                "planet".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "krypton".to_string(),
                                        ),
                                    ),
                                },
                            ),
                        ]),
                    },
                )],
            )
            .unwrap();

        let condition = &PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::Equals(predicates::Equals {
                    key: "author".into(),
                    value: Some(MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "Lex Luthor".to_string(),
                        )),
                    }),
                })),
            })),
        };
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 1);

        let condition = &PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                    key: "author".into(),
                    value: Some(MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "Lex Luthor".to_string(),
                        )),
                    }),
                })),
            })),
        };

        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 2);

        let condition = &PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                    key: "author".into(),
                    value: Some(MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "Lex Luthor".to_string(),
                        )),
                    }),
                })),
            })),
        }
        .or(PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                    key: "planet".into(),
                    value: Some(MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "earth".to_string(),
                        )),
                    }),
                })),
            })),
        });
        let res = handler.get_pred_in_store(&even_store, &condition);
        assert_eq!(res.unwrap().len(), 2);
        handler
            .create_pred_index(&even_store, vec!["author".into(), "planet".into()])
            .unwrap();
        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 2);
    }

    #[test]
    fn test_get_key_in_store() {
        let (handler, _oks, _errs) = create_store_handler(vec![]);
        let odd_store = StoreName {
            value: "Odd".into(),
        };
        let fake_store = StoreName {
            value: "Fakest".into(),
        };
        let input_arr_1 = vec![0.1, 0.2, 0.3];
        let input_arr_2 = vec![0.2, 0.3, 0.4];
        handler
            .set_in_store(
                &odd_store,
                vec![(
                    StoreKey {
                        key: input_arr_1.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "author".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Lex Luthor".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &odd_store,
                vec![(
                    StoreKey {
                        key: input_arr_2.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "author".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Clark Kent".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
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
                vec![StoreKey { key: input_arr_1 }, StoreKey { key: input_arr_2 }],
            )
            .unwrap();
        assert_eq!(ret.len(), 2);
        assert_eq!(
            ret[0].1.value.get("author".into()).cloned().unwrap(),
            MetadataValue {
                value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                    "Lex Luthor".to_string(),
                ),),
            },
        );
        assert_eq!(
            ret[1].1.value.get("author".into()).cloned().unwrap(),
            MetadataValue {
                value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                    "Clark Kent".to_string(),
                ),),
            },
        );
    }

    #[test]
    fn test_get_pred_in_store() {
        let handler = create_store_handler_no_loom(vec!["rank".into()], None, None);
        let even_store = StoreName {
            value: "Even".into(),
        };
        let input_arr_1 = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let input_arr_2 = vec![0.2, 0.3, 0.4, 0.5, 0.6];
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey {
                        key: input_arr_1.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "rank".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Joinin".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    StoreKey {
                        key: input_arr_2.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "rank".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Genin".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )],
            )
            .unwrap();

        let condition = &PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::Equals(predicates::Equals {
                    key: "rank".into(),
                    value: Some(MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "Hokage".to_string(),
                        )),
                    }),
                })),
            })),
        };

        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert!(res.is_empty());

        let condition = &PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                    key: "rank".into(),
                    value: Some(MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "Hokage".to_string(),
                        )),
                    }),
                })),
            })),
        };

        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 2);

        let condition = &PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::Equals(predicates::Equals {
                    key: "rank".into(),
                    value: Some(MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "Joinin".to_string(),
                        )),
                    }),
                })),
            })),
        };

        let res = handler.get_pred_in_store(&even_store, &condition).unwrap();
        assert_eq!(res.len(), 1);
    }

    #[test]
    fn test_get_store_info() {
        let handler = create_store_handler_no_loom(vec!["rank".into()], None, None);
        let odd_store = StoreName {
            value: "Odd".into(),
        };
        let even_store = StoreName {
            value: "Even".into(),
        };
        let input_arr_1 = vec![0.1, 0.2, 0.3];
        let input_arr_2 = vec![0.2, 0.3, 0.4];
        handler
            .set_in_store(
                &odd_store,
                vec![(
                    StoreKey {
                        key: input_arr_1.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "rank".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Joinin".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &odd_store,
                vec![(
                    StoreKey {
                        key: input_arr_2.clone(),
                    },
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "rank".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Genin".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )],
            )
            .unwrap();
        let stores = handler.list_stores();
        assert_eq!(
            stores,
            StdHashSet::from_iter([
                StoreInfo {
                    name: odd_store.value,
                    len: 2,
                    size_in_bytes: 1432,
                },
                StoreInfo {
                    name: even_store.value,
                    len: 0,
                    size_in_bytes: 1080,
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
            vec!["rank".into()],
            Some(input_arr_1.key.len()),
            Some(input_arr_1.key.len()),
        );
        let even_store = StoreName {
            value: "Even".into(),
        };
        handler
            .set_in_store(
                &even_store,
                vec![(
                    input_arr_1.clone(),
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "rank".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Chunin".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    input_arr_2.clone(),
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "rank".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Chunin".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )],
            )
            .unwrap();
        handler
            .set_in_store(
                &even_store,
                vec![(
                    input_arr_3.clone(),
                    StoreValue {
                        value: StdHashMap::from_iter(vec![(
                            "rank".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "Genin".to_string(),
                                    ),
                                ),
                            },
                        )]),
                    },
                )],
            )
            .unwrap();

        let condition = &PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::Equals(predicates::Equals {
                    key: "rank".into(),
                    value: Some(MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "Chunin".to_string(),
                        )),
                    }),
                })),
            })),
        };

        let search_input = StoreKey {
            key: vectors.get(SEACH_TEXT).unwrap().key.clone(),
        };
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

        let condition = &PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                    key: "rank".into(),
                    value: Some(MetadataValue {
                        value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                            "Chunin".to_string(),
                        )),
                    }),
                })),
            })),
        };

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
        let meta_data_key = "english".to_string();
        let store_values = vectors
            .iter()
            .filter(|(sentence, _)| SEACH_TEXT != *sentence)
            .map(|(sentence, store_key)| {
                let value = StoreValue {
                    value: StdHashMap::from_iter(vec![(
                        meta_data_key.clone(),
                        MetadataValue {
                            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                                sentence.into(),
                            )),
                        },
                    )]),
                };
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
            res[0].1.value.get(&meta_data_key).cloned().unwrap(),
            MetadataValue {
                value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                    MOST_SIMILAR[0].into()
                )),
            },
        );
        assert_eq!(
            res[1].1.value.get(&meta_data_key).cloned().unwrap(),
            MetadataValue {
                value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                    MOST_SIMILAR[1].into()
                )),
            },
        );
        assert_eq!(
            res[2].1.value.get(&meta_data_key).cloned().unwrap(),
            MetadataValue {
                value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                    MOST_SIMILAR[2].into()
                )),
            },
        );
    }
}
