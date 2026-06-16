#![allow(clippy::result_large_err)]

use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use ahnlich_replication::node::ReplicationNode;
use ahnlich_replication::storage::StateMachineHandler;
use ahnlich_replication::types::{DbCommand, DbResponse};
use ahnlich_types::db::query;
use openraft::{StorageError, StorageIOError};
use prost::Message;
use utils::persistence::AhnlichPersistenceUtils;

use crate::engine::operations;
use crate::engine::store::{StoreHandler, Stores};

openraft::declare_raft_types!(
    pub DbTypeConfig:
        D = DbCommand,
        R = DbResponse,
        Node = ReplicationNode,
        SnapshotData = Cursor<Vec<u8>>
);

#[derive(Debug)]
pub struct DbStateMachine {
    store_handler: StoreHandler,
}

impl DbStateMachine {
    pub fn new(write_flag: Arc<AtomicBool>) -> Self {
        Self {
            store_handler: StoreHandler::new(write_flag),
        }
    }

    pub fn from_store_handler(store_handler: StoreHandler) -> Self {
        Self { store_handler }
    }

    pub fn store_handler(&self) -> &StoreHandler {
        &self.store_handler
    }
}

macro_rules! command_name {
    ($ty:ty) => {{
        let full = stringify!($ty);
        full.rsplit("::").next().unwrap_or(full)
    }};
}

macro_rules! decode_payload {
    ($ty:ty, $payload:expr) => {{
        let command_name = command_name!($ty);

        <$ty>::decode($payload.as_slice()).map_err(|err| StorageError::IO {
            source: StorageIOError::read_state_machine(&std::io::Error::other(format!(
                "failed to decode {command_name} raft payload: {err}",
            ))),
        })
    }};
}

macro_rules! encode_raw_result {
    ($ty:ty, $result:expr) => {{
        let command_name = command_name!($ty);

        bitcode::serialize($result)
            .map(DbResponse::Bytes)
            .map_err(|err| StorageError::IO {
                source: StorageIOError::write_state_machine(&std::io::Error::other(format!(
                    "failed to encode {command_name} raw result: {err}",
                ))),
            })
    }};
}

macro_rules! operation_error {
    ($ty:ty, $err:expr) => {{
        let command_name = command_name!($ty);

        StorageError::IO {
            source: StorageIOError::write_state_machine(&std::io::Error::other(format!(
                "{command_name} apply failed: {}",
                $err,
            ))),
        }
    }};
}

impl StateMachineHandler<DbTypeConfig> for DbStateMachine {
    type Snapshot = Stores;

    fn apply(&mut self, data: &DbCommand) -> Result<DbResponse, StorageError<u64>> {
        match data {
            DbCommand::CreateStore(payload) => {
                let params = decode_payload!(query::CreateStore, payload)?;
                operations::create_store(&self.store_handler, params)
                    .map(|_| DbResponse::Unit)
                    .map_err(|err| operation_error!(query::CreateStore, err))
            }
            DbCommand::CreatePredIndex(payload) => {
                let params = decode_payload!(query::CreatePredIndex, payload)?;
                let created = operations::create_pred_index(&self.store_handler, params)
                    .map_err(|err| operation_error!(query::CreatePredIndex, err))?;

                encode_raw_result!(query::CreatePredIndex, &created)
            }
            DbCommand::CreateNonLinearAlgorithmIndex(payload) => {
                let params = decode_payload!(query::CreateNonLinearAlgorithmIndex, payload)?;
                let created =
                    operations::create_non_linear_algorithm_index(&self.store_handler, params)
                        .map_err(|err| {
                            operation_error!(query::CreateNonLinearAlgorithmIndex, err)
                        })?;

                encode_raw_result!(query::CreateNonLinearAlgorithmIndex, &created)
            }
            DbCommand::Set(payload) => {
                let params = decode_payload!(query::Set, payload)?;
                let upsert = operations::set(&self.store_handler, params)
                    .map_err(|err| operation_error!(query::Set, err))?;

                encode_raw_result!(query::Set, &upsert)
            }
            DbCommand::DelKey(payload) => {
                let params = decode_payload!(query::DelKey, payload)?;
                let deleted = operations::del_key(&self.store_handler, params)
                    .map_err(|err| operation_error!(query::DelKey, err))?;

                encode_raw_result!(query::DelKey, &deleted)
            }
            DbCommand::DelPred(payload) => {
                let params = decode_payload!(query::DelPred, payload)?;
                let deleted = operations::del_pred(&self.store_handler, params)
                    .map_err(|err| operation_error!(query::DelPred, err))?;

                encode_raw_result!(query::DelPred, &deleted)
            }
            DbCommand::DropPredIndex(payload) => {
                let params = decode_payload!(query::DropPredIndex, payload)?;
                let deleted = operations::drop_pred_index(&self.store_handler, params)
                    .map_err(|err| operation_error!(query::DropPredIndex, err))?;

                encode_raw_result!(query::DropPredIndex, &deleted)
            }
            DbCommand::DropNonLinearAlgorithmIndex(payload) => {
                let params = decode_payload!(query::DropNonLinearAlgorithmIndex, payload)?;
                let deleted =
                    operations::drop_non_linear_algorithm_index(&self.store_handler, params)
                        .map_err(|err| operation_error!(query::DropNonLinearAlgorithmIndex, err))?;

                encode_raw_result!(query::DropNonLinearAlgorithmIndex, &deleted)
            }
            DbCommand::DropStore(payload) => {
                let params = decode_payload!(query::DropStore, payload)?;
                let dropped = operations::drop_store(&self.store_handler, params)
                    .map_err(|err| operation_error!(query::DropStore, err))?;

                encode_raw_result!(query::DropStore, &dropped)
            }
        }
    }

    fn get_snapshot(&self) -> Self::Snapshot {
        self.store_handler.get_snapshot()
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.store_handler.use_snapshot(snapshot);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::AtomicBool;

    use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
    use ahnlich_types::metadata::MetadataValue;

    use super::*;

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
            create_predicates: Vec::new(),
            non_linear_indices: Vec::new(),
            error_if_exists: true,
        }
    }

    fn set_query(store: &str) -> query::Set {
        query::Set {
            store: store.to_owned(),
            inputs: vec![DbStoreEntry {
                key: Some(StoreKey {
                    key: vec![1.0, 2.0],
                }),
                value: Some(store_value("alpha")),
            }],
        }
    }

    fn drop_store_query(store: &str) -> query::DropStore {
        query::DropStore {
            store: store.to_owned(),
            error_if_not_exists: true,
        }
    }

    fn encode_command<M: Message>(query: &M, wrap: impl FnOnce(Vec<u8>) -> DbCommand) -> DbCommand {
        wrap(query.encode_to_vec())
    }

    fn decode_set_response(response: DbResponse) -> ahnlich_types::shared::info::StoreUpsert {
        match response {
            DbResponse::Bytes(bytes) => {
                bitcode::deserialize(bytes.as_slice()).expect("decode set response")
            }
            DbResponse::Unit => panic!("expected byte response"),
        }
    }

    fn decode_count_response(response: DbResponse) -> usize {
        match response {
            DbResponse::Bytes(bytes) => {
                bitcode::deserialize(bytes.as_slice()).expect("decode count response")
            }
            DbResponse::Unit => panic!("expected byte response"),
        }
    }

    #[test]
    fn apply_create_store_and_set_dispatches_and_mutates_state() {
        let mut state_machine = DbStateMachine::new(Arc::new(AtomicBool::new(false)));

        let create_response = state_machine
            .apply(&encode_command(
                &create_store_query("products", 2),
                DbCommand::CreateStore,
            ))
            .expect("create store should succeed");
        assert!(matches!(create_response, DbResponse::Unit));

        let set_response = state_machine
            .apply(&encode_command(&set_query("products"), DbCommand::Set))
            .expect("set should succeed");
        let set = decode_set_response(set_response);
        assert_eq!(
            set,
            ahnlich_types::shared::info::StoreUpsert {
                inserted: 1,
                updated: 0,
            }
        );

        let stores = operations::list_stores(state_machine.store_handler());
        assert_eq!(stores.stores.len(), 1);
        assert_eq!(stores.stores[0].name, "products");
        assert_eq!(stores.stores[0].len, 1);
        assert_eq!(stores.stores[0].dimension, 2);
        assert!(stores.stores[0].predicate_indices.is_empty());
        assert!(stores.stores[0].non_linear_indices.is_empty());
        assert!(
            state_machine
                .store_handler()
                .write_flag
                .load(std::sync::atomic::Ordering::SeqCst)
        );
    }

    #[test]
    fn apply_drop_store_dispatches_and_returns_deleted_count() {
        let mut state_machine = DbStateMachine::new(Arc::new(AtomicBool::new(false)));

        state_machine
            .apply(&encode_command(
                &create_store_query("products", 2),
                DbCommand::CreateStore,
            ))
            .expect("create store should succeed");

        let response = state_machine
            .apply(&encode_command(
                &drop_store_query("products"),
                DbCommand::DropStore,
            ))
            .expect("drop store should succeed");
        let deleted_count = decode_count_response(response);
        assert_eq!(deleted_count, 1);
        assert!(
            operations::list_stores(state_machine.store_handler())
                .stores
                .is_empty()
        );
    }

    #[test]
    fn apply_rejects_malformed_payload_without_mutating_state() {
        let mut state_machine = DbStateMachine::new(Arc::new(AtomicBool::new(false)));

        let err = state_machine
            .apply(&DbCommand::CreateStore(vec![0xff, 0x01, 0x02]))
            .expect_err("malformed payload should fail");

        assert!(matches!(err, StorageError::IO { .. }));
        assert!(
            operations::list_stores(state_machine.store_handler())
                .stores
                .is_empty()
        );
        assert!(
            !state_machine
                .store_handler()
                .write_flag
                .load(std::sync::atomic::Ordering::SeqCst)
        );
    }
}
