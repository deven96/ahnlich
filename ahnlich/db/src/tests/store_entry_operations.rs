use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ahnlich_types::algorithm::nonlinear::{KdTreeConfig, NonLinearIndex, non_linear_index};
use ahnlich_types::db::query;
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
use ahnlich_types::metadata::{MetadataValue, metadata_value::Value as MetadataValueKind};
use ahnlich_types::predicates::{
    Equals, Predicate, PredicateCondition, predicate::Kind as PredicateKind,
    predicate_condition::Kind as PredicateConditionKind,
};
use ahnlich_types::utils::StoreKeyId;

use crate::engine::operations;
use crate::engine::store::{ParallelismConfig, StoreHandler};
use crate::errors::ServerError;

const STORE_NAME: &str = "documents";

fn store_handler() -> StoreHandler {
    StoreHandler::new(
        Arc::new(AtomicBool::new(false)),
        ParallelismConfig::from_cli(2, None, 10_000),
    )
}

fn raw_string(value: &str) -> MetadataValue {
    MetadataValue {
        value: Some(MetadataValueKind::RawString(value.to_owned())),
    }
}

fn store_entry(embedding: Vec<f32>, label: &str, category: &str) -> DbStoreEntry {
    DbStoreEntry {
        key: Some(StoreKey { key: embedding }),
        value: Some(StoreValue {
            value: HashMap::from([
                ("label".to_owned(), raw_string(label)),
                ("category".to_owned(), raw_string(category)),
            ]),
        }),
    }
}

fn seeded_entries() -> Vec<DbStoreEntry> {
    vec![
        store_entry(vec![1.0, 0.0, 0.0], "one", "keep"),
        store_entry(vec![2.0, 0.0, 0.0], "two", "discard"),
        store_entry(vec![3.0, 0.0, 0.0], "three", "keep"),
        store_entry(vec![4.0, 0.0, 0.0], "four", "discard"),
        store_entry(vec![5.0, 0.0, 0.0], "five", "keep"),
    ]
}

fn create_store(handler: &StoreHandler) {
    operations::create_store(
        handler,
        query::CreateStore {
            store: STORE_NAME.to_owned(),
            dimension: 3,
            create_predicates: vec!["category".to_owned()],
            non_linear_indices: vec![NonLinearIndex {
                index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
            }],
            error_if_exists: true,
            schema: None,
        },
    )
    .expect("store creation should succeed");
}

fn set_entries(handler: &StoreHandler, entries: Vec<DbStoreEntry>) {
    operations::set(
        handler,
        query::Set {
            store: STORE_NAME.to_owned(),
            inputs: entries,
            schema: None,
        },
        handler.parallelism_config(),
        handler.active_requests_count(),
    )
    .expect("setting entries should succeed");
}

fn seeded_store() -> StoreHandler {
    let handler = store_handler();
    create_store(&handler);
    set_entries(&handler, seeded_entries());
    handler
}

fn category_condition(category: &str) -> PredicateCondition {
    PredicateCondition {
        kind: Some(PredicateConditionKind::Value(Predicate {
            kind: Some(PredicateKind::Equals(Equals {
                key: "category".to_owned(),
                value: Some(raw_string(category)),
            })),
        })),
    }
}

fn entry_embedding(entry: &DbStoreEntry) -> Vec<f32> {
    entry
        .key
        .as_ref()
        .expect("listed entry should have a key")
        .key
        .clone()
}

#[test]
fn list_store_entries_returns_complete_ordered_pages() {
    let handler = seeded_store();
    let mut cursor = None;
    let mut listed_embeddings = Vec::new();

    loop {
        let page = operations::list_store_entries(
            &handler,
            query::ListStoreEntries {
                store: STORE_NAME.to_owned(),
                cursor,
                limit: Some(2),
                condition: None,
                schema: None,
            },
        )
        .expect("listing entries should succeed");

        assert!(page.entries.len() <= 2);

        listed_embeddings.extend(page.entries.iter().map(entry_embedding));

        match page.next_cursor {
            Some(next_cursor) => {
                assert_eq!(next_cursor.len(), 16);
                cursor = Some(next_cursor);
            }
            None => break,
        }
    }

    let mut expected_embeddings = seeded_entries()
        .iter()
        .map(entry_embedding)
        .collect::<Vec<_>>();

    expected_embeddings.sort_unstable_by_key(|embedding| {
        StoreKeyId::from(&StoreKey {
            key: embedding.clone(),
        })
    });

    assert_eq!(listed_embeddings, expected_embeddings);
}

#[test]
fn list_store_entries_applies_predicate_filter() {
    let handler = seeded_store();

    let page = operations::list_store_entries(
        &handler,
        query::ListStoreEntries {
            store: STORE_NAME.to_owned(),
            cursor: None,
            limit: Some(100),
            condition: Some(category_condition("keep")),
            schema: None,
        },
    )
    .expect("filtered listing should succeed");

    assert_eq!(page.entries.len(), 3);
    assert!(page.next_cursor.is_none());

    let expected_category = raw_string("keep");

    assert!(page.entries.iter().all(|entry| {
        entry
            .value
            .as_ref()
            .and_then(|value| value.value.get("category"))
            == Some(&expected_category)
    }));
}

#[test]
fn list_store_entries_rejects_invalid_pagination() {
    let handler = seeded_store();

    for limit in [0, 1001] {
        let error = operations::list_store_entries(
            &handler,
            query::ListStoreEntries {
                store: STORE_NAME.to_owned(),
                cursor: None,
                limit: Some(limit),
                condition: None,
                schema: None,
            },
        )
        .expect_err("invalid limit should fail");

        assert!(matches!(
            error,
            ServerError::InvalidArgument(message)
                if message.contains("limit")
        ));
    }

    let error = operations::list_store_entries(
        &handler,
        query::ListStoreEntries {
            store: STORE_NAME.to_owned(),
            cursor: Some("not-a-valid-cursor".to_owned()),
            limit: Some(10),
            condition: None,
            schema: None,
        },
    )
    .expect_err("invalid cursor should fail");

    assert!(matches!(
        error,
        ServerError::InvalidArgument(message)
            if message.contains("cursor")
    ));
}

#[test]
fn store_entry_operations_return_not_found_for_missing_store() {
    let handler = store_handler();

    let list_error = operations::list_store_entries(
        &handler,
        query::ListStoreEntries {
            store: "missing".to_owned(),
            cursor: None,
            limit: Some(10),
            condition: None,
            schema: None,
        },
    )
    .expect_err("listing a missing store should fail");

    assert!(matches!(list_error, ServerError::StoreNotFound(_)));

    let clear_error = operations::clear_store(
        &handler,
        query::ClearStore {
            store: "missing".to_owned(),
            schema: None,
        },
    )
    .expect_err("clearing a missing store should fail");

    assert!(matches!(clear_error, ServerError::StoreNotFound(_)));
}

#[test]
fn clear_store_preserves_configuration_and_remains_usable() {
    let handler = seeded_store();
    let before = operations::get_store(
        &handler,
        query::GetStore {
            store: STORE_NAME.to_owned(),
            schema: None,
        },
    )
    .expect("store information should be available");

    handler.write_flag().store(false, Ordering::SeqCst);

    let deleted = operations::clear_store(
        &handler,
        query::ClearStore {
            store: STORE_NAME.to_owned(),
            schema: None,
        },
    )
    .expect("clearing the store should succeed");

    assert_eq!(deleted, 5);
    assert!(handler.write_flag().load(Ordering::SeqCst));

    let after = operations::get_store(
        &handler,
        query::GetStore {
            store: STORE_NAME.to_owned(),
            schema: None,
        },
    )
    .expect("cleared store should still exist");

    assert_eq!(after.len, 0);
    assert_eq!(after.dimension, before.dimension);
    assert_eq!(after.predicate_indices, before.predicate_indices,);
    assert_eq!(after.non_linear_indices, before.non_linear_indices,);

    handler.write_flag().store(false, Ordering::SeqCst);

    let deleted = operations::clear_store(
        &handler,
        query::ClearStore {
            store: STORE_NAME.to_owned(),
            schema: None,
        },
    )
    .expect("clearing an empty store should succeed");

    assert_eq!(deleted, 0);
    assert!(!handler.write_flag().load(Ordering::SeqCst));

    set_entries(
        &handler,
        vec![store_entry(vec![9.0, 0.0, 0.0], "new", "keep")],
    );

    let page = operations::list_store_entries(
        &handler,
        query::ListStoreEntries {
            store: STORE_NAME.to_owned(),
            cursor: None,
            limit: Some(10),
            condition: None,
            schema: None,
        },
    )
    .expect("cleared store should remain usable");

    assert_eq!(page.entries.len(), 1);
}
