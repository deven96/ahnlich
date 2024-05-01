mod predicate;

use flurry::HashMap as ConcurrentHashMap;
use flurry::HashSet as ConcurrentHashSet;
use ndarray::Array1;
use predicate::PredicateIndices;
use std::collections::HashMap as StdHashMap;
use std::num::NonZeroU32;

/// A store key is always an f32 one dimensional array
#[derive(Debug)]
struct StoreKey(Array1<f32>);

/// A hash of Store key, this is more preferable when passing around references as arrays can be
/// potentially larger
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
struct StoreKeyId(String);

/// A store value for now is a simple key value pair of strings
type StoreValue = StdHashMap<String, String>;

/// A Store is a single database containing multiple N*1 arrays where N is the dimension of the
/// store to which all arrays must conform
#[derive(Debug)]
struct Store {
    dimension: NonZeroU32,
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    id_to_value: ConcurrentHashMap<StoreKeyId, StoreValue>,
    /// We store keys separately as we can always compute their hash and find the StoreValue from
    /// there
    keys: ConcurrentHashSet<StoreKey>,
    /// Indices to filter for the store
    predicate_indices: PredicateIndices,
}

impl Store {
    /// Creates a new empty store
    fn create(dimension: NonZeroU32, predicates: Vec<String>) -> Self {
        Self {
            dimension,
            id_to_value: ConcurrentHashMap::new(),
            keys: ConcurrentHashSet::new(),
            predicate_indices: PredicateIndices::init(predicates),
        }
    }

    /// Returns the number of key value pairs in the store
    fn len(&self) -> usize {
        self.id_to_value.pin().len()
    }
}
