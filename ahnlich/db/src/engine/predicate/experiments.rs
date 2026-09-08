//! Benchmark-only alternatives to PredicateIndices::add at ab4ffc70.
//!
//! Control calls the production implementation. Each candidate changes exactly one
//! area of that implementation; default builds use neither. The explicit
//! bench-existing-predicate-index feature routes Set through candidate 1. Keep the
//! duplicated unchanged sections aligned with production when updating experiments.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub type Entries = Vec<(StoreKeyId, Arc<StoreValue>)>;
pub type Memberships = BTreeMap<String, BTreeMap<MetadataValue, BTreeSet<StoreKeyId>>>;

#[derive(Clone, Copy)]
pub enum Variant {
    Control,
    ExistingIndex,
}

/// Narrow facade so benchmarks can exercise the real private predicate structures.
/// Fixture construction and inspection belong outside the measured routine.
pub struct PredicateIngestionFixture {
    indices: PredicateIndices,
    config: crate::engine::store::ParallelismConfig,
    active_requests: usize,
}

impl PredicateIngestionFixture {
    pub fn new(
        allowed: Vec<String>,
        initial: Entries,
        config: crate::engine::store::ParallelismConfig,
        active_requests: usize,
    ) -> Self {
        let indices = PredicateIndices::init(allowed);
        indices.add(initial, &config, active_requests);
        Self {
            indices,
            config,
            active_requests,
        }
    }

    pub fn run(&self, variant: Variant, entries: Entries) {
        match variant {
            Variant::Control => self
                .indices
                .add(entries, &self.config, self.active_requests),
            Variant::ExistingIndex => self.indices.add_existing_index_candidate(
                entries,
                &self.config,
                self.active_requests,
            ),
        }
    }

    pub fn memberships(&self) -> Memberships {
        self.indices
            .inner
            .pin()
            .iter()
            .map(|(key, index)| {
                let buckets = index
                    .0
                    .pin()
                    .iter()
                    .map(|(value, ids)| (value.clone(), ids.pin().iter().copied().collect()))
                    .collect();
                (key.clone(), buckets)
            })
            .collect()
    }
}

impl PredicateIndices {
    #[tracing::instrument(skip_all, fields(new_len = new.len()))]
    pub(in crate::engine) fn add_existing_index_candidate(
        &self,
        new: Vec<(StoreKeyId, Arc<StoreValue>)>,
        parallelism_config: &crate::engine::store::ParallelismConfig,
        active_requests: usize,
    ) {
        let use_parallel = parallelism_config.should_use_parallel(new.len(), active_requests);

        let iter = if use_parallel {
            new.into_par_iter()
                .flat_map(|(store_key_id, store_value)| {
                    // Clone the Arc to get access to the inner value
                    let value_clone = Arc::clone(&store_value);
                    value_clone
                        .value
                        .clone()
                        .into_par_iter()
                        .map(move |(key, val)| {
                            let allowed_keys = self.allowed_predicates.pin();
                            allowed_keys
                                .contains(&key)
                                .then_some((store_key_id, key, val))
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
                })
        } else {
            let mut result = HashMap::new();
            for (store_key_id, store_value) in new {
                let allowed_keys = self.allowed_predicates.pin();
                for (key, val) in store_value.value.iter() {
                    if allowed_keys.contains(key) {
                        result
                            .entry(key.clone())
                            .or_insert_with(Vec::new)
                            .push((val.to_owned(), store_key_id));
                    }
                }
            }
            result
        };

        let predicate_values = self.inner.pin();
        for (key, val) in iter {
            if let Some(existing) = predicate_values.get(&key) {
                existing.add(val, parallelism_config, active_requests);
                continue;
            }

            // Build before publication; a competing creator may still win try_insert.
            let pred = PredicateIndex::init(val.clone(), parallelism_config, active_requests);

            if let Err(existing_predicate) = predicate_values.try_insert(key, pred) {
                existing_predicate
                    .current
                    .add(val, parallelism_config, active_requests);
            };
        }
    }
}
