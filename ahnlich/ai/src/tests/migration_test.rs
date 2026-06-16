use crate::engine::store::{AIStoreHandler, AIStores};
use ahnlich_types::schema::Schema;
use serde_json::json;

#[test]
fn test_ai_migrate_old_flat_snapshot_via_json() {
    // Construct old-format JSON: HashMap<StoreName, AIStore>
    // Where AIStore has name (serialized as string), query_model, index_model, store_original
    let old_format = json!({
        "test_ai_store": {
            "name": "test_ai_store",
            "query_model": "AllMiniLmL6V2",
            "index_model": "AllMiniLmL6V2",
            "store_original": false
        }
    });

    let json_bytes = serde_json::to_vec(&old_format).expect("Failed to serialize old format");

    let migrated: AIStores = AIStoreHandler::load_snapshot(&json_bytes).expect("Migration failed");

    let guard = migrated.guard();
    let inner = migrated
        .get(&Schema::default(), &guard)
        .expect("No public schema after migration");
    assert_eq!(inner.len(), 1, "Expected 1 AI store under public schema");
}

#[test]
fn test_ai_migrate_old_flat_snapshot_via_file() {
    // Write fixture file
    let old_format = json!({
        "test_ai_store": {
            "name": "test_ai_store",
            "query_model": "AllMiniLmL6V2",
            "index_model": "AllMiniLmL6V2",
            "store_original": false
        }
    });

    let fixture_dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures");
    std::fs::create_dir_all(&fixture_dir).expect("Failed to create fixtures dir");
    let fixture_path = fixture_dir.join("ai_old_flat_snapshot.json");
    let json_bytes = serde_json::to_vec_pretty(&old_format).expect("Failed to serialize");
    std::fs::write(&fixture_path, &json_bytes).expect("Failed to write fixture");

    // Read back and migrate
    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");
    let migrated: AIStores =
        AIStoreHandler::load_snapshot(&read_bytes).expect("Migration of fixture failed");

    let guard = migrated.guard();
    let inner = migrated
        .get(&Schema::default(), &guard)
        .expect("No public schema after migration");
    assert_eq!(inner.len(), 1, "Expected 1 AI store under public schema");
}

#[test]
fn test_ai_migrate_from_committed_fixture() {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures")
        .join("ai_old_flat_snapshot.json");

    assert!(
        fixture_path.exists(),
        "Committed fixture not found: {:?}",
        fixture_path
    );

    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

    let migrated: AIStores =
        AIStoreHandler::load_snapshot(&read_bytes).expect("Migration of fixture failed");

    let guard = migrated.guard();
    let inner = migrated
        .get(&Schema::default(), &guard)
        .expect("No public schema after migration");
    assert_eq!(inner.len(), 1, "Expected 1 AI store under public schema");
    let pinned = inner.pin();
    let (key, _) = pinned.iter().next().expect("No store in result");
    assert_eq!(
        key.value, "test_ai_store",
        "Store name preserved after migration"
    );
}

#[test]
fn test_ai_load_v2_snapshot() {
    let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures")
        .join("ai_v2_snapshot.json");

    assert!(
        fixture_path.exists(),
        "V2 fixture not found: {:?}",
        fixture_path
    );

    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");
    let loaded: AIStores = AIStoreHandler::load_snapshot(&read_bytes).expect("V2 load failed");
    let guard = loaded.guard();
    let inner = loaded
        .get(&Schema::default(), &guard)
        .expect("No public schema after V2 load");
    assert_eq!(inner.len(), 1, "Expected 1 AI store under public schema");
    let pinned = inner.pin();
    let (key, _) = pinned.iter().next().expect("No store in result");
    assert_eq!(key.value, "test_ai_store", "Store name preserved in V2");
}
