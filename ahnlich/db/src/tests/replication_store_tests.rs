use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ahnlich_replication::node::ReplicationNode;
use ahnlich_replication::storage::{
    MemorySnapshotStore, ReplicationFailureState, RocksLogStore, StateMachineStore,
};
use ahnlich_replication::types::DbCommand;
use ahnlich_types::db::query;
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreName, StoreValue};
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::predicates::{
    self, Predicate, PredicateCondition, predicate::Kind as PredicateKind,
    predicate_condition::Kind as PredicateConditionKind,
};
use ahnlich_types::schema::Schema;
use futures::executor::block_on;
use openraft::RaftSnapshotBuilder;
use openraft::storage::RaftStateMachine;
use openraft::{CommittedLeaderId, Entry, EntryPayload, LogId, Membership, StoredMembership};
use prost::Message;

use crate::replication::{DbStateMachine, DbTypeConfig};

fn replication_node(id: u64) -> ReplicationNode {
    ReplicationNode {
        raft_addr: format!("127.0.0.1:{}", 9_000 + id),
        service_addr: format!("127.0.0.1:{}", 8_000 + id),
    }
}

fn membership(voters: &[u64]) -> Membership<u64, ReplicationNode> {
    let voter_set = voters.iter().copied().collect::<BTreeSet<_>>();
    let nodes = voters
        .iter()
        .copied()
        .map(|id| (id, replication_node(id)))
        .collect::<BTreeMap<_, _>>();
    Membership::new(vec![voter_set], nodes)
}

fn log_id(term: u64, node_id: u64, index: u64) -> LogId<u64> {
    LogId::new(CommittedLeaderId::new(term, node_id), index)
}

fn normal_entry(term: u64, node_id: u64, index: u64, data: DbCommand) -> Entry<DbTypeConfig> {
    Entry {
        log_id: log_id(term, node_id, index),
        payload: EntryPayload::Normal(data),
    }
}

fn store_value(label: &str) -> StoreValue {
    let mut value = HashMap::new();
    value.insert(
        "label".to_owned(),
        MetadataValue {
            value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                label.to_owned(),
            )),
        },
    );
    StoreValue { value }
}

fn create_store_query(store: &str, dimension: u32) -> query::CreateStore {
    query::CreateStore {
        store: store.to_owned(),
        dimension,
        create_predicates: vec!["label".to_owned()],
        non_linear_indices: Vec::new(),
        error_if_exists: true,
        schema: None,
    }
}

fn set_query(store: &str, key: Vec<f32>, label: &str) -> query::Set {
    query::Set {
        store: store.to_owned(),
        inputs: vec![DbStoreEntry {
            key: Some(StoreKey { key }),
            value: Some(store_value(label)),
        }],
        schema: None,
    }
}

fn label_condition(label: &str) -> PredicateCondition {
    PredicateCondition {
        kind: Some(PredicateConditionKind::Value(Predicate {
            kind: Some(PredicateKind::Equals(predicates::Equals {
                key: "label".to_owned(),
                value: Some(MetadataValue {
                    value: Some(ahnlich_types::metadata::metadata_value::Value::RawString(
                        label.to_owned(),
                    )),
                }),
            })),
        })),
    }
}

fn drop_store_query(store: &str) -> query::DropStore {
    query::DropStore {
        store: store.to_owned(),
        error_if_not_exists: true,
        schema: None,
    }
}

fn clear_store_query(store: &str) -> query::ClearStore {
    query::ClearStore {
        store: store.to_owned(),
        schema: None,
    }
}

fn encode_command<M: Message>(query: &M, wrap: impl FnOnce(Vec<u8>) -> DbCommand) -> DbCommand {
    wrap(query.encode_to_vec())
}

fn decode_count_response(response: ahnlich_replication::types::DbResponse) -> usize {
    match response {
        ahnlich_replication::types::DbResponse::Bytes(bytes) => {
            bitcode::deserialize(bytes.as_slice()).expect("decode count response")
        }
        ahnlich_replication::types::DbResponse::Unit => panic!("expected byte response"),
    }
}

#[test]
fn snapshot_round_trip_restores_store_state() {
    let failure_state = Arc::new(ReplicationFailureState::default());
    let mut store = StateMachineStore::<DbTypeConfig, _>::new(
        DbStateMachine::new(Arc::new(AtomicBool::new(false)), 16, None, 10_000),
        StoredMembership::new(None, membership(&[1])),
        failure_state,
        Arc::new(MemorySnapshotStore::default()),
    )
    .expect("create state machine store");

    block_on(store.apply(vec![
        normal_entry(
            1,
            1,
            1,
            encode_command(&create_store_query("products", 2), DbCommand::CreateStore),
        ),
        normal_entry(
            1,
            1,
            2,
            encode_command(
                &set_query("products", vec![1.0, 2.0], "alpha"),
                DbCommand::Set,
            ),
        ),
    ]))
    .expect("apply should succeed");

    let mut snapshot_builder = block_on(store.get_snapshot_builder());
    let snapshot = block_on(snapshot_builder.build_snapshot()).expect("build snapshot");

    let mut target = StateMachineStore::<DbTypeConfig, _>::new(
        DbStateMachine::new(Arc::new(AtomicBool::new(false)), 16, None, 10_000),
        StoredMembership::new(None, membership(&[1])),
        Arc::new(ReplicationFailureState::default()),
        Arc::new(MemorySnapshotStore::default()),
    )
    .expect("create target state machine store");

    let openraft::Snapshot { meta, snapshot } = snapshot;
    block_on(target.install_snapshot(&meta, snapshot)).expect("install snapshot");
    let restored_entries = target
        .with_handler(|handler| {
            handler.store_handler().get_key_in_store(
                &StoreName {
                    value: "products".to_owned(),
                },
                &Schema::default(),
                vec![StoreKey {
                    key: vec![1.0, 2.0],
                }],
            )
        })
        .expect("state machine access should succeed")
        .expect("restored lookup should succeed");
    assert_eq!(restored_entries.len(), 1);

    let restored_matches = target
        .with_handler(|handler| {
            handler.store_handler().get_pred_in_store(
                &StoreName {
                    value: "products".to_owned(),
                },
                &Schema::default(),
                &label_condition("alpha"),
            )
        })
        .expect("state machine access should succeed")
        .expect("restored predicate lookup should succeed");
    assert_eq!(restored_matches.len(), 1);

    block_on(target.apply(vec![normal_entry(
        1,
        1,
        3,
        encode_command(
            &set_query("products", vec![3.0, 4.0], "beta"),
            DbCommand::Set,
        ),
    )]))
    .expect("restored state should accept indexed writes");
    let new_matches = target
        .with_handler(|handler| {
            handler.store_handler().get_pred_in_store(
                &StoreName {
                    value: "products".to_owned(),
                },
                &Schema::default(),
                &label_condition("beta"),
            )
        })
        .expect("state machine access should succeed")
        .expect("new predicate lookup should succeed");
    assert_eq!(new_matches.len(), 1);

    let response = block_on(target.apply(vec![normal_entry(
        1,
        1,
        4,
        encode_command(&drop_store_query("products"), DbCommand::DropStore),
    )]))
    .expect("restored state should accept drop");
    let deleted_count = decode_count_response(response.into_iter().next().expect("one response"));
    assert_eq!(deleted_count, 1);
}

#[test]
fn durable_snapshot_reopens_with_queryable_and_mutable_predicate_indexes() {
    let dir = tempfile::tempdir().expect("create snapshot directory");
    let snapshot_store =
        Arc::new(RocksLogStore::<DbTypeConfig>::open(dir.path()).expect("open snapshot storage"));
    let mut source = StateMachineStore::<DbTypeConfig, _>::new(
        DbStateMachine::new(Arc::new(AtomicBool::new(false)), 16, None, 10_000),
        StoredMembership::new(None, membership(&[1])),
        Arc::new(ReplicationFailureState::default()),
        snapshot_store.clone(),
    )
    .expect("create state machine store");

    block_on(source.apply(vec![
        normal_entry(
            1,
            1,
            1,
            encode_command(&create_store_query("products", 2), DbCommand::CreateStore),
        ),
        normal_entry(
            1,
            1,
            2,
            encode_command(
                &set_query("products", vec![1.0, 2.0], "alpha"),
                DbCommand::Set,
            ),
        ),
    ]))
    .expect("apply should succeed");
    let mut snapshot_builder = block_on(source.get_snapshot_builder());
    block_on(snapshot_builder.build_snapshot()).expect("persist snapshot");
    drop(snapshot_builder);
    drop(source);
    drop(snapshot_store);

    let reopened_store =
        Arc::new(RocksLogStore::<DbTypeConfig>::open(dir.path()).expect("reopen snapshot storage"));
    let mut restored = StateMachineStore::<DbTypeConfig, _>::new(
        DbStateMachine::new(Arc::new(AtomicBool::new(false)), 16, None, 10_000),
        StoredMembership::new(None, membership(&[9])),
        Arc::new(ReplicationFailureState::default()),
        reopened_store,
    )
    .expect("restore state machine from durable snapshot");

    let restored_matches = restored
        .with_handler(|handler| {
            handler.store_handler().get_pred_in_store(
                &StoreName {
                    value: "products".to_owned(),
                },
                &Schema::default(),
                &label_condition("alpha"),
            )
        })
        .expect("state machine access should succeed")
        .expect("persisted predicate lookup should succeed");
    assert_eq!(restored_matches.len(), 1);

    block_on(restored.apply(vec![normal_entry(
        1,
        1,
        3,
        encode_command(
            &set_query("products", vec![3.0, 4.0], "beta"),
            DbCommand::Set,
        ),
    )]))
    .expect("reopened state should accept indexed writes");
    let new_matches = restored
        .with_handler(|handler| {
            handler.store_handler().get_pred_in_store(
                &StoreName {
                    value: "products".to_owned(),
                },
                &Schema::default(),
                &label_condition("beta"),
            )
        })
        .expect("state machine access should succeed")
        .expect("new predicate lookup should succeed");
    assert_eq!(new_matches.len(), 1);
}

#[test]
fn apply_operation_failure_marks_replication_failure_state() {
    let failure_state = Arc::new(ReplicationFailureState::default());
    let mut store = StateMachineStore::<DbTypeConfig, _>::new(
        DbStateMachine::new(Arc::new(AtomicBool::new(false)), 16, None, 10_000),
        StoredMembership::new(None, membership(&[1])),
        failure_state.clone(),
        Arc::new(MemorySnapshotStore::default()),
    )
    .expect("create state machine store");

    let err = block_on(store.apply(vec![normal_entry(
        1,
        1,
        1,
        encode_command(
            &set_query("missing-store", vec![1.0, 2.0], "alpha"),
            DbCommand::Set,
        ),
    )]))
    .expect_err("apply should fail");

    assert!(matches!(err, openraft::StorageError::IO { .. }));
    assert!(failure_state.failed());

    let reason = failure_state.reason().expect("failure reason should exist");
    assert!(reason.contains("state machine apply failed"));
    assert!(reason.contains("Set apply failed"));
}

#[test]
fn clear_store_command_removes_entries_without_dropping_store() {
    let mut store = StateMachineStore::<DbTypeConfig, _>::new(
        DbStateMachine::new(Arc::new(AtomicBool::new(false)), 16, None, 10_000),
        StoredMembership::new(None, membership(&[1])),
        Arc::new(ReplicationFailureState::default()),
        Arc::new(MemorySnapshotStore::default()),
    )
    .expect("create state machine store");

    let responses = block_on(store.apply(vec![
        normal_entry(
            1,
            1,
            1,
            encode_command(&create_store_query("products", 2), DbCommand::CreateStore),
        ),
        normal_entry(
            1,
            1,
            2,
            encode_command(&set_query("products"), DbCommand::Set),
        ),
        normal_entry(
            1,
            1,
            3,
            encode_command(&clear_store_query("products"), DbCommand::ClearStore),
        ),
    ]))
    .expect("replicated operations should succeed");

    let deleted_count = decode_count_response(
        responses
            .into_iter()
            .last()
            .expect("clear operation should return a response"),
    );

    assert_eq!(deleted_count, 1);

    let store_info = store
        .with_handler(|handler| {
            handler.store_handler().get_store(
                &StoreName {
                    value: "products".to_owned(),
                },
                &Schema::default(),
            )
        })
        .expect("state machine access should succeed")
        .expect("cleared store should still exist");

    assert_eq!(store_info.len, 0);
    assert_eq!(store_info.dimension, 2);
}
