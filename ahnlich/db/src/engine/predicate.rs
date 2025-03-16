use super::super::errors::ServerError;
use super::store::Store;
use super::store::StoreKeyId;
use flurry::HashMap as ConcurrentHashMap;
use flurry::HashSet as ConcurrentHashSet;
use grpc_types::keyval::StoreValue;
use grpc_types::metadata::MetadataValue;
use grpc_types::predicates::{
    predicate_condition::Kind as PredicateKind, Predicate, PredicateCondition,
};
use itertools::Itertools;
use rayon::iter::IndexedParallelIterator;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet as StdHashSet;
use std::mem::size_of_val;
use utils::parallel;

/// Predicates are essentially nested hashmaps that let us retrieve original keys that match a
/// precise value. Take the following example
///
/// {
///     "Country": {
///         "Nigeria": [StoreKeyId(1), StoreKeyId(2)],
///         "Australia": ..,
///     },
///     "Author": {
///         ...
///     }
/// }
///
/// where `allowed_predicates` = ["Country", "Author"]
///
/// It takes less time to retrieve "where country = 'Nigeria'" by traversing the nested hashmap to
/// obtain StoreKeyId(1) and StoreKeyId(2) than it would be to make a linear pass over an entire
/// Store of size N comparing their metadata "country" along the way. Given that StoreKeyId is
/// computed via blake hash, it is typically fast to compute and also of a fixed size which means
/// predicate indices don't balloon with large metadata
///
/// Whichever key is not expressly included in `allowed_predicates` goes through the linear
/// pass in order to obtain keys that satisfy the condition
type InnerPredicateIndexVal = ConcurrentHashSet<StoreKeyId>;
type InnerPredicateIndex = ConcurrentHashMap<MetadataValue, InnerPredicateIndexVal>;
type InnerPredicateIndices = ConcurrentHashMap<String, PredicateIndex>;

/// Predicate indices are all the indexes referenced by their names
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct PredicateIndices {
    inner: InnerPredicateIndices,
    /// These are the index keys that are meant to generate predicate indexes
    allowed_predicates: ConcurrentHashSet<String>,
}

impl PredicateIndices {
    #[tracing::instrument(skip(self))]
    pub(super) fn size(&self) -> usize {
        size_of_val(&self)
            + self
                .inner
                .iter(&self.inner.guard())
                .map(|(k, v)| size_of_val(k) + v.size())
                .sum::<usize>()
            + self
                .allowed_predicates
                .iter(&self.allowed_predicates.guard())
                .map(size_of_val)
                .sum::<usize>()
    }

    #[tracing::instrument]
    pub(super) fn init(allowed_predicates: Vec<String>) -> Self {
        let created = ConcurrentHashSet::new();
        for key in allowed_predicates {
            created.insert(key, &created.guard());
        }
        Self {
            inner: InnerPredicateIndices::new(),
            allowed_predicates: created,
        }
    }

    /// returns the current predicates within predicate index
    #[tracing::instrument(skip(self))]
    pub(super) fn current_predicates(&self) -> StdHashSet<String> {
        let allowed_predicates = self.allowed_predicates.pin();
        allowed_predicates.into_iter().cloned().collect()
    }

    /// Removes a store key id when it's corresponding entry in the store is removed
    #[tracing::instrument(skip(self))]
    pub(super) fn remove_store_keys(&self, remove_keys: &[StoreKeyId]) {
        let pinned = self.inner.pin();
        for (_, values) in pinned.iter() {
            values.remove_store_keys(remove_keys);
        }
    }

    /// Removes predicates from being tracked
    #[tracing::instrument(skip(self))]
    pub(super) fn remove_predicates(
        &self,
        predicates: Vec<String>,
        error_if_not_exists: bool,
    ) -> Result<usize, ServerError> {
        if predicates.is_empty() {
            return Ok(0);
        }

        let pinned_keys = self.allowed_predicates.pin();
        let pinned_predicate_values = self.inner.pin();
        // first check all predicates
        if let (true, Some(non_existing_index)) = (
            error_if_not_exists,
            predicates.iter().find(|&a| !pinned_keys.contains(a)),
        ) {
            return Err(ServerError::PredicateNotFound(non_existing_index.clone()));
        }
        let mut deleted = 0;
        for predicate in predicates {
            let removed = pinned_keys.remove(&predicate);
            pinned_predicate_values.remove(&predicate);

            if removed {
                deleted += 1;
            };
        }
        Ok(deleted)
    }

    /// This adds predicates to the allowed predicates and allows newer entries to be indexed
    /// It first checks for the predicates that do not currently exist and then attempts to add
    /// those to the underlying predicates
    #[tracing::instrument(skip(self))]
    pub(super) fn add_predicates(
        &self,
        predicates: Vec<String>,
        refresh_with_values: Option<Vec<(StoreKeyId, StoreValue)>>,
    ) {
        let pinned_keys = self.allowed_predicates.pin();
        let pinned_inner = self.inner.pin();
        // `insert` implicity adds it to allowed_predicates which is what lets us to be able to
        // search again
        let mut new_predicates = predicates
            .into_iter()
            .filter(|pred| pinned_keys.insert(pred.clone()))
            .unique()
            .peekable();
        // Only update for new predicates
        if let Some(new_values) = (new_predicates.peek().is_some())
            .then_some(refresh_with_values)
            .flatten()
        {
            for new_predicate in new_predicates {
                let val = new_values
                    .iter()
                    .flat_map(|(store_key_id, store_value)| {
                        store_value
                            .iter()
                            .filter(|(key, _)| **key == new_predicate)
                            .map(|(_, val)| (val.clone(), store_key_id.clone()))
                    })
                    .collect::<Vec<_>>();
                let pred = PredicateIndex::init(val.clone());
                if let Err(existing_predicate) = pinned_inner.try_insert(new_predicate, pred) {
                    existing_predicate.current.add(val)
                }
            }
        }
    }

    /// Adds predicates if the key is within allowed_predicates
    #[tracing::instrument(skip_all, fields(new_len = new.len()))]
    pub(super) fn add(&self, new: Vec<(StoreKeyId, StoreValue)>) {
        let iter = new
            .into_par_iter()
            .flat_map(|(store_key_id, store_value)| {
                store_value.value.into_par_iter().map(move |(key, val)| {
                    let allowed_keys = self.allowed_predicates.pin();
                    allowed_keys
                        .contains(&key)
                        .then_some((store_key_id.clone(), key, val))
                })
            })
            .flatten()
            .map(|(store_key_id, key, val)| (key, (val.to_owned(), store_key_id)))
            .fold(HashMap::new, |mut acc: HashMap<_, Vec<_>>, (k, v)| {
                acc.entry(k).or_default().push(v);
                acc
            })
            .reduce(HashMap::new, |mut acc, map| {
                for (key, mut values) in map {
                    acc.entry(key).or_default().append(&mut values);
                }
                acc
            });

        let predicate_values = self.inner.pin();
        for (key, val) in iter {
            // If there exists a predicate index as we want to update it, just add to that
            // predicate index instead
            let pred = PredicateIndex::init(val.clone());
            if let Err(existing_predicate) = predicate_values.try_insert(key, pred) {
                existing_predicate.current.add(val);
            };
        }
    }

    /// returns the store key id that fulfill the predicate condition
    #[tracing::instrument(skip_all)]
    pub(super) fn matches(
        &self,
        condition: &PredicateCondition,
        // used to check original store for things that do not have predicate
        store: &Store,
    ) -> Result<StdHashSet<StoreKeyId>, ServerError> {
        match condition {
            PredicateCondition::Value(main_predicate) => {
                let predicate_values = self.inner.pin();
                let key = main_predicate.get_key();
                if let Some(predicate) = predicate_values.get(key) {
                    // retrieve the precise predicate if it exists and check against it
                    return Ok(predicate.matches(main_predicate));
                }
                store.get_match_without_predicate(main_predicate)
            }
            PredicateCondition::And(first, second) => {
                let first_result = self.matches(first, store)?;
                let second_result = self.matches(second, store)?;
                // Get intersection of both conditions
                Ok(first_result.intersection(&second_result).cloned().collect())
            }
            PredicateCondition::Or(first, second) => {
                let first_result = self.matches(first, store)?;
                let second_result = self.matches(second, store)?;
                // Get union of both conditions
                Ok(first_result.union(&second_result).cloned().collect())
            }
        }
    }
}

/// A predicate index is a simple datastructure that stores a value key to all matching store key
/// ids. This is essential in helping us filter down the entire dataset using a predicate before
/// performing similarity algorithmic search
#[derive(Debug, Serialize, Deserialize)]
struct PredicateIndex(#[serde(with = "custom_metadata_map")] InnerPredicateIndex);

mod custom_metadata_map {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(map: &InnerPredicateIndex, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let vec: Vec<(MetadataValue, InnerPredicateIndexVal)> = map
            .iter(&map.guard())
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect();
        vec.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<InnerPredicateIndex, D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<(MetadataValue, InnerPredicateIndexVal)> = Vec::deserialize(deserializer)?;
        let map = ConcurrentHashMap::new();
        for (k, v) in vec {
            map.insert(k, v, &map.guard());
        }
        Ok(map)
    }
}

impl PredicateIndex {
    #[tracing::instrument(skip(self))]
    fn size(&self) -> usize {
        size_of_val(&self)
            + self
                .0
                .iter(&self.0.guard())
                .map(|(k, v)| size_of_val(k) + v.iter(&v.guard()).map(size_of_val).sum::<usize>())
                .sum::<usize>()
    }

    #[tracing::instrument(skip(init), fields(input_length = init.len()))]
    fn init(init: Vec<(MetadataValue, StoreKeyId)>) -> Self {
        let new = Self(InnerPredicateIndex::new());
        new.add(init);
        new
    }

    /// Removes a store key id when it's corresponding entry in the store is removed
    #[tracing::instrument(skip(self))]
    fn remove_store_keys(&self, remove_keys: &[StoreKeyId]) {
        let inner = self.0.pin();
        for (_, values) in inner.iter() {
            let values_pinned = values.pin();
            for k in remove_keys {
                values_pinned.remove(k);
            }
        }
    }

    /// adds a store key id to the index using the predicate value
    /// TODO: Optimize stack consumption of this particular call as it seems to consume more than
    /// the default number when ran using Loom, this may cause an issue down the line
    #[tracing::instrument(skip_all, fields(update_len = update.len()))]
    fn add(&self, update: Vec<(MetadataValue, StoreKeyId)>) {
        if update.is_empty() {
            return;
        }
        let chunk_size = parallel::chunk_size(update.len());
        update
            .into_par_iter()
            .chunks(chunk_size)
            .for_each(|values| {
                let pinned = self.0.pin();
                for (predicate_value, store_key_id) in values {
                    if let Some((_, value)) = pinned.get_key_value(&predicate_value) {
                        value.insert(store_key_id, &value.guard());
                    } else {
                        // Use try_insert as it is very possible that the hashmap itself now has that key that
                        // was not previously there as it has been inserted on a different thread
                        let new_hashset = ConcurrentHashSet::new();
                        new_hashset.insert(store_key_id.clone(), &new_hashset.guard());
                        if let Err(error_current) = pinned.try_insert(predicate_value, new_hashset)
                        {
                            error_current
                                .current
                                .insert(store_key_id, &error_current.current.guard());
                        }
                    }
                }
            });
    }

    /// checks the predicate index for a predicate op and value. The return type is a StdHashSet<_>
    /// because we do not modify it at any point so we do not need concurrency protection
    #[tracing::instrument(skip(self))]
    fn matches(&self, predicate: &Predicate) -> StdHashSet<StoreKeyId> {
        let pinned = self.0.pin();

        match predicate {
            Predicate::Equals { value, .. } => {
                if let Some(set) = pinned.get(value) {
                    set.pin().iter().cloned().collect::<StdHashSet<_>>()
                } else {
                    StdHashSet::new()
                }
            }
            Predicate::NotEquals { value, .. } => pinned
                .iter()
                .filter(|(key, _)| **key != *value)
                .flat_map(|(_, value)| value.pin().iter().cloned().collect::<Vec<_>>())
                .collect(),
            Predicate::In { value, .. } => pinned
                .iter()
                .filter(|(key, _)| value.contains(key))
                .flat_map(|(_, value)| value.pin().iter().cloned().collect::<Vec<_>>())
                .collect(),
            Predicate::NotIn { value, .. } => pinned
                .iter()
                .filter(|(key, _)| !value.contains(key))
                .flat_map(|(_, value)| value.pin().iter().cloned().collect::<Vec<_>>())
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap as StdHashMap;
    use std::num::NonZeroUsize;
    use std::sync::Arc;

    fn store_value_0() -> StoreValue {
        StdHashMap::from_iter(vec![
            (
                MetadataKey::new("name".into()),
                MetadataValue::RawString("David".into()),
            ),
            (
                MetadataKey::new("country".into()),
                MetadataValue::RawString("Nigeria".into()),
            ),
            (
                MetadataKey::new("state".into()),
                MetadataValue::RawString("Markudi".into()),
            ),
        ])
    }

    fn store_value_1() -> StoreValue {
        StdHashMap::from_iter(vec![
            (
                MetadataKey::new("name".into()),
                MetadataValue::RawString("David".into()),
            ),
            (
                MetadataKey::new("country".into()),
                MetadataValue::RawString("USA".into()),
            ),
            (
                MetadataKey::new("state".into()),
                MetadataValue::RawString("Washington".into()),
            ),
        ])
    }

    fn store_value_2() -> StoreValue {
        StdHashMap::from_iter(vec![
            (
                MetadataKey::new("name".into()),
                MetadataValue::RawString("Diretnan".into()),
            ),
            (
                MetadataKey::new("country".into()),
                MetadataValue::RawString("Nigeria".into()),
            ),
            (
                MetadataKey::new("state".into()),
                MetadataValue::RawString("Plateau".into()),
            ),
        ])
    }

    fn create_shared_predicate_indices(
        allowed_predicates: Vec<MetadataKey>,
    ) -> Arc<PredicateIndices> {
        let shared_pred = Arc::new(PredicateIndices::init(allowed_predicates));
        let handles = (0..4).map(|i| {
            let shared_data = shared_pred.clone();
            let handle = std::thread::spawn(move || {
                let values = match i {
                    0 => store_value_0(),
                    1 => store_value_1(),
                    2 => store_value_2(),
                    _ => StdHashMap::new(),
                };
                let store_key: StoreKeyId = format!("{i}").into();
                let data = vec![(store_key, values)];
                shared_data.add(data);
            });
            handle
        });
        for handle in handles {
            handle.join().unwrap();
        }
        shared_pred
    }

    fn create_shared_predicate() -> Arc<PredicateIndex> {
        let shared_pred = Arc::new(PredicateIndex::init(vec![]));
        let handles = (0..4).map(|i| {
            let shared_data = shared_pred.clone();
            let handle = std::thread::spawn(move || {
                let key = if i % 2 == 0 { "Even" } else { "Odd" };
                shared_data.add(vec![(
                    MetadataValue::RawString(key.into()),
                    format!("{i}").into(),
                )]);
            });
            handle
        });
        for handle in handles {
            handle.join().unwrap();
        }
        shared_pred
    }

    #[test]
    fn test_add_index_to_predicate_after() {
        let shared_pred = create_shared_predicate_indices(vec![MetadataKey::new("country".into())]);
        let result = shared_pred.matches(
            &PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("name".into()),
                value: MetadataValue::RawString("David".into()),
            }),
            &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
        );
        // We don't have an index but it should use original store and return empty
        assert!(result.unwrap().is_empty());
        shared_pred.add_predicates(
            vec![
                MetadataKey::new("country".into()),
                MetadataKey::new("name".into()),
            ],
            Some(vec![
                ("0".into(), store_value_0()),
                ("1".into(), store_value_1()),
                ("2".into(), store_value_2()),
            ]),
        );
        let result = shared_pred
            .matches(
                &PredicateCondition::Value(Predicate::Equals {
                    key: MetadataKey::new("name".into()),
                    value: MetadataValue::RawString("David".into()),
                }),
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        // Now we expect index to be up to date
        assert_eq!(result, StdHashSet::from_iter(["0".into(), "1".into()]),);
    }

    #[test]
    fn test_predicate_indices_matches() {
        let shared_pred = create_shared_predicate_indices(vec![
            MetadataKey::new("country".into()),
            MetadataKey::new("name".into()),
            MetadataKey::new("state".into()),
            MetadataKey::new("age".into()),
        ]);
        let result = shared_pred
            .matches(
                &PredicateCondition::Value(Predicate::NotEquals {
                    key: MetadataKey::new("age".into()),
                    value: MetadataValue::RawString("14".into()),
                }),
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        // There are no entries where age is 14
        assert!(result.is_empty());
        let result = shared_pred
            .matches(
                &PredicateCondition::Value(Predicate::NotEquals {
                    key: MetadataKey::new("country".into()),
                    value: MetadataValue::RawString("Nigeria".into()),
                }),
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        // only person 1 is not from Nigeria
        assert_eq!(result, StdHashSet::from_iter(["1".into()]));
        let result = shared_pred
            .matches(
                &PredicateCondition::Value(Predicate::Equals {
                    key: MetadataKey::new("country".into()),
                    value: MetadataValue::RawString("Nigeria".into()),
                }),
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        assert_eq!(result, StdHashSet::from_iter(["0".into(), "2".into()]),);
        let check = PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("state".into()),
            value: MetadataValue::RawString("Washington".into()),
        })
        .or(PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("age".into()),
            value: MetadataValue::RawString("14".into()),
        }));
        let result = shared_pred
            .matches(
                &check,
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        // only person 1 is from Washington
        assert_eq!(result, StdHashSet::from_iter(["1".into()]));
        let check = PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("country".into()),
            value: MetadataValue::RawString("Nigeria".into()),
        })
        .and(PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("state".into()),
            value: MetadataValue::RawString("Plateau".into()),
        }));
        let result = shared_pred
            .matches(
                &check,
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        // only person 1 is fulfills all
        assert_eq!(result, StdHashSet::from_iter(["2".into()]));
        let check = PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("name".into()),
            value: MetadataValue::RawString("David".into()),
        })
        .or(PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("name".into()),
            value: MetadataValue::RawString("Diretnan".into()),
        }));
        let result = shared_pred
            .matches(
                &check,
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        // all 3 fulfill this
        assert_eq!(
            result,
            StdHashSet::from_iter(["2".into(), "0".into(), "1".into(),]),
        );
        let check = check.and(PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("country".into()),
            value: MetadataValue::RawString("USA".into()),
        }));
        let result = shared_pred
            .matches(
                &check,
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        // only person 1 is from Washington with any of those names
        assert_eq!(result, StdHashSet::from_iter(["1".into()]));
        // remove all Nigerians from the predicate and check that conditions working before no
        // longer work and those working before still work
        shared_pred.remove_store_keys(&["0".into(), "2".into()]);
        let result = shared_pred
            .matches(
                &PredicateCondition::Value(Predicate::Equals {
                    key: MetadataKey::new("country".into()),
                    value: MetadataValue::RawString("Nigeria".into()),
                }),
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        assert!(result.is_empty());
        let check = check.and(PredicateCondition::Value(Predicate::Equals {
            key: MetadataKey::new("country".into()),
            value: MetadataValue::RawString("USA".into()),
        }));
        let result = shared_pred
            .matches(
                &check,
                &Store::create(NonZeroUsize::new(1).unwrap(), vec![], StdHashSet::new()),
            )
            .unwrap();
        // only person 1 is from Washington with any of those names
        assert_eq!(result, StdHashSet::from_iter(["1".into()]));
    }

    #[test]
    fn test_adding_and_removing_entries_for_predicate() {
        let shared_pred = create_shared_predicate();
        assert_eq!(shared_pred.0.len(), 2);
        assert_eq!(
            shared_pred
                .0
                .pin()
                .get(&MetadataValue::RawString("Even".into()))
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            shared_pred
                .0
                .pin()
                .get(&MetadataValue::RawString("Odd".into()))
                .unwrap()
                .len(),
            2
        );
        shared_pred.remove_store_keys(&["1".into(), "0".into()]);
        assert_eq!(
            shared_pred
                .0
                .pin()
                .get(&MetadataValue::RawString("Even".into()))
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            shared_pred
                .0
                .pin()
                .get(&MetadataValue::RawString("Odd".into()))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_matches_works_on_predicate_index() {
        let shared_pred = create_shared_predicate();
        assert_eq!(
            shared_pred
                .matches(&Predicate::Equals {
                    key: MetadataKey::new("".to_string()),
                    value: MetadataValue::RawString("Even".into())
                })
                .len(),
            2
        );
        assert_eq!(
            shared_pred
                .matches(&Predicate::NotEquals {
                    key: MetadataKey::new("".to_string()),
                    value: MetadataValue::RawString("Even".into())
                })
                .len(),
            2
        );
        assert_eq!(
            shared_pred
                .matches(&Predicate::Equals {
                    key: MetadataKey::new("".to_string()),
                    value: MetadataValue::RawString("Odd".into())
                })
                .len(),
            2
        );
        assert_eq!(
            shared_pred
                .matches(&Predicate::In {
                    key: MetadataKey::new("".to_string()),
                    value: StdHashSet::from_iter([
                        MetadataValue::RawString("Odd".into()),
                        MetadataValue::RawString("Even".into())
                    ])
                })
                .len(),
            4
        );
        assert_eq!(
            shared_pred
                .matches(&Predicate::NotIn {
                    key: MetadataKey::new("".to_string()),
                    value: StdHashSet::from_iter([MetadataValue::RawString("Odd".into()),])
                })
                .len(),
            2
        );
        assert_eq!(
            shared_pred
                .matches(&Predicate::NotIn {
                    key: MetadataKey::new("".to_string()),
                    value: StdHashSet::from_iter([MetadataValue::RawString("Even".into()),])
                })
                .len(),
            2
        );
        assert_eq!(
            shared_pred
                .matches(&Predicate::NotEquals {
                    key: MetadataKey::new("".to_string()),
                    value: MetadataValue::RawString("Odd".into())
                })
                .len(),
            2
        );
        assert_eq!(
            shared_pred
                .matches(&Predicate::NotEquals {
                    key: MetadataKey::new("".to_string()),
                    value: MetadataValue::RawString("NotExists".into())
                })
                .len(),
            4
        );
        assert_eq!(
            shared_pred
                .matches(&Predicate::Equals {
                    key: MetadataKey::new("".to_string()),
                    value: MetadataValue::RawString("NotExists".into())
                })
                .len(),
            0
        );
    }
}
