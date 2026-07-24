use crate::engine::store::StoreHandler;
use ahnlich_types::{
    keyval::{StoreKey, StoreName, StoreValue},
    metadata::{MetadataValue, metadata_value::Value as MValue},
    predicates::{
        Equals, Predicate, PredicateCondition, predicate::Kind as PredKind,
        predicate_condition::Kind as PCKind,
    },
    schema::Schema,
};
use std::collections::{HashMap, HashSet as StdHashSet};
use std::num::NonZeroUsize;
use std::sync::{Arc, atomic::AtomicBool};

#[test]
fn test_upsert_key_only_update() {
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let store_name = StoreName {
        value: "test_store".to_string(),
    };
    let schema = Schema::default();

    handler
        .create_store(
            store_name.clone(),
            &schema,
            NonZeroUsize::new(3).unwrap(),
            vec!["id".to_string()],
            StdHashSet::new(),
            false,
        )
        .unwrap();

    let old_key = StoreKey {
        key: vec![1.0, 2.0, 3.0],
    };
    let value = StoreValue {
        value: HashMap::from_iter([(
            "id".to_string(),
            MetadataValue {
                value: Some(MValue::RawString("123".to_string())),
            },
        )]),
    };

    handler
        .set_in_store(&store_name, &schema, vec![(old_key.clone(), value.clone())])
        .unwrap();

    let condition = PredicateCondition {
        kind: Some(PCKind::Value(Predicate {
            kind: Some(PredKind::Equals(Equals {
                key: "id".to_string(),
                value: Some(MetadataValue {
                    value: Some(MValue::RawString("123".to_string())),
                }),
            })),
        })),
    };

    let new_key = StoreKey {
        key: vec![4.0, 5.0, 6.0],
    };

    let result = handler.upsert(
        &store_name,
        &schema,
        Some(new_key.clone()),
        None,
        &condition,
        false,
    );

    assert!(result.is_ok());
    let upsert_result = result.unwrap();
    assert_eq!(upsert_result.updated, 1, "Should report 1 update");
    assert_eq!(upsert_result.inserted, 0, "Should report 0 insertions");

    let entries = handler
        .get_pred_in_store(&store_name, &schema, &condition)
        .unwrap();
    assert_eq!(entries.len(), 1);

    let old_entries = handler
        .get_key_in_store(&store_name, &schema, vec![old_key])
        .unwrap();
    assert_eq!(old_entries.len(), 0);

    let all_entries = handler
        .get_key_in_store(&store_name, &schema, vec![new_key.clone()])
        .unwrap();
    assert_eq!(all_entries.len(), 1);
    assert_eq!(
        all_entries[0].1.value.get("id").unwrap().value,
        Some(MValue::RawString("123".to_string()))
    );
}

#[test]
fn test_upsert_value_only_update() {
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let store_name = StoreName {
        value: "test_store".to_string(),
    };
    let schema = Schema::default();

    handler
        .create_store(
            store_name.clone(),
            &schema,
            NonZeroUsize::new(3).unwrap(),
            vec!["id".to_string(), "name".to_string()],
            StdHashSet::new(),
            false,
        )
        .unwrap();

    let key = StoreKey {
        key: vec![1.0, 2.0, 3.0],
    };
    let old_value = StoreValue {
        value: HashMap::from_iter([
            (
                "id".to_string(),
                MetadataValue {
                    value: Some(MValue::RawString("123".to_string())),
                },
            ),
            (
                "name".to_string(),
                MetadataValue {
                    value: Some(MValue::RawString("Alice".to_string())),
                },
            ),
        ]),
    };

    handler
        .set_in_store(&store_name, &schema, vec![(key.clone(), old_value.clone())])
        .unwrap();

    let condition = PredicateCondition {
        kind: Some(PCKind::Value(Predicate {
            kind: Some(PredKind::Equals(Equals {
                key: "id".to_string(),
                value: Some(MetadataValue {
                    value: Some(MValue::RawString("123".to_string())),
                }),
            })),
        })),
    };

    let new_value = StoreValue {
        value: HashMap::from_iter([
            (
                "id".to_string(),
                MetadataValue {
                    value: Some(MValue::RawString("456".to_string())),
                },
            ),
            (
                "name".to_string(),
                MetadataValue {
                    value: Some(MValue::RawString("Bob".to_string())),
                },
            ),
        ]),
    };

    let result = handler.upsert(
        &store_name,
        &schema,
        None,
        Some(new_value.clone()),
        &condition,
        false,
    );

    assert!(result.is_ok());
    let upsert_result = result.unwrap();
    assert_eq!(upsert_result.updated, 1, "Should report 1 update");
    assert_eq!(upsert_result.inserted, 0, "Should report 0 insertions");

    let entries = handler
        .get_key_in_store(&store_name, &schema, vec![key.clone()])
        .unwrap();
    assert_eq!(entries.len(), 1);

    assert_eq!(
        entries[0].1.value.get("id").unwrap().value,
        Some(MValue::RawString("456".to_string()))
    );
    assert_eq!(
        entries[0].1.value.get("name").unwrap().value,
        Some(MValue::RawString("Bob".to_string()))
    );

    let old_entries = handler
        .get_pred_in_store(&store_name, &schema, &condition)
        .unwrap();
    assert_eq!(old_entries.len(), 0);
}

#[test]
fn test_upsert_merge_metadata() {
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let store_name = StoreName {
        value: "test_store".to_string(),
    };
    let schema = Schema::default();

    handler
        .create_store(
            store_name.clone(),
            &schema,
            NonZeroUsize::new(3).unwrap(),
            vec!["id".to_string(), "name".to_string(), "age".to_string()],
            StdHashSet::new(),
            false,
        )
        .unwrap();

    let key = StoreKey {
        key: vec![1.0, 2.0, 3.0],
    };
    let old_value = StoreValue {
        value: HashMap::from_iter([
            (
                "id".to_string(),
                MetadataValue {
                    value: Some(MValue::RawString("123".to_string())),
                },
            ),
            (
                "name".to_string(),
                MetadataValue {
                    value: Some(MValue::RawString("Alice".to_string())),
                },
            ),
            (
                "age".to_string(),
                MetadataValue {
                    value: Some(MValue::RawString("30".to_string())),
                },
            ),
        ]),
    };

    handler
        .set_in_store(&store_name, &schema, vec![(key.clone(), old_value.clone())])
        .unwrap();

    let condition = PredicateCondition {
        kind: Some(PCKind::Value(Predicate {
            kind: Some(PredKind::Equals(Equals {
                key: "id".to_string(),
                value: Some(MetadataValue {
                    value: Some(MValue::RawString("123".to_string())),
                }),
            })),
        })),
    };

    let new_value = StoreValue {
        value: HashMap::from_iter([(
            "name".to_string(),
            MetadataValue {
                value: Some(MValue::RawString("Bob".to_string())),
            },
        )]),
    };

    let result = handler.upsert(
        &store_name,
        &schema,
        None,
        Some(new_value.clone()),
        &condition,
        true,
    );

    assert!(result.is_ok());
    let upsert_result = result.unwrap();
    assert_eq!(upsert_result.updated, 1, "Should report 1 update");
    assert_eq!(upsert_result.inserted, 0, "Should report 0 insertions");

    let entries = handler
        .get_key_in_store(&store_name, &schema, vec![key.clone()])
        .unwrap();
    assert_eq!(entries.len(), 1);

    assert_eq!(
        entries[0].1.value.get("id").unwrap().value,
        Some(MValue::RawString("123".to_string()))
    );
    assert_eq!(
        entries[0].1.value.get("age").unwrap().value,
        Some(MValue::RawString("30".to_string()))
    );

    assert_eq!(
        entries[0].1.value.get("name").unwrap().value,
        Some(MValue::RawString("Bob".to_string()))
    );
}

#[test]
fn test_upsert_both_key_and_value() {
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let store_name = StoreName {
        value: "test_store".to_string(),
    };
    let schema = Schema::default();

    handler
        .create_store(
            store_name.clone(),
            &schema,
            NonZeroUsize::new(3).unwrap(),
            vec!["id".to_string()],
            StdHashSet::new(),
            false,
        )
        .unwrap();

    let old_key = StoreKey {
        key: vec![1.0, 2.0, 3.0],
    };
    let old_value = StoreValue {
        value: HashMap::from_iter([(
            "id".to_string(),
            MetadataValue {
                value: Some(MValue::RawString("123".to_string())),
            },
        )]),
    };

    handler
        .set_in_store(
            &store_name,
            &schema,
            vec![(old_key.clone(), old_value.clone())],
        )
        .unwrap();

    let condition = PredicateCondition {
        kind: Some(PCKind::Value(Predicate {
            kind: Some(PredKind::Equals(Equals {
                key: "id".to_string(),
                value: Some(MetadataValue {
                    value: Some(MValue::RawString("123".to_string())),
                }),
            })),
        })),
    };

    let new_key = StoreKey {
        key: vec![4.0, 5.0, 6.0],
    };
    let new_value = StoreValue {
        value: HashMap::from_iter([(
            "id".to_string(),
            MetadataValue {
                value: Some(MValue::RawString("456".to_string())),
            },
        )]),
    };

    let result = handler.upsert(
        &store_name,
        &schema,
        Some(new_key.clone()),
        Some(new_value.clone()),
        &condition,
        false,
    );

    assert!(result.is_ok());
    let upsert_result = result.unwrap();
    assert_eq!(upsert_result.updated, 1, "Should report 1 update");
    assert_eq!(upsert_result.inserted, 0, "Should report 0 insertions");

    let old_entries = handler
        .get_key_in_store(&store_name, &schema, vec![old_key])
        .unwrap();
    assert_eq!(old_entries.len(), 0);

    let entries = handler
        .get_key_in_store(&store_name, &schema, vec![new_key.clone()])
        .unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(
        entries[0].1.value.get("id").unwrap().value,
        Some(MValue::RawString("456".to_string()))
    );
}

#[test]
fn test_upsert_error_none_none() {
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let store_name = StoreName {
        value: "test_store".to_string(),
    };
    let schema = Schema::default();

    handler
        .create_store(
            store_name.clone(),
            &schema,
            NonZeroUsize::new(3).unwrap(),
            vec!["id".to_string()],
            StdHashSet::new(),
            false,
        )
        .unwrap();

    let condition = PredicateCondition {
        kind: Some(PCKind::Value(Predicate {
            kind: Some(PredKind::Equals(Equals {
                key: "id".to_string(),
                value: Some(MetadataValue {
                    value: Some(MValue::RawString("123".to_string())),
                }),
            })),
        })),
    };

    let result = handler.upsert(&store_name, &schema, None, None, &condition, false);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("UPSERT requires at least one of new_key or new_value"));
}

#[test]
fn test_upsert_error_no_match() {
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let store_name = StoreName {
        value: "test_store".to_string(),
    };
    let schema = Schema::default();

    handler
        .create_store(
            store_name.clone(),
            &schema,
            NonZeroUsize::new(3).unwrap(),
            vec!["id".to_string()],
            StdHashSet::new(),
            false,
        )
        .unwrap();

    let key = StoreKey {
        key: vec![1.0, 2.0, 3.0],
    };
    let value = StoreValue {
        value: HashMap::from_iter([(
            "id".to_string(),
            MetadataValue {
                value: Some(MValue::RawString("123".to_string())),
            },
        )]),
    };

    handler
        .set_in_store(&store_name, &schema, vec![(key.clone(), value.clone())])
        .unwrap();

    let condition = PredicateCondition {
        kind: Some(PCKind::Value(Predicate {
            kind: Some(PredKind::Equals(Equals {
                key: "id".to_string(),
                value: Some(MetadataValue {
                    value: Some(MValue::RawString("999".to_string())),
                }),
            })),
        })),
    };

    let new_key = StoreKey {
        key: vec![4.0, 5.0, 6.0],
    };

    let result = handler.upsert(&store_name, &schema, Some(new_key), None, &condition, false);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Predicate condition found 0 entries"));
}

#[test]
fn test_upsert_error_multiple_matches() {
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let store_name = StoreName {
        value: "test_store".to_string(),
    };
    let schema = Schema::default();

    handler
        .create_store(
            store_name.clone(),
            &schema,
            NonZeroUsize::new(3).unwrap(),
            vec!["category".to_string()],
            StdHashSet::new(),
            false,
        )
        .unwrap();

    let entries = vec![
        (
            StoreKey {
                key: vec![1.0, 2.0, 3.0],
            },
            StoreValue {
                value: HashMap::from_iter([(
                    "category".to_string(),
                    MetadataValue {
                        value: Some(MValue::RawString("A".to_string())),
                    },
                )]),
            },
        ),
        (
            StoreKey {
                key: vec![4.0, 5.0, 6.0],
            },
            StoreValue {
                value: HashMap::from_iter([(
                    "category".to_string(),
                    MetadataValue {
                        value: Some(MValue::RawString("A".to_string())),
                    },
                )]),
            },
        ),
    ];

    handler.set_in_store(&store_name, &schema, entries).unwrap();

    let condition = PredicateCondition {
        kind: Some(PCKind::Value(Predicate {
            kind: Some(PredKind::Equals(Equals {
                key: "category".to_string(),
                value: Some(MetadataValue {
                    value: Some(MValue::RawString("A".to_string())),
                }),
            })),
        })),
    };

    let new_key = StoreKey {
        key: vec![7.0, 8.0, 9.0],
    };

    let result = handler.upsert(&store_name, &schema, Some(new_key), None, &condition, false);

    assert!(result.is_err());
    let err_msg = format!("{}", result.unwrap_err());
    assert!(err_msg.contains("Predicate condition found 2 entries"));
}
