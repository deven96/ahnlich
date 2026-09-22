// Each benchmark target uses a different subset of this shared fixture.
#![allow(dead_code)]

use papaya::{HashMap, HashSet};
use rayon::prelude::*;
use seize::Collector;
use std::sync::Arc;

type PrimaryMap = HashMap<u64, u64>;
type NonLinearMap = HashMap<u8, u64>;
type PredicateBuckets = HashMap<u64, HashSet<u64>>;
type PredicateMap = HashMap<usize, PredicateIndex>;

#[derive(Clone, Copy, Debug)]
pub(crate) enum SharingScope {
    Independent,
    PerIndex,
    PredicateWide,
    StoreWide,
}

impl SharingScope {
    pub(crate) const ALL: [Self; 4] = [
        Self::Independent,
        Self::PerIndex,
        Self::PredicateWide,
        Self::StoreWide,
    ];

    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Independent => "control",
            Self::PerIndex => "per_index",
            Self::PredicateWide => "predicate_wide",
            Self::StoreWide => "store_wide",
        }
    }

    pub(crate) const fn expected_collectors(self, predicate_count: usize, buckets: usize) -> usize {
        match self {
            Self::Independent => 4 + predicate_count + buckets,
            Self::PerIndex => 4 + predicate_count,
            Self::PredicateWide => 3,
            Self::StoreWide => 1,
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Scenario {
    pub(crate) label: &'static str,
    pub(crate) entries: usize,
    pub(crate) predicates: usize,
    pub(crate) cardinality: usize,
}

impl Scenario {
    pub(crate) fn expected_buckets(self) -> usize {
        self.predicates * self.entries.min(self.cardinality)
    }
}

pub(crate) const TIMING_SCENARIOS: [Scenario; 6] = [
    Scenario {
        label: "empty",
        entries: 0,
        predicates: 0,
        cardinality: 1,
    },
    Scenario {
        label: "one_index_low_cardinality_10k",
        entries: 10_000,
        predicates: 1,
        cardinality: 10,
    },
    Scenario {
        label: "one_index_unique_10k",
        entries: 10_000,
        predicates: 1,
        cardinality: 10_000,
    },
    Scenario {
        label: "four_indexes_low_cardinality_10k",
        entries: 10_000,
        predicates: 4,
        cardinality: 10,
    },
    Scenario {
        label: "four_indexes_unique_10k",
        entries: 10_000,
        predicates: 4,
        cardinality: 10_000,
    },
    Scenario {
        label: "one_index_unique_100k",
        entries: 100_000,
        predicates: 1,
        cardinality: 100_000,
    },
];

pub(crate) const MEMORY_SCENARIOS: [Scenario; 5] = [
    TIMING_SCENARIOS[0],
    TIMING_SCENARIOS[1],
    TIMING_SCENARIOS[2],
    TIMING_SCENARIOS[4],
    TIMING_SCENARIOS[5],
];

fn new_map<K, V>(collector: Option<&Arc<Collector>>) -> HashMap<K, V>
where
    K: Send + 'static,
    V: Send + 'static,
{
    let builder = HashMap::builder().capacity(1);
    match collector {
        Some(collector) => builder.shared_collector(Arc::clone(collector)).build(),
        None => builder.build(),
    }
}

fn new_set<K>(collector: Option<&Arc<Collector>>) -> HashSet<K>
where
    K: Send + 'static,
{
    let builder = HashSet::builder().capacity(1);
    match collector {
        Some(collector) => builder.shared_collector(Arc::clone(collector)).build(),
        None => builder.build(),
    }
}

struct PredicateIndex {
    buckets: PredicateBuckets,
    collector: Option<Arc<Collector>>,
}

impl PredicateIndex {
    fn new(collector: Option<Arc<Collector>>) -> Self {
        Self {
            buckets: new_map(collector.as_ref()),
            collector,
        }
    }

    fn insert(&self, predicate_value: u64, store_key_id: u64) {
        let buckets = self.buckets.pin();
        if let Some(store_key_ids) = buckets.get(&predicate_value) {
            store_key_ids.pin().insert(store_key_id);
            return;
        }

        let store_key_ids = new_set(self.collector.as_ref());
        store_key_ids.pin().insert(store_key_id);
        if let Err(existing) = buckets.try_insert(predicate_value, store_key_ids) {
            existing.current.pin().insert(store_key_id);
        }
    }

    fn contains(&self, predicate_value: u64, store_key_id: u64) -> bool {
        let buckets = self.buckets.pin();
        buckets
            .get(&predicate_value)
            .is_some_and(|store_key_ids| store_key_ids.pin().contains(&store_key_id))
    }

    fn clear_store_keys(&self) {
        let buckets = self.buckets.pin();
        for store_key_ids in buckets.values() {
            store_key_ids.pin().clear();
        }
    }

    fn bucket_count(&self) -> usize {
        self.buckets.len()
    }
}

pub(crate) struct BenchStore {
    primary: PrimaryMap,
    non_linear: NonLinearMap,
    allowed_predicates: HashSet<usize>,
    predicates: PredicateMap,
}

impl BenchStore {
    pub(crate) fn new(scope: SharingScope, predicate_count: usize) -> Self {
        let store_collector =
            matches!(scope, SharingScope::StoreWide).then(|| Arc::new(Collector::new()));
        let predicate_collector = match scope {
            SharingScope::PredicateWide => Some(Arc::new(Collector::new())),
            SharingScope::StoreWide => store_collector.clone(),
            SharingScope::Independent | SharingScope::PerIndex => None,
        };

        let primary = new_map(store_collector.as_ref());
        let non_linear = new_map(store_collector.as_ref());
        let allowed_predicates = new_set(predicate_collector.as_ref());
        let predicates = new_map(predicate_collector.as_ref());

        let allowed = allowed_predicates.pin();
        let predicate_maps = predicates.pin();
        for predicate in 0..predicate_count {
            allowed.insert(predicate);
            let index_collector = match scope {
                SharingScope::PerIndex => Some(Arc::new(Collector::new())),
                SharingScope::PredicateWide | SharingScope::StoreWide => {
                    predicate_collector.clone()
                }
                SharingScope::Independent => None,
            };
            predicate_maps.insert(predicate, PredicateIndex::new(index_collector));
        }
        drop(predicate_maps);
        drop(allowed);

        Self {
            primary,
            non_linear,
            allowed_predicates,
            predicates,
        }
    }

    pub(crate) fn ingest(&self, start: usize, entries: usize, cardinality: usize) {
        assert!(cardinality > 0);
        let primary = self.primary.pin();
        let predicates = self.predicates.pin();

        for entry in start..start + entries {
            let store_key_id = entry as u64;
            primary.insert(store_key_id, store_key_id);
            for (_, index) in predicates.iter() {
                index.insert((entry % cardinality) as u64, store_key_id);
            }
        }
    }

    pub(crate) fn ingest_predicates_parallel(
        &self,
        start: usize,
        entries: usize,
        cardinality: usize,
    ) {
        assert!(cardinality > 0);
        (0..self.predicates.len())
            .into_par_iter()
            .for_each(|predicate| {
                let predicates = self.predicates.pin();
                let index = predicates.get(&predicate).expect("predicate must exist");
                for entry in start..start + entries {
                    index.insert((entry % cardinality) as u64, entry as u64);
                }
            });
    }

    pub(crate) fn scan_primary(&self) -> u64 {
        self.primary
            .pin()
            .iter()
            .fold(0, |checksum, (key, value)| checksum ^ key ^ value)
    }

    pub(crate) fn predicate_lookup(&self, predicate: usize, value: u64, key: u64) -> bool {
        self.predicates
            .pin()
            .get(&predicate)
            .is_some_and(|index| index.contains(value, key))
    }

    pub(crate) fn clear_store_keys(&self) {
        self.primary.pin().clear();
        let predicates = self.predicates.pin();
        for (_, index) in predicates.iter() {
            index.clear_store_keys();
        }
    }

    pub(crate) fn primary_len(&self) -> usize {
        self.primary.len()
    }

    pub(crate) fn bucket_count(&self) -> usize {
        self.predicates
            .pin()
            .values()
            .map(PredicateIndex::bucket_count)
            .sum()
    }

    pub(crate) fn configured_predicates(&self) -> usize {
        debug_assert_eq!(self.allowed_predicates.len(), self.predicates.len());
        self.allowed_predicates.len()
    }

    pub(crate) fn non_linear_len(&self) -> usize {
        self.non_linear.len()
    }
}

pub(crate) fn populated_store(scope: SharingScope, scenario: Scenario) -> BenchStore {
    let store = BenchStore::new(scope, scenario.predicates);
    store.ingest(0, scenario.entries, scenario.cardinality);
    assert_eq!(store.primary_len(), scenario.entries);
    assert_eq!(store.bucket_count(), scenario.expected_buckets());
    assert_eq!(store.configured_predicates(), scenario.predicates);
    assert_eq!(store.non_linear_len(), 0);
    store
}
