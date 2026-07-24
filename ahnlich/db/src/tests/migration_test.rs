use crate::engine::store::StoreHandler;
use crate::engine::store::Stores;
use ahnlich_types::algorithm::{
    algorithms::DistanceMetric,
    nonlinear::{HnswConfig, KdTreeConfig, non_linear_index},
};
use ahnlich_types::keyval::{StoreKey, StoreName, StoreValue};
use ahnlich_types::metadata::{MetadataValue, metadata_value};
use ahnlich_types::schema::Schema;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use utils::persistence::AhnlichPersistenceUtils;

fn db_fixture_dir() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures")
}

fn populated_db_fixture_handler() -> StoreHandler {
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let predicates = vec!["category".to_string(), "color".to_string()];
    let mut non_linear_indices: HashSet<non_linear_index::Index> = HashSet::new();
    non_linear_indices.insert(non_linear_index::Index::Kdtree(KdTreeConfig {}));
    non_linear_indices.insert(non_linear_index::Index::Hnsw(HnswConfig {
        distance: Some(DistanceMetric::Cosine as i32),
        ef_construction: Some(150),
        maximum_connections: Some(24),
        maximum_connections_zero: Some(48),
        extend_candidates: Some(true),
        keep_pruned_connections: Some(true),
    }));

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

    for (key, name, category, color) in [
        (vec![0.5f32, 0.1, 0.8], "item1", "fruit", "red"),
        (vec![0.2f32, 0.7, 0.3], "item2", "vegetable", "green"),
        (vec![0.9f32, 0.4, 0.2], "item3", "grain", "yellow"),
    ] {
        let mut meta = HashMap::new();
        meta.insert(
            "name".to_string(),
            MetadataValue {
                value: Some(metadata_value::Value::RawString(name.to_string())),
            },
        );
        meta.insert(
            "category".to_string(),
            MetadataValue {
                value: Some(metadata_value::Value::RawString(category.to_string())),
            },
        );
        meta.insert(
            "color".to_string(),
            MetadataValue {
                value: Some(metadata_value::Value::RawString(color.to_string())),
            },
        );

        handler
            .set_in_store(
                &StoreName {
                    value: "fixture_store".to_string(),
                },
                &Schema::default(),
                vec![(StoreKey { key }, StoreValue { value: meta })],
            )
            .unwrap();
    }

    handler
}

fn populated_db_v2_snapshot_json() -> Value {
    serde_json::to_value(populated_db_fixture_handler().get_snapshot())
        .expect("DB V2 snapshot serialization failed")
}

fn populated_db_old_flat_snapshot_json() -> Value {
    populated_db_v2_snapshot_json()
        .get("stores")
        .and_then(Value::as_object)
        .and_then(|schemas| schemas.get("public"))
        .cloned()
        .expect("generated DB snapshot should contain public schema")
}

fn assert_populated_db_snapshot(migrated: Stores) {
    let mut handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    handler.use_snapshot(migrated);
    let store_info = handler
        .get_store(
            &StoreName {
                value: "fixture_store".to_string(),
            },
            &Schema::default(),
        )
        .expect("fixture store should load after migration");

    assert_eq!(store_info.len, 3, "fixture should preserve inserted values");
    assert_eq!(
        store_info.dimension, 3,
        "fixture should preserve store dimension"
    );
    assert!(
        store_info
            .predicate_indices
            .iter()
            .any(|predicate| predicate == "category"),
        "fixture should preserve category predicate index"
    );
    assert!(
        store_info
            .predicate_indices
            .iter()
            .any(|predicate| predicate == "color"),
        "fixture should preserve color predicate index"
    );
    assert!(
        store_info
            .non_linear_indices
            .iter()
            .any(|index| matches!(index.index, Some(non_linear_index::Index::Kdtree(_)))),
        "fixture should preserve KDTree index"
    );
    assert!(
        store_info
            .non_linear_indices
            .iter()
            .any(|index| matches!(index.index, Some(non_linear_index::Index::Hnsw(_)))),
        "fixture should preserve HNSW index"
    );
}

#[test]
fn test_db_migrate_old_flat_snapshot_via_json() {
    let old_format = populated_db_old_flat_snapshot_json();
    let json_bytes = serde_json::to_vec(&old_format).expect("Failed to serialize old format");
    let migrated: Stores = StoreHandler::load_snapshot(&json_bytes).expect("Migration failed");
    {
        let guard = migrated.guard();
        let inner = migrated
            .get(&Schema::default(), &guard)
            .expect("No public schema after migration");
        assert_eq!(inner.len(), 1, "Expected 1 DB store under public schema");
    }
    assert_populated_db_snapshot(migrated);
}

#[test]
fn test_db_migrate_from_committed_fixture() {
    let fixture_path = db_fixture_dir().join("db_old_flat_snapshot.json");

    assert!(
        fixture_path.exists(),
        "Committed fixture not found: {:?}",
        fixture_path
    );

    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");
    let migrated: Stores =
        StoreHandler::load_snapshot(&read_bytes).expect("Migration of fixture failed");
    {
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
    assert_populated_db_snapshot(migrated);
}

#[test]
#[ignore]
fn generate_db_v2_fixture() {
    let fixture_dir = db_fixture_dir();
    let old_flat_path = fixture_dir.join("db_old_flat_snapshot.json");
    let old_flat_json =
        serde_json::to_string_pretty(&populated_db_old_flat_snapshot_json()).unwrap();
    std::fs::write(&old_flat_path, &old_flat_json).expect("Failed to write old flat fixture");
    eprintln!("Generated old flat fixture at: {:?}", old_flat_path);

    let v2_path = fixture_dir.join("db_v2_snapshot.json");
    let v2_json = serde_json::to_string_pretty(&populated_db_v2_snapshot_json()).unwrap();
    std::fs::write(&v2_path, &v2_json).expect("Failed to write V2 fixture");
    eprintln!("Generated V2 fixture at: {:?}", v2_path);
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
    {
        let guard = loaded.guard();
        let inner = loaded
            .get(&Schema::default(), &guard)
            .expect("No public schema after V2 load");
        assert_eq!(inner.len(), 1, "Expected 1 DB store under public schema");
        let pinned = inner.pin();
        let (key, _store) = pinned.iter().next().expect("No store in result");
        assert_eq!(key.value, "fixture_store", "Store name preserved in V2");
    }
    assert_populated_db_snapshot(loaded);
}
