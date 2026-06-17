use crate::engine::store::StoreHandler;
use crate::engine::store::Stores;
use ahnlich_types::schema::Schema;
use serde_json::json;

#[test]
fn test_db_migrate_old_flat_snapshot_via_json() {
    let old_format = json!({
        "fixture_store": {
            "dimension": 3,
            "id_to_value": {},
            "predicate_indices": {
                "inner": {},
                "allowed_predicates": []
            },
            "non_linear_indices": {
                "algorithm_to_index": {}
            },
            "cached_len": 0,
            "cached_size_bytes": 0,
            "size_dirty": true
        }
    });

    let json_bytes = serde_json::to_vec(&old_format).expect("Failed to serialize old format");
    let migrated: Stores = StoreHandler::load_snapshot(&json_bytes).expect("Migration failed");
    let guard = migrated.guard();
    let inner = migrated
        .get(&Schema::default(), &guard)
        .expect("No public schema after migration");
    assert_eq!(inner.len(), 1, "Expected 1 DB store under public schema");
}

#[test]
fn test_db_migrate_from_committed_fixture() {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures")
        .join("db_old_flat_snapshot.json");

    assert!(
        fixture_path.exists(),
        "Committed fixture not found: {:?}",
        fixture_path
    );

    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");
    let migrated: Stores =
        StoreHandler::load_snapshot(&read_bytes).expect("Migration of fixture failed");
    let guard = migrated.guard();
    let inner = migrated
        .get(&Schema::default(), &guard)
        .expect("No public schema after migration");
    assert_eq!(inner.len(), 1, "Expected 1 DB store under public schema");
    let pinned = inner.pin();
    let (key, _) = pinned.iter().next().expect("No store in result");
    assert_eq!(
        key.value, "fixture_store",
        "Store name preserved after migration"
    );
}

#[test]
#[ignore]
fn generate_db_v2_fixture() {
    use crate::engine::store::StoreHandler;
    use ahnlich_types::algorithm::nonlinear::KdTreeConfig;
    use ahnlich_types::algorithm::nonlinear::non_linear_index;
    use ahnlich_types::keyval::{StoreKey, StoreName, StoreValue};
    use ahnlich_types::metadata::{MetadataValue, metadata_value};
    use ahnlich_types::schema::Schema;
    use std::collections::{HashMap, HashSet};
    use std::num::NonZeroUsize;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;
    use utils::persistence::AhnlichPersistenceUtils;

    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let predicates = vec!["category".to_string(), "color".to_string()];
    let mut non_linear_indices: HashSet<non_linear_index::Index> = HashSet::new();
    non_linear_indices.insert(non_linear_index::Index::Kdtree(KdTreeConfig {}));
    handler
        .create_store(
            StoreName {
                value: "fixture_store".to_string(),
            },
            &Schema::default(),
            NonZeroUsize::new(3).unwrap(),
            predicates,
            non_linear_indices,
            false,
        )
        .unwrap();

    // Insert entry 1
    let key1 = StoreKey {
        key: vec![0.5f32, 0.1, 0.8],
    };
    let mut meta1 = HashMap::new();
    meta1.insert(
        "name".to_string(),
        MetadataValue {
            value: Some(metadata_value::Value::RawString("item1".to_string())),
        },
    );
    meta1.insert(
        "category".to_string(),
        MetadataValue {
            value: Some(metadata_value::Value::RawString("fruit".to_string())),
        },
    );
    meta1.insert(
        "color".to_string(),
        MetadataValue {
            value: Some(metadata_value::Value::RawString("red".to_string())),
        },
    );
    handler
        .set_in_store(
            &StoreName {
                value: "fixture_store".to_string(),
            },
            vec![(key1, StoreValue { value: meta1 })],
        )
        .unwrap();

    // Insert entry 2
    let key2 = StoreKey {
        key: vec![0.2f32, 0.7, 0.3],
    };
    let mut meta2 = HashMap::new();
    meta2.insert(
        "name".to_string(),
        MetadataValue {
            value: Some(metadata_value::Value::RawString("item2".to_string())),
        },
    );
    meta2.insert(
        "category".to_string(),
        MetadataValue {
            value: Some(metadata_value::Value::RawString("vegetable".to_string())),
        },
    );
    meta2.insert(
        "color".to_string(),
        MetadataValue {
            value: Some(metadata_value::Value::RawString("green".to_string())),
        },
    );
    handler
        .set_in_store(
            &StoreName {
                value: "fixture_store".to_string(),
            },
            vec![(key2, StoreValue { value: meta2 })],
        )
        .unwrap();

    let snapshot = handler.get_snapshot();
    let json = serde_json::to_string_pretty(&snapshot).expect("Serialization failed");
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures")
        .join("db_v2_snapshot.json");
    std::fs::write(&fixture_path, &json).expect("Failed to write fixture");
    eprintln!("Generated fixture at: {:?}", fixture_path);
}

#[test]
fn test_db_load_v2_snapshot() {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures")
        .join("db_v2_snapshot.json");

    assert!(
        fixture_path.exists(),
        "V2 fixture not found: {:?}",
        fixture_path
    );

    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");
    let loaded: Stores = StoreHandler::load_snapshot(&read_bytes).expect("V2 load failed");
    let guard = loaded.guard();
    let inner = loaded
        .get(&Schema::default(), &guard)
        .expect("No public schema after V2 load");
    assert_eq!(inner.len(), 1, "Expected 1 DB store under public schema");
    let pinned = inner.pin();
    let (key, _store) = pinned.iter().next().expect("No store in result");
    assert_eq!(key.value, "fixture_store", "Store name preserved in V2");
}
