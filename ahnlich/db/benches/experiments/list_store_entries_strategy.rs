use std::collections::{BTreeSet, HashSet};
use std::num::NonZeroUsize;
use std::ops::Bound::{Excluded, Unbounded};
use std::time::Duration;

use ahnlich_types::utils::StoreKeyId;
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use crossbeam_skiplist::SkipSet;
use parking_lot::RwLock;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

type LockedBTreeSet = RwLock<BTreeSet<StoreKeyId>>;

const STORE_SIZES: [usize; 2] = [100_000, 1_000_000];
const PAGE_LIMITS: [usize; 2] = [100, 1_000];

const SELECTIVITIES: [(usize, &str); 6] = [
    (1, "0_01_percent"),
    (10, "0_1_percent"),
    (100, "1_percent"),
    (1_000, "10_percent"),
    (5_000, "50_percent"),
    (10_000, "100_percent"),
];

trait OrderedStoreKeyIds {
    fn matching_page(
        &self,
        cursor: Option<StoreKeyId>,
        matching_ids: &HashSet<StoreKeyId>,
        limit: NonZeroUsize,
    ) -> Vec<StoreKeyId>;
}

impl OrderedStoreKeyIds for LockedBTreeSet {
    fn matching_page(
        &self,
        cursor: Option<StoreKeyId>,
        matching_ids: &HashSet<StoreKeyId>,
        limit: NonZeroUsize,
    ) -> Vec<StoreKeyId> {
        let index = self.read();
        let page_size = limit.get() + 1;

        match cursor {
            Some(cursor) => index
                .range((Excluded(cursor), Unbounded))
                .filter(|id| matching_ids.contains(id))
                .take(page_size)
                .copied()
                .collect(),
            None => index
                .iter()
                .filter(|id| matching_ids.contains(id))
                .take(page_size)
                .copied()
                .collect(),
        }
    }
}

impl OrderedStoreKeyIds for SkipSet<StoreKeyId> {
    fn matching_page(
        &self,
        cursor: Option<StoreKeyId>,
        matching_ids: &HashSet<StoreKeyId>,
        limit: NonZeroUsize,
    ) -> Vec<StoreKeyId> {
        let page_size = limit.get() + 1;

        match cursor {
            Some(cursor) => self
                .range((Excluded(cursor), Unbounded))
                .filter_map(|entry| {
                    let id = *entry.value();
                    matching_ids.contains(&id).then_some(id)
                })
                .take(page_size)
                .collect(),
            None => self
                .iter()
                .filter_map(|entry| {
                    let id = *entry.value();
                    matching_ids.contains(&id).then_some(id)
                })
                .take(page_size)
                .collect(),
        }
    }
}

fn generate_ids(count: usize) -> Vec<StoreKeyId> {
    let mut random = StdRng::seed_from_u64(42);
    let mut seen = HashSet::with_capacity(count);
    let mut ids = Vec::with_capacity(count);

    while ids.len() < count {
        let id = StoreKeyId(random.next_u64());

        if seen.insert(id) {
            ids.push(id);
        }
    }

    ids
}

fn build_btree(ids: &[StoreKeyId]) -> LockedBTreeSet {
    RwLock::new(ids.iter().copied().collect())
}

fn build_skipset(ids: &[StoreKeyId]) -> SkipSet<StoreKeyId> {
    let index = SkipSet::new();

    for id in ids {
        index.insert(*id);
    }

    index
}

fn build_matching_ids(ids: &[StoreKeyId], selectivity_basis_points: usize) -> HashSet<StoreKeyId> {
    let matching_count = ids
        .len()
        .saturating_mul(selectivity_basis_points)
        .div_ceil(10_000)
        .max(1);

    ids.iter().take(matching_count).copied().collect()
}

fn sorted_matching_page(
    matching_ids: &HashSet<StoreKeyId>,
    cursor: Option<StoreKeyId>,
    limit: NonZeroUsize,
) -> Vec<StoreKeyId> {
    let mut ids = matching_ids
        .iter()
        .copied()
        .filter(|id| cursor.is_none_or(|cursor| *id > cursor))
        .collect::<Vec<_>>();

    ids.sort_unstable();
    ids.truncate(limit.get() + 1);
    ids
}

fn benchmark_query_strategies(criterion: &mut Criterion) {
    for store_size in STORE_SIZES {
        let ids = generate_ids(store_size);

        let mut sorted_ids = ids.clone();
        sorted_ids.sort_unstable();

        let cursor = Some(sorted_ids[sorted_ids.len() / 2]);
        let btree = build_btree(&ids);
        let skipset = build_skipset(&ids);

        for limit in PAGE_LIMITS {
            let limit = NonZeroUsize::new(limit).expect("page limit must be non-zero");

            for (selectivity_basis_points, selectivity_label) in SELECTIVITIES {
                let matching_ids = build_matching_ids(&ids, selectivity_basis_points);

                let mut group = criterion.benchmark_group(format!(
                    "list_strategy/{store_size}/limit_{}/selectivity_{selectivity_label}",
                    limit.get(),
                ));

                group.bench_function("sort_matching_ids", |benchmark| {
                    benchmark.iter(|| {
                        black_box(sorted_matching_page(
                            black_box(&matching_ids),
                            black_box(cursor),
                            limit,
                        ))
                    });
                });

                group.bench_function("btree_scan", |benchmark| {
                    benchmark.iter(|| {
                        black_box(btree.matching_page(
                            black_box(cursor),
                            black_box(&matching_ids),
                            limit,
                        ))
                    });
                });

                group.bench_function("skipset_scan", |benchmark| {
                    benchmark.iter(|| {
                        black_box(skipset.matching_page(
                            black_box(cursor),
                            black_box(&matching_ids),
                            limit,
                        ))
                    });
                });

                group.finish();
            }
        }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(15)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = benchmark_query_strategies
}

criterion_main!(benches);
