//! Benchmark facade for PredicateIndices ingestion implementations.
//!
//! Control calls the preserved `add` implementation and candidate calls the production
//! existing-index implementation.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

pub type Entries = Vec<(StoreKeyId, Arc<StoreValue>)>;
pub type Memberships = BTreeMap<String, BTreeMap<MetadataValue, BTreeSet<StoreKeyId>>>;

#[derive(Clone, Copy)]
pub enum Variant {
    Control,
    ExistingIndex,
}

#[derive(Clone, Copy)]
pub enum CollectorSharingVariant {
    Control,
    Candidate,
}

enum CollectorIndex {
    Control(PredicateIndex),
    Candidate(SharedPredicateIndex),
}

pub struct PredicateCollectorFixture {
    index: CollectorIndex,
    config: crate::engine::store::ParallelismConfig,
    active_requests: usize,
}

impl PredicateCollectorFixture {
    pub fn new(
        variant: CollectorSharingVariant,
        initial: Vec<(MetadataValue, StoreKeyId)>,
        config: crate::engine::store::ParallelismConfig,
        active_requests: usize,
    ) -> Self {
        let index = match variant {
            CollectorSharingVariant::Control => {
                CollectorIndex::Control(PredicateIndex::init(initial, &config, active_requests))
            }
            CollectorSharingVariant::Candidate => CollectorIndex::Candidate(
                SharedPredicateIndex::init(initial, &config, active_requests),
            ),
        };
        Self {
            index,
            config,
            active_requests,
        }
    }

    pub fn add(&self, entries: Vec<(MetadataValue, StoreKeyId)>) {
        match &self.index {
            CollectorIndex::Control(index) => {
                index.add(entries, &self.config, self.active_requests)
            }
            CollectorIndex::Candidate(index) => {
                index.add(entries, &self.config, self.active_requests)
            }
        }
    }

    pub fn memberships(&self) -> BTreeMap<MetadataValue, BTreeSet<StoreKeyId>> {
        let entries = match &self.index {
            CollectorIndex::Control(index) => &index.0,
            CollectorIndex::Candidate(index) => &index.inner,
        };
        entries
            .pin()
            .iter()
            .map(|(value, ids)| (value.clone(), ids.pin().iter().copied().collect()))
            .collect()
    }
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
                    .inner
                    .pin()
                    .iter()
                    .map(|(value, ids)| (value.clone(), ids.pin().iter().copied().collect()))
                    .collect();
                (key.clone(), buckets)
            })
            .collect()
    }
}
