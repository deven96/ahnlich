use std::collections::BTreeSet;
use std::hint::black_box;

use ahnlich_types::utils::StoreKeyId;
use crossbeam_skiplist::SkipSet;
use parking_lot::RwLock;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};
use utils::allocator::GLOBAL_ALLOCATOR;

const ENTRY_COUNTS: [usize; 3] = [10_000, 100_000, 1_000_000];

fn generate_ids(count: usize) -> Vec<StoreKeyId> {
    let mut random = StdRng::seed_from_u64(42);

    (0..count).map(|_| StoreKeyId(random.next_u64())).collect()
}

fn measure_allocation<T>(build: impl FnOnce() -> T) -> usize {
    let before = GLOBAL_ALLOCATOR.allocated();
    let index = build();

    black_box(&index);

    let after = GLOBAL_ALLOCATOR.allocated();
    let allocated = after.saturating_sub(before);

    drop(index);

    allocated
}

fn main() {
    println!(
        "{:<12} {:<12} {:>16} {:>16}",
        "entries", "index", "allocated bytes", "bytes per ID"
    );

    for entry_count in ENTRY_COUNTS {
        let ids = generate_ids(entry_count);

        let btree_bytes =
            measure_allocation(|| RwLock::new(ids.iter().copied().collect::<BTreeSet<_>>()));

        let skipset_bytes = measure_allocation(|| {
            let index = SkipSet::new();

            for id in &ids {
                index.insert(*id);
            }

            index
        });

        println!(
            "{:<12} {:<12} {:>16} {:>16.2}",
            entry_count,
            "btree",
            btree_bytes,
            btree_bytes as f64 / entry_count as f64
        );

        println!(
            "{:<12} {:<12} {:>16} {:>16.2}",
            entry_count,
            "skipset",
            skipset_bytes,
            skipset_bytes as f64 / entry_count as f64
        );
    }
}
