use super::ConcurrentHashMap;
use super::ConcurrentHashSet;
use super::StoreKeyId;
use super::StoreValue;
use itertools::Itertools;
use types::predicate::Predicate;
use types::predicate::PredicateCondition;
use types::predicate::PredicateOp;

type InnerPredicateIndex = ConcurrentHashMap<String, ConcurrentHashSet<StoreKeyId>>;
type InnerPredicateIndices = ConcurrentHashMap<String, PredicateIndex>;

/// Predicate indices are all the indexes referenced by their names
#[derive(Debug)]
pub(super) struct PredicateIndices {
    inner: InnerPredicateIndices,
    /// These are the index keys that are meant to generate predicate indexes
    allowed_predicates: ConcurrentHashSet<String>,
}

impl PredicateIndices {
    pub(super) fn new() -> Self {
        Self {
            inner: InnerPredicateIndices::new(),
            allowed_predicates: ConcurrentHashSet::new(),
        }
    }

    /// Removes predicates from being tracked
    pub(super) fn remove_predicates(&self, predicates: Vec<String>) {
        let pinned_keys = self.allowed_predicates.pin();
        let pinned_predicate_values = self.inner.pin();
        for predicate in predicates {
            pinned_keys.remove(&predicate);
            pinned_predicate_values.remove(&predicate);
        }
    }

    /// Adds predicates if the key is within allowed_predicates
    pub(super) fn add(&self, new: Vec<(StoreKeyId, StoreValue)>) {
        let allowed_keys = self.allowed_predicates.pin();
        let predicate_values = self.inner.pin();
        for (store_key_id, store_value) in new {
            for (key, val) in store_value {
                if !allowed_keys.contains(&key) {
                    continue;
                }
                let update = vec![(val, store_key_id.clone())];
                // If there exists a predicate index as we want to update it, just add to that
                // predicate index instead
                if let Err(existing_predicate) =
                    predicate_values.try_insert(key.clone(), PredicateIndex::new(update.clone()))
                {
                    existing_predicate.current.add(update);
                }
            }
        }
    }

    /// returns the store key id that fulfill the predicate condition
    pub(super) fn matches(&self, condition: &PredicateCondition) -> Vec<StoreKeyId> {
        match condition {
            PredicateCondition::Value(Predicate { key, value, op }) => {
                let predicate_values = self.inner.pin();
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
    fn new(init: Vec<(String, StoreKeyId)>) -> Self {
        let new = Self(InnerPredicateIndex::new());
        new.add(init);
        new
    }
    /// adds a store key id to the index using the predicate value
    fn add(&self, update: Vec<(String, StoreKeyId)>) {
        let pinned = self.0.pin();
        for (predicate_value, store_key_id) in update {
            if let Some((_, value)) = pinned.get_key_value(&predicate_value) {
                value.insert(store_key_id, &value.guard());
            } else {
                let new_hashset = ConcurrentHashSet::new();
                new_hashset.insert(store_key_id.clone(), &new_hashset.guard());
                // Use try_insert as it is very possible that the hashmap itself now has that key that
                // was not previously there as it has been inserted on a different thread
                if let Err(error_current) = pinned.try_insert(predicate_value, new_hashset) {
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
    fn matches<'a>(&self, predicate_op: &'a PredicateOp, value: &'a String) -> Vec<StoreKeyId> {
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
    use super::*;
    use loom::thread;
    use loom::MAX_THREADS;
    use std::sync::Arc;

    fn create_shared_predicate() -> Arc<PredicateIndex> {
        let shared_pred = Arc::new(PredicateIndex::new(vec![]));
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

    #[test]
    fn test_adding_entries_to_predicate() {
        loom::model(|| {
            let shared_pred = create_shared_predicate();
            assert_eq!(shared_pred.0.len(), 2);
            assert_eq!(shared_pred.0.pin().get("Even").unwrap().len(), 2);
            assert_eq!(shared_pred.0.pin().get("Odd").unwrap().len(), 2);
        })
    }

    #[test]
    fn test_matches_works_on_predicate_index() {
        loom::model(|| {
            let shared_pred = create_shared_predicate();
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::Equals, &"Even".into())
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::NotEquals, &"Even".into())
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::Equals, &"Odd".into())
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::NotEquals, &"Odd".into())
                    .len(),
                2
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::NotEquals, &"NotExists".into())
                    .len(),
                4
            );
            assert_eq!(
                shared_pred
                    .matches(&PredicateOp::Equals, &"NotExists".into())
                    .len(),
                0
            );
        })
    }
}
