use super::ConcurrentHashMap;
use super::ConcurrentHashSet;
use super::StoreKeyId;
use super::StoreValue;
use itertools::Itertools;
use std::collections::HashSet as StdHashSet;
use types::predicate::Predicate;
use types::predicate::PredicateCondition;
use types::predicate::PredicateKey;
use types::predicate::PredicateOp;
use types::predicate::PredicateValue;

type InnerPredicateIndex = ConcurrentHashMap<PredicateValue, ConcurrentHashSet<StoreKeyId>>;
type InnerPredicateIndices = ConcurrentHashMap<PredicateKey, PredicateIndex>;

/// Predicate indices are all the indexes referenced by their names
#[derive(Debug)]
pub(super) struct PredicateIndices {
    inner: InnerPredicateIndices,
    /// These are the index keys that are meant to generate predicate indexes
    allowed_predicates: ConcurrentHashSet<PredicateKey>,
}

impl PredicateIndices {
    pub(super) fn init(allowed_predicates: Vec<PredicateKey>) -> Self {
        let created = ConcurrentHashSet::new();
        for key in allowed_predicates {
            created.insert(key, &created.guard());
        }
        Self {
            inner: InnerPredicateIndices::new(),
            allowed_predicates: created,
        }
    }

    /// Removes predicates from being tracked
    pub(super) fn remove_predicates(&self, predicates: Vec<PredicateKey>) {
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
        predicates: Vec<PredicateKey>,
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
    pub(super) fn matches(&self, condition: &PredicateCondition) -> Vec<StoreKeyId> {
        match condition {
            PredicateCondition::Value(Predicate { key, value, op }) => {
                let predicate_values = self.inner.pin();
                // retrieve the precise predicate if it exists and check against it, else we assume
                // this doesn't match anything and return an empty vec
                if let Some(predicate) = predicate_values.get(key) {
                    return predicate.matches(op, value);
                }
                vec![]
            }
            PredicateCondition::And(first, second) => {
                let first_result = self.matches(first);
                let second_result = self.matches(second);
                // Get intersection of both conditions
                first_result
                    .into_iter()
                    .filter(|elem| second_result.contains(elem))
                    .collect()
            }
            PredicateCondition::Or(first, second) => {
                let first_result = self.matches(first);
                let second_result = self.matches(second);
                // Get union of both conditions
                // There cannot be multiple occurences in either first or second as specificied by
                // PredicateIndex::matches but upon union, there might be multiple occurences,
                // hence the need for unique
                first_result
                    .into_iter()
                    .chain(second_result.into_iter())
                    .unique()
                    .collect()
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
    fn init(init: Vec<(PredicateValue, StoreKeyId)>) -> Self {
        let new = Self(InnerPredicateIndex::new());
        new.add(init);
        new
    }
    /// adds a store key id to the index using the predicate value
    /// TODO: Optimize stack consumption of this particular call as it seems to consume more than
    /// the default number when ran using Loom, this may cause an issue down the line
    fn add(&self, update: Vec<(PredicateValue, StoreKeyId)>) {
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
    /// checks the predicate index for a predicate op and value. The return type is a Vec<_> but
    /// because the internal type is a concurrent hashset, we can be certain that there are no
    /// duplicates within it, hence no need for higher up validation
    fn matches<'a>(
        &self,
        predicate_op: &'a PredicateOp,
        value: &'a PredicateValue,
    ) -> Vec<StoreKeyId> {
        let pinned = self.0.pin();
        match predicate_op {
            PredicateOp::Equals => {
                if let Some(set) = pinned.get(value) {
                    set.pin().iter().cloned().collect::<Vec<_>>()
                } else {
                    vec![]
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
    use super::super::StdHashMap;
    use super::*;
    use loom::thread;
    use loom::thread::Builder;
    use loom::MAX_THREADS;
    use std::sync::Arc;

    fn store_value_0() -> StoreValue {
        StdHashMap::from_iter(vec![
            ("name".into(), "David".into()),
            ("country".into(), "Nigeria".into()),
            ("state".into(), "Markudi".into()),
        ])
    }

    fn store_value_1() -> StoreValue {
        StdHashMap::from_iter(vec![
            ("name".into(), "David".into()),
            ("country".into(), "USA".into()),
            ("state".into(), "Washington".into()),
        ])
    }

    fn store_value_2() -> StoreValue {
        StdHashMap::from_iter(vec![
            ("name".into(), "Diretnan".into()),
            ("country".into(), "Nigeria".into()),
            ("state".into(), "Plateau".into()),
        ])
    }

    fn create_shared_predicate_indices(
        allowed_predicates: Vec<PredicateKey>,
    ) -> Arc<PredicateIndices> {
        let shared_pred = Arc::new(PredicateIndices::init(allowed_predicates));
        let handles = (0..MAX_THREADS - 1).map(|i| {
            // MAX_THREADS is usually 4 so we are iterating from 0, 1, 2 since loom expects our
            // max number of threads spawned to be MAX_THREADS - 1
            let shared_data = shared_pred.clone();
            let handle = std::thread::spawn(move || {
                let values = match i {
                    0 => store_value_0(),
                    1 => store_value_1(),
                    2 => store_value_2(),
                    _ => StdHashMap::new(),
                };
                let store_key = StoreKeyId(format!("{i}"));
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
        let handles = (0..MAX_THREADS - 1).map(|i| {
            let shared_data = shared_pred.clone();
            let handle = thread::spawn(move || {
                let key = if i % 2 == 0 { "Even" } else { "Odd" };
                shared_data.add(vec![(String::from(key), StoreKeyId(format!("{i}")))]);
            });
            handle
        });
        for handle in handles {
            handle.join().unwrap();
        }
        shared_pred
    }

    fn assert_vecs_equal_unordered<T: PartialEq + Ord + Clone + std::fmt::Debug>(
        expected: &[T],
        actual: &[T],
    ) {
        let mut expected_sorted = expected.to_vec();
        let mut actual_sorted = actual.to_vec();

        expected_sorted.sort();
        actual_sorted.sort();

        assert_eq!(expected_sorted, actual_sorted);
    }

    #[test]
    fn test_add_index_to_predicate_after() {
        let shared_pred =
            create_shared_predicate_indices(vec![PredicateKey::new("country".into())]);
        let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
            key: PredicateKey::new("name".into()),
            value: PredicateValue::new("David".into()),
            op: PredicateOp::Equals,
        }));
        // We expect to be empty as we didn't index name
        assert_eq!(result, vec![]);
        shared_pred.add_predicates(
            vec![
                PredicateKey::new("country".into()),
                PredicateKey::new("name".into()),
            ],
            Some(vec![
                (StoreKeyId("0".into()), store_value_0()),
                (StoreKeyId("1".into()), store_value_1()),
                (StoreKeyId("2".into()), store_value_2()),
            ]),
        );
        let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
            key: PredicateKey::new("name".into()),
            value: PredicateValue::new("David".into()),
            op: PredicateOp::Equals,
        }));
        // Now we expect index to be up to date
        assert_vecs_equal_unordered(
            &result,
            &vec![StoreKeyId("0".into()), StoreKeyId("1".into())],
        );
    }

    #[test]
    fn test_predicate_indices_matches() {
        loom::model(|| {
            let shared_pred = create_shared_predicate_indices(vec![
                PredicateKey::new("country".into()),
                PredicateKey::new("name".into()),
                PredicateKey::new("state".into()),
                PredicateKey::new("age".into()),
            ]);
            let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
                key: PredicateKey::new("age".into()),
                value: PredicateValue::new("14".into()),
                op: PredicateOp::NotEquals,
            }));
            // There is no key like this
            assert_eq!(result, vec![]);
            let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
                key: PredicateKey::new("country".into()),
                value: PredicateValue::new("Nigeria".into()),
                op: PredicateOp::NotEquals,
            }));
            // only person 1 is not from Nigeria
            assert_eq!(result, vec![StoreKeyId("1".into())]);
            let result = shared_pred.matches(&PredicateCondition::Value(Predicate {
                key: PredicateKey::new("country".into()),
                value: PredicateValue::new("Nigeria".into()),
                op: PredicateOp::Equals,
            }));
            assert_vecs_equal_unordered(
                &result,
                &vec![StoreKeyId("0".into()), StoreKeyId("2".into())],
            );
            let check = PredicateCondition::Value(Predicate {
                key: PredicateKey::new("state".into()),
                value: PredicateValue::new("Washington".into()),
                op: PredicateOp::Equals,
            })
            .or(PredicateCondition::Value(Predicate {
                key: PredicateKey::new("age".into()),
                value: PredicateValue::new("14".into()),
                op: PredicateOp::Equals,
            }));
            let result = shared_pred.matches(&check);
            // only person 1 is from Washington
            assert_eq!(result, vec![StoreKeyId("1".into())]);
            let check = PredicateCondition::Value(Predicate {
                key: PredicateKey::new("country".into()),
                value: PredicateValue::new("Nigeria".into()),
                op: PredicateOp::Equals,
            })
            .and(PredicateCondition::Value(Predicate {
                key: PredicateKey::new("state".into()),
                value: PredicateValue::new("Plateau".into()),
                op: PredicateOp::Equals,
            }));
            let result = shared_pred.matches(&check);
            // only person 1 is fulfills all
            assert_eq!(result, vec![StoreKeyId("2".into())]);
            let check = PredicateCondition::Value(Predicate {
                key: PredicateKey::new("name".into()),
                value: PredicateValue::new("David".into()),
                op: PredicateOp::Equals,
            })
            .or(PredicateCondition::Value(Predicate {
                key: PredicateKey::new("name".into()),
                value: PredicateValue::new("Diretnan".into()),
                op: PredicateOp::Equals,
            }));
            let result = shared_pred.matches(&check);
            // all 3 fulfill this
            assert_vecs_equal_unordered(
                &result,
                &vec![
                    StoreKeyId("2".into()),
                    StoreKeyId("0".into()),
                    StoreKeyId("1".into()),
                ],
            );
            let check = check.and(PredicateCondition::Value(Predicate {
                key: PredicateKey::new("country".into()),
                value: PredicateValue::new("USA".into()),
                op: PredicateOp::Equals,
            }));
            let result = shared_pred.matches(&check);
            // only person 1 is from Washington with any of those names
            assert_eq!(result, vec![StoreKeyId("1".into())]);
        })
    }

    #[test]
    fn test_adding_entries_to_predicate() {
        loom::model(|| {
            let shared_pred = create_shared_predicate();
            assert_eq!(shared_pred.0.len(), 2);
            assert_eq!(
                shared_pred
                    .0
                    .pin()
                    .get(&PredicateValue::new("Even".into()))
                    .unwrap()
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .0
                    .pin()
                    .get(&PredicateValue::new("Odd".into()))
                    .unwrap()
                    .len(),
                2
            );
        })
    }

    #[test]
    fn test_matches_works_on_predicate_index() {
        loom::model(|| {
            let shared_pred = create_shared_predicate();
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::Equals, &PredicateValue::new("Even".into()))
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::NotEquals, &PredicateValue::new("Even".into()))
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::Equals, &PredicateValue::new("Odd".into()))
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::NotEquals, &PredicateValue::new("Odd".into()))
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(
                        &PredicateOp::NotEquals,
                        &PredicateValue::new("NotExists".into())
                    )
                    .len(),
                4
            );
            assert_eq!(
                shared_pred
                    .matches(
                        &PredicateOp::Equals,
                        &PredicateValue::new("NotExists".into())
                    )
                    .len(),
                0
            );
        })
    }
}
