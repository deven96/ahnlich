use super::store::StoreKeyId;
use flurry::HashMap as ConcurrentHashMap;
use flurry::HashSet as ConcurrentHashSet;
use itertools::Itertools;
use std::collections::HashSet as StdHashSet;
use std::mem::size_of_val;
use types::keyval::StoreValue;
use types::metadata::MetadataKey;
use types::metadata::MetadataValue;
use types::predicate::Predicate;
use types::predicate::PredicateCondition;
use types::predicate::PredicateOp;

type InnerPredicateIndex = ConcurrentHashMap<MetadataValue, ConcurrentHashSet<StoreKeyId>>;
type InnerPredicateIndices = ConcurrentHashMap<MetadataKey, PredicateIndex>;

/// Predicate indices are all the indexes referenced by their names
#[derive(Debug)]
pub(super) struct PredicateIndices {
    inner: InnerPredicateIndices,
    /// These are the index keys that are meant to generate predicate indexes
    allowed_predicates: ConcurrentHashSet<MetadataKey>,
}

impl PredicateIndices {
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

    pub(super) fn init(allowed_predicates: Vec<MetadataKey>) -> Self {
        let created = ConcurrentHashSet::new();
        for key in allowed_predicates {
            created.insert(key, &created.guard());
        }
        Self {
            inner: InnerPredicateIndices::new(),
            allowed_predicates: created,
        }
    }

    /// Removes a store key id when it's corresponding entry in the store is removed
    pub(super) fn remove_store_keys(&self, remove_keys: &[StoreKeyId]) {
        let pinned = self.inner.pin();
        for (_, values) in pinned.iter() {
            values.remove_store_keys(remove_keys);
        }
    }

    /// Removes predicates from being tracked
    pub(super) fn remove_predicates(&self, predicates: Vec<MetadataKey>) {
        let pinned_keys = self.allowed_predicates.pin();
        let pinned_predicate_values = self.inner.pin();
        for predicate in predicates {
            pinned_keys.remove(&predicate);
            pinned_predicate_values.remove(&predicate);
        }
    }

    /// This adds predicates to the allowed predicates and allows newer entries to be indexed
    /// It first checks for the predicates that do not currently exist and then attempts to add
    /// those to the underlying predicates
    pub(super) fn add_predicates(
        &self,
        predicates: Vec<MetadataKey>,
        refresh_with_values: Option<Vec<(StoreKeyId, StoreValue)>>,
    ) {
        let pinned_keys = self.allowed_predicates.pin();
        let pinned_inner = self.inner.pin();
        // `insert` implicity adds it to allowed_predicates which is what lets us to be able to
        // search again
        let new_predicates: StdHashSet<_> = predicates
            .into_iter()
            .filter(|pred| pinned_keys.insert(pred.clone()))
            .unique()
            .collect();
        // Only update for new predicates
        if let Some(new_values) = (!new_predicates.is_empty())
            .then_some(refresh_with_values)
            .flatten()
        {
            for new_predicate in new_predicates {
                let val = new_values
                    .iter()
                    .map(|(store_key_id, store_value)| {
                        store_value
                            .iter()
                            .filter(|(key, _)| **key == new_predicate)
                            .map(|(_, val)| (val.clone(), store_key_id.clone()))
                    })
                    .flatten()
                    .collect::<Vec<_>>();
                let pred = PredicateIndex::init(val.clone());
                if let Err(existing_predicate) =
                    pinned_inner.try_insert(new_predicate.clone(), pred)
                {
                    existing_predicate.current.add(val)
                }
            }
        }
    }

    /// Adds predicates if the key is within allowed_predicates
    pub(super) fn add(&self, new: Vec<(StoreKeyId, StoreValue)>) {
        let predicate_values = self.inner.pin();
        let iter = new
            .into_iter()
            .flat_map(|(store_key_id, store_value)| {
                store_value.into_iter().map(move |(key, val)| {
                    let allowed_keys = self.allowed_predicates.pin();
                    allowed_keys
                        .contains(&key)
                        .then_some((store_key_id.clone(), key, val))
                })
            })
            .flatten()
            .into_iter()
            .map(|(store_key_id, key, val)| (key, (val, store_key_id)))
            .into_group_map();

        for (key, val) in iter {
            // If there exists a predicate index as we want to update it, just add to that
            // predicate index instead
            let pred = PredicateIndex::init(val.clone());
            if let Err(existing_predicate) = predicate_values.try_insert(key.clone(), pred) {
                existing_predicate.current.add(val);
            };
        }
    }

    /// returns the store key id that fulfill the predicate condition
    pub(super) fn matches(&self, condition: &PredicateCondition) -> StdHashSet<StoreKeyId> {
        match condition {
            PredicateCondition::Value(Predicate { key, value, op }) => {
                let predicate_values = self.inner.pin();
                // retrieve the precise predicate if it exists and check against it, else we assume
                // this doesn't match anything and return an empty vec
                if let Some(predicate) = predicate_values.get(key) {
                    return predicate.matches(op, value);
                }
                StdHashSet::new()
            }
            PredicateCondition::And(first, second) => {
                let first_result = self.matches(first);
                let second_result = self.matches(second);
                // Get intersection of both conditions
                first_result.intersection(&second_result).cloned().collect()
            }
            PredicateCondition::Or(first, second) => {
                let first_result = self.matches(first);
                let second_result = self.matches(second);
                // Get union of both conditions
                first_result.union(&second_result).cloned().collect()
            }
        }
    }
}

/// A predicate index is a simple datastructure that stores a value key to all matching store key
/// ids. This is essential in helping us filter down the entire dataset using a predicate before
/// performing similarity algorithmic search
#[derive(Debug)]
struct PredicateIndex(InnerPredicateIndex);

impl PredicateIndex {
    fn size(&self) -> usize {
        size_of_val(&self)
            + self
                .0
                .iter(&self.0.guard())
                .map(|(k, v)| size_of_val(k) + v.iter(&v.guard()).map(size_of_val).sum::<usize>())
                .sum::<usize>()
    }

    fn init(init: Vec<(MetadataValue, StoreKeyId)>) -> Self {
        let new = Self(InnerPredicateIndex::new());
        new.add(init);
        new
    }

    /// Removes a store key id when it's corresponding entry in the store is removed
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
    fn add(&self, update: Vec<(MetadataValue, StoreKeyId)>) {
        if update.is_empty() {
            return;
        }
        let pinned = self.0.pin();
        for (predicate_value, store_key_id) in update {
            if let Some((_, value)) = pinned.get_key_value(&predicate_value) {
                value.insert(store_key_id, &value.guard());
            } else {
                // Use try_insert as it is very possible that the hashmap itself now has that key that
                // was not previously there as it has been inserted on a different thread
                let new_hashset = ConcurrentHashSet::new();
                new_hashset.insert(store_key_id.clone(), &new_hashset.guard());
                if let Err(error_current) = pinned.try_insert(predicate_value.clone(), new_hashset)
                {
                    error_current
                        .current
                        .insert(store_key_id, &error_current.current.guard());
                }
            }
        }
    }
    /// checks the predicate index for a predicate op and value. The return type is a StdHashSet<_>
    /// because we do not modify it at any point so we do not need concurrency protection
    fn matches<'a>(
        &self,
        predicate_op: &'a PredicateOp,
        value: &'a MetadataValue,
    ) -> StdHashSet<StoreKeyId> {
        let pinned = self.0.pin();
        match predicate_op {
            PredicateOp::Equals => {
                if let Some(set) = pinned.get(value) {
                    set.pin().iter().cloned().collect::<StdHashSet<_>>()
                } else {
                    StdHashSet::new()
                }
            }
            PredicateOp::NotEquals => pinned
                .iter()
                .filter(|(key, _)| **key != *value)
                .map(|(_, value)| value.pin().iter().cloned().collect::<Vec<_>>())
                .flatten()
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use loom::thread;
    use std::collections::HashMap as StdHashMap;
    use std::sync::Arc;

    fn store_value_0() -> StoreValue {
        StdHashMap::from_iter(vec![
            (
                MetadataKey::new("name".into()),
                MetadataValue::new("David".into()),
            ),
            (
                MetadataKey::new("country".into()),
                MetadataValue::new("Nigeria".into()),
            ),
            (
                MetadataKey::new("state".into()),
                MetadataValue::new("Markudi".into()),
            ),
        ])
    }

    fn store_value_1() -> StoreValue {
        StdHashMap::from_iter(vec![
            (
                MetadataKey::new("name".into()),
                MetadataValue::new("David".into()),
            ),
            (
                MetadataKey::new("country".into()),
                MetadataValue::new("USA".into()),
            ),
            (
                MetadataKey::new("state".into()),
                MetadataValue::new("Washington".into()),
            ),
        ])
    }

    fn store_value_2() -> StoreValue {
        StdHashMap::from_iter(vec![
            (
                MetadataKey::new("name".into()),
                MetadataValue::new("Diretnan".into()),
            ),
            (
                MetadataKey::new("country".into()),
                MetadataValue::new("Nigeria".into()),
            ),
            (
                MetadataKey::new("state".into()),
                MetadataValue::new("Plateau".into()),
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
            let handle = thread::spawn(move || {
                let key = if i % 2 == 0 { "Even" } else { "Odd" };
                shared_data.add(vec![(
                    MetadataValue::new(key.into()),
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
        let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
            key: MetadataKey::new("name".into()),
            value: MetadataValue::new("David".into()),
            op: PredicateOp::Equals,
        }));
        // We expect to be empty as we didn't index name
        assert!(result.is_empty());
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
        let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
            key: MetadataKey::new("name".into()),
            value: MetadataValue::new("David".into()),
            op: PredicateOp::Equals,
        }));
        // Now we expect index to be up to date
        assert_eq!(result, StdHashSet::from_iter(["0".into(), "1".into()]),);
    }

    #[test]
    fn test_predicate_indices_matches() {
        loom::model(|| {
            let shared_pred = create_shared_predicate_indices(vec![
                MetadataKey::new("country".into()),
                MetadataKey::new("name".into()),
                MetadataKey::new("state".into()),
                MetadataKey::new("age".into()),
            ]);
            let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
                key: MetadataKey::new("age".into()),
                value: MetadataValue::new("14".into()),
                op: PredicateOp::NotEquals,
            }));
            // There is no key like this
            assert!(result.is_empty());
            let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
                key: MetadataKey::new("country".into()),
                value: MetadataValue::new("Nigeria".into()),
                op: PredicateOp::NotEquals,
            }));
            // only person 1 is not from Nigeria
            assert_eq!(result, StdHashSet::from_iter(["1".into()]));
            let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
                key: MetadataKey::new("country".into()),
                value: MetadataValue::new("Nigeria".into()),
                op: PredicateOp::Equals,
            }));
            assert_eq!(result, StdHashSet::from_iter(["0".into(), "2".into()]),);
            let check = PredicateCondition::Value(Predicate {
                key: MetadataKey::new("state".into()),
                value: MetadataValue::new("Washington".into()),
                op: PredicateOp::Equals,
            })
            .or(PredicateCondition::Value(Predicate {
                key: MetadataKey::new("age".into()),
                value: MetadataValue::new("14".into()),
                op: PredicateOp::Equals,
            }));
            let result = shared_pred.matches(&check);
            // only person 1 is from Washington
            assert_eq!(result, StdHashSet::from_iter(["1".into()]));
            let check = PredicateCondition::Value(Predicate {
                key: MetadataKey::new("country".into()),
                value: MetadataValue::new("Nigeria".into()),
                op: PredicateOp::Equals,
            })
            .and(PredicateCondition::Value(Predicate {
                key: MetadataKey::new("state".into()),
                value: MetadataValue::new("Plateau".into()),
                op: PredicateOp::Equals,
            }));
            let result = shared_pred.matches(&check);
            // only person 1 is fulfills all
            assert_eq!(result, StdHashSet::from_iter(["2".into()]));
            let check = PredicateCondition::Value(Predicate {
                key: MetadataKey::new("name".into()),
                value: MetadataValue::new("David".into()),
                op: PredicateOp::Equals,
            })
            .or(PredicateCondition::Value(Predicate {
                key: MetadataKey::new("name".into()),
                value: MetadataValue::new("Diretnan".into()),
                op: PredicateOp::Equals,
            }));
            let result = shared_pred.matches(&check);
            // all 3 fulfill this
            assert_eq!(
                result,
                StdHashSet::from_iter(["2".into(), "0".into(), "1".into(),]),
            );
            let check = check.and(PredicateCondition::Value(Predicate {
                key: MetadataKey::new("country".into()),
                value: MetadataValue::new("USA".into()),
                op: PredicateOp::Equals,
            }));
            let result = shared_pred.matches(&check);
            // only person 1 is from Washington with any of those names
            assert_eq!(result, StdHashSet::from_iter(["1".into()]));
            // remove all Nigerians from the predicate and check that conditions working before no
            // longer work and those working before still work
            shared_pred.remove_store_keys(&["0".into(), "2".into()]);
            let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
                key: MetadataKey::new("country".into()),
                value: MetadataValue::new("Nigeria".into()),
                op: PredicateOp::Equals,
            }));
            assert!(result.is_empty());
            let check = check.and(PredicateCondition::Value(Predicate {
                key: MetadataKey::new("country".into()),
                value: MetadataValue::new("USA".into()),
                op: PredicateOp::Equals,
            }));
            let result = shared_pred.matches(&check);
            // only person 1 is from Washington with any of those names
            assert_eq!(result, StdHashSet::from_iter(["1".into()]));
        })
    }

    #[test]
    fn test_adding_and_removing_entries_for_predicate() {
        loom::model(|| {
            let shared_pred = create_shared_predicate();
            assert_eq!(shared_pred.0.len(), 2);
            assert_eq!(
                shared_pred
                    .0
                    .pin()
                    .get(&MetadataValue::new("Even".into()))
                    .unwrap()
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .0
                    .pin()
                    .get(&MetadataValue::new("Odd".into()))
                    .unwrap()
                    .len(),
                2
            );
            shared_pred.remove_store_keys(&["1".into(), "0".into()]);
            assert_eq!(
                shared_pred
                    .0
                    .pin()
                    .get(&MetadataValue::new("Even".into()))
                    .unwrap()
                    .len(),
                1
            );
            assert_eq!(
                shared_pred
                    .0
                    .pin()
                    .get(&MetadataValue::new("Odd".into()))
                    .unwrap()
                    .len(),
                1
            );
        })
    }

    #[test]
    fn test_matches_works_on_predicate_index() {
        loom::model(|| {
            let shared_pred = create_shared_predicate();
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::Equals, &MetadataValue::new("Even".into()))
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::NotEquals, &MetadataValue::new("Even".into()))
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::Equals, &MetadataValue::new("Odd".into()))
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::NotEquals, &MetadataValue::new("Odd".into()))
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(
                        &PredicateOp::NotEquals,
                        &MetadataValue::new("NotExists".into())
                    )
                    .len(),
                4
            );
            assert_eq!(
                shared_pred
                    .matches(
                        &PredicateOp::Equals,
                        &MetadataValue::new("NotExists".into())
                    )
                    .len(),
                0
            );
        })
    }
}