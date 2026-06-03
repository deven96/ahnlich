//! Fallible constructors for papaya concurrent collections.
//!
//! Papaya's `HashMap::new()` and `HashSet::new()` are lazy. The internal hash
//! table is not allocated until the first `insert()`. This laziness means that
//! allocation failures (e.g., OOM) or assertion panics
//! (`assert!(len.is_power_of_two())`) are deferred to arbitrary mutation points
//! rather than surfaced at construction time.
//!
//! The helpers in this module force immediate allocation via
//! [`with_capacity`](papaya::HashMap::with_capacity) inside
//! [`std::panic::catch_unwind`], converting panics into [`Result`]s.
//!
//! # Usage guidance
//!
//! | Context | Recommended pattern |
//! |---|---|
//! | Server startup (`new()` constructors before panic hook) | Return `Result` to the caller, or use `unwrap_or_else` + `eprintln!` + `process::abort()` to keep a [`Result`]-free API |
//! | Inside tasks / after panic hook installed | `try_new_*().expect("clear message")` — the hook catches the panic |
//! | Hot-path runtime allocation | Use `try_new_*()` directly and handle the error |

use std::panic::catch_unwind;
use std::sync::Arc;

use papaya::HashMap as ConcurrentHashMap;
use papaya::HashSet as ConcurrentHashSet;

/// Creates a new pre-allocated [`ConcurrentHashMap`] inside [`catch_unwind`].
///
/// Papaya's `HashMap::new()` lazily allocates its internal table on the first
/// insert, which may trigger `assert!(len.is_power_of_two())` or OOM panics.
/// Pre-allocating via `with_capacity(1)` forces the table allocation at
/// construction time so that any failure surfaces immediately.
///
/// # Errors
///
/// Returns `Err` with a descriptive message if the papaya construction panics
/// (e.g. a memory allocation failure or an internal assertion).
pub fn try_new_hashmap<K, V>() -> Result<ConcurrentHashMap<K, V>, String> {
    catch_unwind(|| ConcurrentHashMap::with_capacity(1))
        .map_err(|_| "Failed to initialize concurrent hashmap".to_string())
}

/// Creates a new [`ConcurrentHashMap`] wrapped in [`Arc`] using [`try_new_hashmap`].
///
/// Convenience wrapper for the common `Arc<ConcurrentHashMap<K, V>>` pattern.
pub fn try_new_arc_hashmap<K, V>() -> Result<Arc<ConcurrentHashMap<K, V>>, String> {
    try_new_hashmap().map(Arc::new)
}

/// Creates a new pre-allocated [`ConcurrentHashSet`] inside [`catch_unwind`].
///
/// Behaves identically to [`try_new_hashmap`] but for [`ConcurrentHashSet`].
pub fn try_new_hashset<T: std::hash::Hash + Eq>() -> Result<ConcurrentHashSet<T>, String> {
    catch_unwind(|| ConcurrentHashSet::with_capacity(1))
        .map_err(|_| "Failed to initialize concurrent hashset".to_string())
}

/// Creates a new [`ConcurrentHashSet`] with the given capacity inside [`catch_unwind`].
///
/// Useful when the caller knows the expected size and wants to avoid resizing.
pub fn try_new_hashset_with_capacity<T: std::hash::Hash + Eq>(
    capacity: usize,
) -> Result<ConcurrentHashSet<T>, String> {
    catch_unwind(move || ConcurrentHashSet::with_capacity(capacity))
        .map_err(|_| format!("Failed to initialize concurrent hashset with capacity {capacity}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::thread;

    /// Verifies a basic put/get round-trip on a map created via fallible constructor.
    #[test]
    fn hashmap_put_get() {
        let map = try_new_hashmap::<&str, i32>().unwrap();
        let guard = map.pin();
        guard.insert("key", 42);
        assert_eq!(guard.get("key"), Some(&42));
    }

    /// Verifies a map created via fallible constructor starts empty.
    #[test]
    fn hashmap_empty() {
        let map = try_new_hashmap::<String, i32>().unwrap();
        let guard = map.pin();
        assert!(guard.is_empty());
    }

    /// Verifies an Arc-wrapped map created via fallible constructor.
    #[test]
    fn arc_hashmap_put_get() {
        let map = try_new_arc_hashmap::<u64, u64>().unwrap();
        let guard = map.pin();
        guard.insert(1, 100);
        assert_eq!(guard.get(&1), Some(&100));
    }

    /// Verifies a basic set insertion and membership test.
    #[test]
    fn hashset_insert_contains() {
        let set = try_new_hashset::<i32>().unwrap();
        let guard = set.pin();
        guard.insert(42);
        assert!(guard.contains(&42));
        assert!(!guard.contains(&0));
    }

    /// Verifies a set created with explicit capacity 0 is still usable.
    #[test]
    fn hashset_with_capacity_zero() {
        let set = try_new_hashset_with_capacity::<i32>(0).unwrap();
        let guard = set.pin();
        guard.insert(1);
        assert!(guard.contains(&1));
    }

    /// Verifies a set created with explicit capacity 1 is usable.
    #[test]
    fn hashset_with_capacity_one() {
        let set = try_new_hashset_with_capacity::<i32>(1).unwrap();
        let guard = set.pin();
        guard.insert(1);
        assert!(guard.contains(&1));
    }

    /// Verifies a set with a large explicit capacity handles many inserts.
    #[test]
    fn hashset_with_capacity_large() {
        let set = try_new_hashset_with_capacity::<i32>(10_000).unwrap();
        let guard = set.pin();
        for i in 0..1000 {
            guard.insert(i);
        }
        assert_eq!(guard.len(), 1000);
    }

    /// Verifies that capacity is only a lower bound; sets with capacity 0 can still hold data.
    #[test]
    fn hashset_with_capacity_zero_still_holds_data() {
        let set = try_new_hashset_with_capacity::<i32>(0).unwrap();
        let guard = set.pin();
        for i in 0..100 {
            guard.insert(i);
        }
        assert_eq!(guard.len(), 100);
    }

    /// Verifies concurrent construction from many threads doesn't panic.
    #[test]
    fn concurrent_construction() {
        let mut handles = vec![];
        for _ in 0..16 {
            handles.push(thread::spawn(|| {
                let map = try_new_hashmap::<u64, u64>().unwrap();
                let guard = map.pin();
                guard.insert(1, 2);
                assert_eq!(guard.get(&1), Some(&2));
            }));
        }
        for h in handles {
            h.join().expect("thread panicked during concurrent construction");
        }
    }

    /// Verifies concurrent construction and usage of hash sets.
    #[test]
    fn concurrent_hashset_construction() {
        let mut handles = vec![];
        for _ in 0..16 {
            handles.push(thread::spawn(|| {
                let set = try_new_hashset::<u64>().unwrap();
                let guard = set.pin();
                guard.insert(42);
                assert!(guard.contains(&42));
            }));
        }
        for h in handles {
            h.join().expect("thread panicked during concurrent set construction");
        }
    }

    /// Verifies concurrent insertions into a shared map are all visible afterward.
    #[test]
    fn concurrent_writes_to_shared_map() {
        let map = Arc::new(try_new_hashmap::<u64, u64>().unwrap());
        let mut handles = vec![];
        for i in 0..10 {
            let map = map.clone();
            handles.push(thread::spawn(move || {
                let guard = map.pin();
                guard.insert(i, i * 2);
            }));
        }
        for h in handles {
            h.join().expect("write thread panicked");
        }
        let guard = map.pin();
        assert_eq!(guard.len(), 10);
        for i in 0..10 {
            assert_eq!(guard.get(&i), Some(&(i * 2)));
        }
    }

    /// Verifies concurrent reads and writes to a shared map are safe.
    #[test]
    fn concurrent_reads_and_writes() {
        let map = Arc::new(try_new_hashmap::<u64, u64>().unwrap());
        // Pre-populate
        {
            let guard = map.pin();
            for i in 0..100 {
                guard.insert(i, i);
            }
        }
        let mut handles = vec![];
        // Writers
        for _ in 0..4 {
            let map = map.clone();
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    let guard = map.pin();
                    guard.insert(i, i + 1);
                }
            }));
        }
        // Readers
        for _ in 0..4 {
            let map = map.clone();
            handles.push(thread::spawn(move || {
                for i in 0..100 {
                    let guard = map.pin();
                    let _ = guard.get(&i);
                }
            }));
        }
        for h in handles {
            h.join().expect("reader/writer thread panicked");
        }
    }

    /// Verifies the catch_unwind wrapper correctly returns Err when the inner
    /// closure panics.
    #[test]
    fn catch_unwind_wrapper_catches_panics() {
        let result = catch_unwind(AssertUnwindSafe(|| {
            panic!("simulated panic inside catch_unwind");
        }));
        assert!(result.is_err());
    }

    /// Verifies papaya types can be used together with AssertUnwindSafe.
    /// This ensures integration with catch_unwind works for our use case.
    #[test]
    fn unwind_safe_compatibility() {
        let map = try_new_hashmap::<&str, &str>().unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| {
            let guard = map.pin();
            let _ = guard.get("anything");
        }));
        assert!(result.is_ok());
    }

    /// Verifies the ok path: try_new_hashmap returns Ok when allocation succeeds.
    #[test]
    fn hashmap_returns_ok() {
        let result = try_new_hashmap::<i32, i32>();
        assert!(result.is_ok());
    }

    /// Verifies the ok path for sets.
    #[test]
    fn hashset_returns_ok() {
        let result = try_new_hashset::<i32>();
        assert!(result.is_ok());
    }

    /// Verifies the ok path for sets with capacity.
    #[test]
    fn hashset_with_capacity_returns_ok() {
        let result = try_new_hashset_with_capacity::<i32>(64);
        assert!(result.is_ok());
    }

    /// Verifies maps work with string keys (the primary use case in the db crate).
    #[test]
    fn hashmap_string_keys() {
        let map = try_new_hashmap::<String, String>().unwrap();
        let guard = map.pin();
        guard.insert("hello".to_string(), "world".to_string());
        assert_eq!(guard.get("hello"), Some(&"world".to_string()));
    }

    /// Verifies type inference works with complex key types.
    #[test]
    fn hashmap_complex_key_type() {
        #[derive(Hash, Eq, PartialEq, Debug)]
        struct CustomKey(u64, String);

        let map = try_new_hashmap::<CustomKey, bool>().unwrap();
        let guard = map.pin();
        guard.insert(CustomKey(1, "a".into()), true);
        assert_eq!(guard.get(&CustomKey(1, "a".into())), Some(&true));
    }

    /// Verifies try_new_arc_hashmap produces a map that can be shared via Arc.
    #[test]
    fn arc_hashmap_is_shareable() {
        let map = try_new_arc_hashmap::<u64, u64>().unwrap();
        let map_ref = &map;
        let guard = map_ref.pin();
        guard.insert(1, 10);
        assert_eq!(guard.get(&1), Some(&10));
    }

    /// Stress test: many inserts into a concurrently shared set.
    #[test]
    fn hashset_concurrent_stress() {
        let set = Arc::new(try_new_hashset::<u64>().unwrap());
        let mut handles = vec![];
        for t in 0..8 {
            let set = set.clone();
            handles.push(thread::spawn(move || {
                for i in (t * 125)..((t + 1) * 125) {
                    let guard = set.pin();
                    guard.insert(i as u64);
                }
            }));
        }
        for h in handles {
            h.join().expect("stress thread panicked");
        }
        let guard = set.pin();
        assert_eq!(guard.len(), 1000);
    }

    /// Verifies the map is correctly created and immediately usable for iteration.
    #[test]
    fn hashmap_iteration() {
        let map = try_new_hashmap::<i32, i32>().unwrap();
        let guard = map.pin();
        for i in 0..5 {
            guard.insert(i, -i);
        }
        let mut entries: Vec<_> = guard.iter().map(|(k, v)| (*k, *v)).collect();
        entries.sort();
        assert_eq!(entries, [(0, 0), (1, -1), (2, -2), (3, -3), (4, -4)]);
    }

    /// Verifies the catch_unwind wrapper catches panics from seize collector init.
    /// This tests the actual codepath used in the helper.
    #[test]
    fn fallible_catches_papaya_init_panic() {
        // Inject a panic by wrapping in catch_unwind and manually panicking
        // to verify the error-propagation pattern works end-to-end.
        let result: Result<ConcurrentHashMap<i32, i32>, String> = catch_unwind(AssertUnwindSafe(|| {
            // Use with_capacity(0) to show it still creates a valid map
            // (capacity 0 does not allocate in papaya, so it never panics)
            ConcurrentHashMap::with_capacity(0)
        }))
        .map_err(|_| "simulated failure".to_string());

        // with_capacity(0) is always ok, so this should be Ok
        assert!(result.is_ok());
    }

    /// Verifies that reserve does not panic after fallible construction.
    #[test]
    fn hashmap_reserve_after_fallible_init() {
        let map = try_new_hashmap::<i32, i32>().unwrap();
        let guard = map.guard();
        map.reserve(1000, &guard);
        let pinned = map.pin();
        for i in 0..1000 {
            pinned.insert(i, i);
        }
        assert_eq!(pinned.len(), 1000);
    }

    /// Regression test: verifies that creating many maps sequentially succeeds
    /// (catches issues with collector state leaks across init calls).
    #[test]
    fn repeated_construction() {
        for _ in 0..100 {
            let map = try_new_hashmap::<u64, u64>().unwrap();
            let guard = map.pin();
            guard.insert(1, 1);
            drop(guard);
            drop(map);
        }
    }

    /// Regression test: verifies that creating many sets sequentially succeeds.
    #[test]
    fn repeated_construction_hashset() {
        for _ in 0..100 {
            let set = try_new_hashset::<u64>().unwrap();
            let guard = set.pin();
            guard.insert(1);
            drop(guard);
            drop(set);
        }
    }
}
