use crate::engine::store::Stores;
use crate::engine::store::StoreHandler;
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
    let migrated: Stores = StoreHandler::load_snapshot(&read_bytes).expect("Migration of fixture failed");
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
    let (key, _) = pinned.iter().next().expect("No store in result");
    assert_eq!(
        key.value, "fixture_store",
        "Store name preserved in V2"
    );
}
