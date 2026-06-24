use crate::cli::server::SupportedModels;
use crate::engine::store::{AIStoreHandler, AIStores};
use ahnlich_types::ai::models::AiModel;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::schema::Schema;
use serde_json::Value;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use utils::persistence::AhnlichPersistenceUtils;

fn ai_fixture_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures")
}

fn populated_ai_fixture_handler() -> AIStoreHandler {
    let handler = AIStoreHandler::new(
        Arc::new(AtomicBool::new(false)),
        vec![SupportedModels::AllMiniLML6V2],
    );
    handler
        .create_store(
            StoreName {
                value: "test_ai_store".to_string(),
            },
            &Schema::default(),
            AiModel::AllMiniLmL6V2,
            AiModel::AllMiniLmL6V2,
            false,
            true,
        )
        .unwrap();
    handler
}

fn populated_ai_v2_snapshot_json() -> Value {
    serde_json::to_value(populated_ai_fixture_handler().get_snapshot())
        .expect("AI V2 snapshot serialization failed")
}

fn populated_ai_old_flat_snapshot_json() -> Value {
    populated_ai_v2_snapshot_json()
        .get("stores")
        .and_then(Value::as_object)
        .and_then(|schemas| schemas.get("public"))
        .cloned()
        .expect("generated AI snapshot should contain public schema")
}

fn assert_populated_ai_snapshot(migrated: AIStores) {
    let mut handler = AIStoreHandler::new(
        Arc::new(AtomicBool::new(false)),
        vec![SupportedModels::AllMiniLML6V2],
    );
    handler.use_snapshot(migrated);
    let store_info = handler
        .get_store(
            &StoreName {
                value: "test_ai_store".to_string(),
            },
            None,
        )
        .expect("fixture AI store should load after migration");

    assert_eq!(store_info.name, "test_ai_store");
    assert_eq!(store_info.query_model, AiModel::AllMiniLmL6V2 as i32);
    assert_eq!(store_info.index_model, AiModel::AllMiniLmL6V2 as i32);
    assert_eq!(store_info.schema, Some("public".to_string()));
}

#[test]
fn test_ai_migrate_old_flat_snapshot_via_json() {
    // Construct old-format JSON: HashMap<StoreName, AIStore>
    // Where AIStore has name (serialized as string), query_model, index_model, store_original
    let old_format = populated_ai_old_flat_snapshot_json();

    let json_bytes = serde_json::to_vec(&old_format).expect("Failed to serialize old format");

    let migrated: AIStores = AIStoreHandler::load_snapshot(&json_bytes).expect("Migration failed");

    {
        let guard = migrated.guard();
        let inner = migrated
            .get(&Schema::default(), &guard)
            .expect("No public schema after migration");
        assert_eq!(inner.len(), 1, "Expected 1 AI store under public schema");
    }
    assert_populated_ai_snapshot(migrated);
}

#[test]
fn test_ai_migrate_old_flat_snapshot_via_file() {
    // Write fixture file
    let old_format = populated_ai_old_flat_snapshot_json();
    let fixture_path = std::env::temp_dir().join(format!(
        "ai_old_flat_snapshot_{}_{}.json",
        std::process::id(),
        std::thread::current().name().unwrap_or("migration")
    ));
    let json_bytes = serde_json::to_vec_pretty(&old_format).expect("Failed to serialize");
    std::fs::write(&fixture_path, &json_bytes).expect("Failed to write fixture");

    // Read back and migrate
    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");
    let migrated: AIStores =
        AIStoreHandler::load_snapshot(&read_bytes).expect("Migration of fixture failed");

    {
        let guard = migrated.guard();
        let inner = migrated
            .get(&Schema::default(), &guard)
            .expect("No public schema after migration");
        assert_eq!(inner.len(), 1, "Expected 1 AI store under public schema");
    }
    assert_populated_ai_snapshot(migrated);
    let _ = std::fs::remove_file(&fixture_path);
}

#[test]
fn test_ai_migrate_from_committed_fixture() {
    let fixture_path = ai_fixture_dir().join("ai_old_flat_snapshot.json");

    assert!(
        fixture_path.exists(),
        "Committed fixture not found: {:?}",
        fixture_path
    );

    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

    let migrated: AIStores =
        AIStoreHandler::load_snapshot(&read_bytes).expect("Migration of fixture failed");

    {
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
    assert_populated_ai_snapshot(migrated);
}

#[test]
#[ignore]
fn generate_ai_fixtures() {
    let fixture_dir = ai_fixture_dir();
    let old_flat_path = fixture_dir.join("ai_old_flat_snapshot.json");
    let old_flat_json =
        serde_json::to_string_pretty(&populated_ai_old_flat_snapshot_json()).unwrap();
    std::fs::write(&old_flat_path, &old_flat_json).expect("Failed to write old flat fixture");
    eprintln!("Generated old flat fixture at: {:?}", old_flat_path);

    let v2_path = fixture_dir.join("ai_v2_snapshot.json");
    let v2_json = serde_json::to_string_pretty(&populated_ai_v2_snapshot_json()).unwrap();
    std::fs::write(&v2_path, &v2_json).expect("Failed to write V2 fixture");
    eprintln!("Generated V2 fixture at: {:?}", v2_path);
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
    {
        let guard = loaded.guard();
        let inner = loaded
            .get(&Schema::default(), &guard)
            .expect("No public schema after V2 load");
        assert_eq!(inner.len(), 1, "Expected 1 AI store under public schema");
        let pinned = inner.pin();
        let (key, _) = pinned.iter().next().expect("No store in result");
        assert_eq!(key.value, "test_ai_store", "Store name preserved in V2");
    }
    assert_populated_ai_snapshot(loaded);
}
