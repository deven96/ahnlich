use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use ahnlich_replication::storage::{
    CombinedStorage, MemLogStore, RocksLogStore, StateMachineHandler, StateMachineStore,
};
use ahnlich_replication::types::{
    ClientWriteRequest, ClientWriteResponse, DbCommand, DbResponse, DbResponseKind,
};
use ahnlich_types::db::{query, server};
use ahnlich_types::keyval::StoreName;
use openraft::OptionalSend;
use openraft::{
    BasicNode, Entry, LogState, RaftLogReader, RaftStorage, RaftTypeConfig, Snapshot, SnapshotMeta,
    StorageError, StorageIOError, StoredMembership, Vote, declare_raft_types,
};
use prost::Message;

use crate::engine::store::{StoreHandler, Stores};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DbSnapshot {
    pub stores: Stores,
    pub client_last: HashMap<String, (u64, DbResponse)>,
}

#[derive(Debug)]
pub struct DbStateMachineHandler {
    store_handler: Arc<StoreHandler>,
    client_last: HashMap<String, (u64, DbResponse)>,
}

impl DbStateMachineHandler {
    pub fn new(store_handler: Arc<StoreHandler>) -> Self {
        Self {
            store_handler,
            client_last: HashMap::new(),
        }
    }
}

declare_raft_types!(
    pub TypeConfig:
        D = ClientWriteRequest<DbCommand>,
        R = ClientWriteResponse<DbResponse>,
        NodeId = u64,
        Node = BasicNode,
        SnapshotData = Cursor<Vec<u8>>,
        Entry = Entry<TypeConfig>
);

impl StateMachineHandler<TypeConfig> for DbStateMachineHandler {
    type Snapshot = DbSnapshot;

    fn apply(
        &mut self,
        data: &<TypeConfig as RaftTypeConfig>::D,
    ) -> Result<<TypeConfig as RaftTypeConfig>::R, StorageError<u64>> {
        if let Some((last_id, last_resp)) = self.client_last.get(&data.client_id) {
            if *last_id >= data.request_id {
                return Ok(ClientWriteResponse {
                    response: last_resp.clone(),
                });
            }
        }

        let response = match &data.command {
            DbCommand::CreateStore(payload) => {
                let req =
                    query::CreateStore::decode(payload.as_slice()).map_err(map_storage_error)?;
                let dimensions = std::num::NonZeroUsize::new(req.dimension as usize)
                    .ok_or_else(|| map_storage_error("dimension must be > 0"))?;
                let non_linear = req
                    .non_linear_indices
                    .into_iter()
                    .filter_map(|idx| {
                        ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm::try_from(idx).ok()
                    })
                    .collect();
                self.store_handler
                    .create_store(
                        StoreName { value: req.store },
                        dimensions,
                        req.create_predicates.into_iter().collect(),
                        non_linear,
                        req.error_if_exists,
                    )
                    .map_err(map_storage_error)?;
                DbResponse {
                    kind: DbResponseKind::Unit,
                    payload: server::Unit {}.encode_to_vec(),
                }
            }
            DbCommand::CreatePredIndex(payload) => {
                let req = query::CreatePredIndex::decode(payload.as_slice())
                    .map_err(map_storage_error)?;
                let created = self
                    .store_handler
                    .create_pred_index(&StoreName { value: req.store }, req.predicates)
                    .map_err(map_storage_error)?;
                DbResponse {
                    kind: DbResponseKind::CreateIndex,
                    payload: server::CreateIndex {
                        created_indexes: created as u64,
                    }
                    .encode_to_vec(),
                }
            }
            DbCommand::CreateNonLinearAlgorithmIndex(payload) => {
                let req = query::CreateNonLinearAlgorithmIndex::decode(payload.as_slice())
                    .map_err(map_storage_error)?;
                let indices = req
                    .non_linear_indices
                    .into_iter()
                    .filter_map(|idx| {
                        ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm::try_from(idx).ok()
                    })
                    .collect();
                let created = self
                    .store_handler
                    .create_non_linear_algorithm_index(&StoreName { value: req.store }, indices)
                    .map_err(map_storage_error)?;
                DbResponse {
                    kind: DbResponseKind::CreateIndex,
                    payload: server::CreateIndex {
                        created_indexes: created as u64,
                    }
                    .encode_to_vec(),
                }
            }
            DbCommand::Set(payload) => {
                let req = query::Set::decode(payload.as_slice()).map_err(map_storage_error)?;
                let inputs = req
                    .inputs
                    .into_iter()
                    .filter_map(|entry| match (entry.key, entry.value) {
                        (Some(key), Some(value)) => Some((key, value)),
                        _ => None,
                    })
                    .collect();
                let set = self
                    .store_handler
                    .set_in_store(&StoreName { value: req.store }, inputs)
                    .map_err(map_storage_error)?;
                DbResponse {
                    kind: DbResponseKind::Set,
                    payload: server::Set { upsert: Some(set) }.encode_to_vec(),
                }
            }
            DbCommand::DropPredIndex(payload) => {
                let req =
                    query::DropPredIndex::decode(payload.as_slice()).map_err(map_storage_error)?;
                let deleted = self
                    .store_handler
                    .drop_pred_index_in_store(
                        &StoreName { value: req.store },
                        req.predicates,
                        req.error_if_not_exists,
                    )
                    .map_err(map_storage_error)?;
                DbResponse {
                    kind: DbResponseKind::Del,
                    payload: server::Del {
                        deleted_count: deleted as u64,
                    }
                    .encode_to_vec(),
                }
            }
            DbCommand::DropNonLinearAlgorithmIndex(payload) => {
                let req = query::DropNonLinearAlgorithmIndex::decode(payload.as_slice())
                    .map_err(map_storage_error)?;
                let deleted = self
                    .store_handler
                    .drop_non_linear_algorithm_index(
                        &StoreName { value: req.store },
                        req.non_linear_indices
                            .into_iter()
                            .filter_map(|idx| {
                                ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm::try_from(
                                    idx,
                                )
                                .ok()
                            })
                            .collect(),
                        req.error_if_not_exists,
                    )
                    .map_err(map_storage_error)?;
                DbResponse {
                    kind: DbResponseKind::Del,
                    payload: server::Del {
                        deleted_count: deleted as u64,
                    }
                    .encode_to_vec(),
                }
            }
            DbCommand::DelKey(payload) => {
                let req = query::DelKey::decode(payload.as_slice()).map_err(map_storage_error)?;
                let keys = req
                    .keys
                    .into_iter()
                    .map(|k| ahnlich_types::keyval::StoreKey { key: k.key })
                    .collect();
                let deleted = self
                    .store_handler
                    .del_key_in_store(&StoreName { value: req.store }, keys)
                    .map_err(map_storage_error)?;
                DbResponse {
                    kind: DbResponseKind::Del,
                    payload: server::Del {
                        deleted_count: deleted as u64,
                    }
                    .encode_to_vec(),
                }
            }
            DbCommand::DelPred(payload) => {
                let req = query::DelPred::decode(payload.as_slice()).map_err(map_storage_error)?;
                let condition = req
                    .condition
                    .ok_or_else(|| map_storage_error("Predicate Condition is required"))?;
                let deleted = self
                    .store_handler
                    .del_pred_in_store(&StoreName { value: req.store }, &condition)
                    .map_err(map_storage_error)?;
                DbResponse {
                    kind: DbResponseKind::Del,
                    payload: server::Del {
                        deleted_count: deleted as u64,
                    }
                    .encode_to_vec(),
                }
            }
            DbCommand::DropStore(payload) => {
                let req =
                    query::DropStore::decode(payload.as_slice()).map_err(map_storage_error)?;
                let deleted = self
                    .store_handler
                    .drop_store(StoreName { value: req.store }, req.error_if_not_exists)
                    .map_err(map_storage_error)?;
                DbResponse {
                    kind: DbResponseKind::Del,
                    payload: server::Del {
                        deleted_count: deleted as u64,
                    }
                    .encode_to_vec(),
                }
            }
        };

        self.client_last
            .insert(data.client_id.clone(), (data.request_id, response.clone()));

        Ok(ClientWriteResponse { response })
    }

    fn get_snapshot(&self) -> Self::Snapshot {
        DbSnapshot {
            stores: self.store_handler.get_stores(),
            client_last: self.client_last.clone(),
        }
    }

    fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
        self.store_handler.use_snapshot(snapshot.stores);
        self.client_last = snapshot.client_last;
    }
}

fn map_storage_error<E: ToString>(err: E) -> StorageError<u64> {
    StorageError::IO {
        source: StorageIOError::write_state_machine(&std::io::Error::other(err.to_string())),
    }
}

pub fn make_state_machine_store(
    store_handler: Arc<StoreHandler>,
) -> StateMachineStore<TypeConfig, DbStateMachineHandler> {
    let handler = DbStateMachineHandler::new(store_handler);
    let membership = StoredMembership::default();
    StateMachineStore::new(handler, membership)
}

pub fn make_mem_storage(
    store_handler: Arc<StoreHandler>,
) -> CombinedStorage<MemLogStore<TypeConfig>, StateMachineStore<TypeConfig, DbStateMachineHandler>>
{
    CombinedStorage {
        log: MemLogStore::default(),
        state_machine: make_state_machine_store(store_handler),
    }
}

pub type MemStorage =
    CombinedStorage<MemLogStore<TypeConfig>, StateMachineStore<TypeConfig, DbStateMachineHandler>>;

pub type RocksStorage = CombinedStorage<
    RocksLogStore<TypeConfig>,
    StateMachineStore<TypeConfig, DbStateMachineHandler>,
>;

#[derive(Debug, Clone)]
pub enum DbStorage {
    Mem(MemStorage),
    Rocks(RocksStorage),
}

impl RaftLogReader<TypeConfig> for DbStorage {
    async fn try_get_log_entries<
        RB: std::ops::RangeBounds<u64> + Clone + std::fmt::Debug + OptionalSend,
    >(
        &mut self,
        range: RB,
    ) -> Result<Vec<<TypeConfig as RaftTypeConfig>::Entry>, StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.try_get_log_entries(range).await,
            DbStorage::Rocks(s) => s.try_get_log_entries(range).await,
        }
    }
}

impl RaftStorage<TypeConfig> for DbStorage {
    type LogReader = DbStorage;
    type SnapshotBuilder = <MemStorage as RaftStorage<TypeConfig>>::SnapshotBuilder;

    async fn save_vote(&mut self, vote: &Vote<u64>) -> Result<(), StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.save_vote(vote).await,
            DbStorage::Rocks(s) => s.save_vote(vote).await,
        }
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<u64>>, StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.read_vote().await,
            DbStorage::Rocks(s) => s.read_vote().await,
        }
    }

    async fn save_committed(
        &mut self,
        committed: Option<openraft::LogId<u64>>,
    ) -> Result<(), StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.save_committed(committed).await,
            DbStorage::Rocks(s) => s.save_committed(committed).await,
        }
    }

    async fn read_committed(&mut self) -> Result<Option<openraft::LogId<u64>>, StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.read_committed().await,
            DbStorage::Rocks(s) => s.read_committed().await,
        }
    }

    async fn get_log_state(&mut self) -> Result<LogState<TypeConfig>, StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.get_log_state().await,
            DbStorage::Rocks(s) => s.get_log_state().await,
        }
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.clone()
    }

    async fn append_to_log<I>(&mut self, entries: I) -> Result<(), StorageError<u64>>
    where
        I: IntoIterator<Item = <TypeConfig as RaftTypeConfig>::Entry> + OptionalSend,
    {
        match self {
            DbStorage::Mem(s) => s.append_to_log(entries).await,
            DbStorage::Rocks(s) => s.append_to_log(entries).await,
        }
    }

    async fn delete_conflict_logs_since(
        &mut self,
        log_id: openraft::LogId<u64>,
    ) -> Result<(), StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.delete_conflict_logs_since(log_id).await,
            DbStorage::Rocks(s) => s.delete_conflict_logs_since(log_id).await,
        }
    }

    async fn purge_logs_upto(
        &mut self,
        log_id: openraft::LogId<u64>,
    ) -> Result<(), StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.purge_logs_upto(log_id).await,
            DbStorage::Rocks(s) => s.purge_logs_upto(log_id).await,
        }
    }

    async fn last_applied_state(
        &mut self,
    ) -> Result<
        (
            Option<openraft::LogId<u64>>,
            StoredMembership<u64, BasicNode>,
        ),
        StorageError<u64>,
    > {
        match self {
            DbStorage::Mem(s) => s.last_applied_state().await,
            DbStorage::Rocks(s) => s.last_applied_state().await,
        }
    }

    async fn apply_to_state_machine(
        &mut self,
        entries: &[<TypeConfig as RaftTypeConfig>::Entry],
    ) -> Result<Vec<<TypeConfig as RaftTypeConfig>::R>, StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.apply_to_state_machine(entries).await,
            DbStorage::Rocks(s) => s.apply_to_state_machine(entries).await,
        }
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        match self {
            DbStorage::Mem(s) => s.get_snapshot_builder().await,
            DbStorage::Rocks(s) => s.get_snapshot_builder().await,
        }
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<<TypeConfig as RaftTypeConfig>::SnapshotData>, StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.begin_receiving_snapshot().await,
            DbStorage::Rocks(s) => s.begin_receiving_snapshot().await,
        }
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<u64, BasicNode>,
        snapshot: Box<<TypeConfig as RaftTypeConfig>::SnapshotData>,
    ) -> Result<(), StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.install_snapshot(meta, snapshot).await,
            DbStorage::Rocks(s) => s.install_snapshot(meta, snapshot).await,
        }
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<TypeConfig>>, StorageError<u64>> {
        match self {
            DbStorage::Mem(s) => s.get_current_snapshot().await,
            DbStorage::Rocks(s) => s.get_current_snapshot().await,
        }
    }
}

pub fn make_rocks_storage(
    store_handler: Arc<StoreHandler>,
    path: impl AsRef<std::path::Path>,
) -> Result<
    CombinedStorage<
        RocksLogStore<TypeConfig>,
        StateMachineStore<TypeConfig, DbStateMachineHandler>,
    >,
    StorageError<u64>,
> {
    let log = RocksLogStore::open(path)?;
    Ok(CombinedStorage {
        log,
        state_machine: make_state_machine_store(store_handler),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
    use ahnlich_types::db::query::{
        CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey, DelPred,
        DropNonLinearAlgorithmIndex, DropPredIndex, DropStore, Set,
    };
    use ahnlich_types::keyval::StoreName;
    use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
    use ahnlich_types::metadata::{MetadataValue, metadata_value::Value as MetadataValueEnum};
    use ahnlich_types::predicates::{
        Predicate, PredicateCondition, predicate::Kind as PredicateKind,
        predicate_condition::Kind as PredicateConditionKind,
    };
    use prost::Message;
    use std::collections::HashMap;
    use std::sync::atomic::AtomicBool;

    fn new_state_machine() -> DbStateMachineHandler {
        let write_flag = Arc::new(AtomicBool::new(false));
        let store_handler = Arc::new(StoreHandler::new(write_flag));
        DbStateMachineHandler::new(store_handler)
    }

    fn create_store_command(
        store: &str,
        client_id: &str,
        request_id: u64,
    ) -> ClientWriteRequest<DbCommand> {
        let req = CreateStore {
            store: store.to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
        };

        ClientWriteRequest {
            client_id: client_id.to_string(),
            request_id,
            command: DbCommand::CreateStore(req.encode_to_vec()),
        }
    }

    fn store_count(snapshot: &DbSnapshot) -> usize {
        let guard = snapshot.stores.guard();
        snapshot.stores.iter(&guard).count()
    }

    fn set_command(
        store: &str,
        key: Vec<f32>,
        metadata: HashMap<String, MetadataValue>,
        client_id: &str,
        request_id: u64,
    ) -> ClientWriteRequest<DbCommand> {
        let req = Set {
            store: store.to_string(),
            inputs: vec![DbStoreEntry {
                key: Some(StoreKey { key }),
                value: Some(StoreValue { value: metadata }),
            }],
        };
        ClientWriteRequest {
            client_id: client_id.to_string(),
            request_id,
            command: DbCommand::Set(req.encode_to_vec()),
        }
    }

    #[test]
    fn apply_create_store_writes_store() {
        let mut sm = new_state_machine();
        let applied = sm
            .apply(&create_store_command("test_store", "c1", 1))
            .expect("create_store apply should succeed");

        assert_eq!(applied.response.kind, DbResponseKind::Unit);
        let _: server::Unit =
            server::Unit::decode(applied.response.payload.as_slice()).expect("valid Unit payload");

        let snapshot = sm.get_snapshot();
        let guard = snapshot.stores.guard();
        let has_store = snapshot.stores.iter(&guard).any(|(name, _)| {
            name == &StoreName {
                value: "test_store".to_string(),
            }
        });
        assert!(has_store, "store should exist in state machine snapshot");
    }

    #[test]
    fn apply_drop_store_deletes_store() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("drop_me", "seed", 1))
            .expect("seed create_store should succeed");

        let req = DropStore {
            store: "drop_me".to_string(),
            error_if_not_exists: true,
        };
        let applied = sm
            .apply(&ClientWriteRequest {
                client_id: "c2".to_string(),
                request_id: 2,
                command: DbCommand::DropStore(req.encode_to_vec()),
            })
            .expect("drop_store apply should succeed");

        assert_eq!(applied.response.kind, DbResponseKind::Del);
        let del =
            server::Del::decode(applied.response.payload.as_slice()).expect("valid Del payload");
        assert_eq!(del.deleted_count, 1);
        assert_eq!(store_count(&sm.get_snapshot()), 0);
    }

    #[test]
    fn apply_idempotent_replay_returns_cached_response() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("replay_store", "seed", 1))
            .expect("seed create_store should succeed");
        assert_eq!(store_count(&sm.get_snapshot()), 1);

        let request = ClientWriteRequest {
            client_id: "same-client".to_string(),
            request_id: 10,
            command: DbCommand::DropStore(
                DropStore {
                    store: "replay_store".to_string(),
                    error_if_not_exists: true,
                }
                .encode_to_vec(),
            ),
        };

        let first = sm
            .apply(&request)
            .expect("first drop_store apply should succeed")
            .response;
        let count_after_first = store_count(&sm.get_snapshot());
        let second = sm
            .apply(&request)
            .expect("second drop_store apply should return cached response")
            .response;
        let count_after_second = store_count(&sm.get_snapshot());

        assert_eq!(first.kind, second.kind);
        assert_eq!(first.payload, second.payload);
        assert_eq!(count_after_first, 0);
        assert_eq!(count_after_second, 0);
    }

    #[test]
    fn apply_set_writes_entries() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("set_store", "seed", 1))
            .expect("seed create_store should succeed");

        let applied = sm
            .apply(&set_command(
                "set_store",
                vec![1.0, 2.0, 3.0],
                HashMap::new(),
                "set-client",
                2,
            ))
            .expect("set apply should succeed");

        assert_eq!(applied.response.kind, DbResponseKind::Set);
        let set =
            server::Set::decode(applied.response.payload.as_slice()).expect("valid Set payload");
        assert!(
            set.upsert.is_some(),
            "set response should include upsert info"
        );
    }

    #[test]
    fn apply_del_key_deletes_expected_count() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("key_store", "seed", 1))
            .expect("seed create_store should succeed");
        sm.apply(&set_command(
            "key_store",
            vec![9.0, 8.0, 7.0],
            HashMap::new(),
            "seed",
            2,
        ))
        .expect("seed set should succeed");

        let del_req = DelKey {
            store: "key_store".to_string(),
            keys: vec![StoreKey {
                key: vec![9.0, 8.0, 7.0],
            }],
        };
        let applied = sm
            .apply(&ClientWriteRequest {
                client_id: "del-key".to_string(),
                request_id: 3,
                command: DbCommand::DelKey(del_req.encode_to_vec()),
            })
            .expect("del_key apply should succeed");

        assert_eq!(applied.response.kind, DbResponseKind::Del);
        let del =
            server::Del::decode(applied.response.payload.as_slice()).expect("valid Del payload");
        assert_eq!(del.deleted_count, 1);
    }

    #[test]
    fn apply_del_pred_deletes_expected_count() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("pred_store", "seed", 1))
            .expect("seed create_store should succeed");
        let metadata = HashMap::from([(
            "role".to_string(),
            MetadataValue {
                value: Some(MetadataValueEnum::RawString("mage".to_string())),
            },
        )]);
        sm.apply(&set_command(
            "pred_store",
            vec![3.0, 2.0, 1.0],
            metadata,
            "seed",
            2,
        ))
        .expect("seed set should succeed");

        let del_pred = DelPred {
            store: "pred_store".to_string(),
            condition: Some(PredicateCondition {
                kind: Some(PredicateConditionKind::Value(Predicate {
                    kind: Some(PredicateKind::Equals(ahnlich_types::predicates::Equals {
                        key: "role".to_string(),
                        value: Some(MetadataValue {
                            value: Some(MetadataValueEnum::RawString("mage".to_string())),
                        }),
                    })),
                })),
            }),
        };
        let applied = sm
            .apply(&ClientWriteRequest {
                client_id: "del-pred".to_string(),
                request_id: 3,
                command: DbCommand::DelPred(del_pred.encode_to_vec()),
            })
            .expect("del_pred apply should succeed");

        assert_eq!(applied.response.kind, DbResponseKind::Del);
        let del =
            server::Del::decode(applied.response.payload.as_slice()).expect("valid Del payload");
        assert_eq!(del.deleted_count, 1);
    }

    #[test]
    fn apply_create_pred_index_returns_created_count() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("index_store", "seed", 1))
            .expect("seed create_store should succeed");

        let req = CreatePredIndex {
            store: "index_store".to_string(),
            predicates: vec!["planet".to_string()],
        };
        let applied = sm
            .apply(&ClientWriteRequest {
                client_id: "pred-index".to_string(),
                request_id: 2,
                command: DbCommand::CreatePredIndex(req.encode_to_vec()),
            })
            .expect("create_pred_index apply should succeed");

        assert_eq!(applied.response.kind, DbResponseKind::CreateIndex);
        let out = server::CreateIndex::decode(applied.response.payload.as_slice())
            .expect("valid CreateIndex payload");
        assert_eq!(out.created_indexes, 1);
    }

    #[test]
    fn apply_drop_pred_index_returns_deleted_count() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("drop_index_store", "seed", 1))
            .expect("seed create_store should succeed");
        sm.apply(&ClientWriteRequest {
            client_id: "seed-index".to_string(),
            request_id: 2,
            command: DbCommand::CreatePredIndex(
                CreatePredIndex {
                    store: "drop_index_store".to_string(),
                    predicates: vec!["planet".to_string()],
                }
                .encode_to_vec(),
            ),
        })
        .expect("seed create_pred_index should succeed");

        let req = DropPredIndex {
            store: "drop_index_store".to_string(),
            predicates: vec!["planet".to_string()],
            error_if_not_exists: true,
        };
        let applied = sm
            .apply(&ClientWriteRequest {
                client_id: "drop-index".to_string(),
                request_id: 3,
                command: DbCommand::DropPredIndex(req.encode_to_vec()),
            })
            .expect("drop_pred_index apply should succeed");

        assert_eq!(applied.response.kind, DbResponseKind::Del);
        let del =
            server::Del::decode(applied.response.payload.as_slice()).expect("valid Del payload");
        assert_eq!(del.deleted_count, 1);
    }

    #[test]
    fn apply_create_non_linear_algorithm_index_returns_created_count() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("nl_store", "seed", 1))
            .expect("seed create_store should succeed");

        let req = CreateNonLinearAlgorithmIndex {
            store: "nl_store".to_string(),
            non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
        };
        let applied = sm
            .apply(&ClientWriteRequest {
                client_id: "nl-create".to_string(),
                request_id: 2,
                command: DbCommand::CreateNonLinearAlgorithmIndex(req.encode_to_vec()),
            })
            .expect("create_non_linear_algorithm_index apply should succeed");

        assert_eq!(applied.response.kind, DbResponseKind::CreateIndex);
        let out = server::CreateIndex::decode(applied.response.payload.as_slice())
            .expect("valid CreateIndex payload");
        assert_eq!(out.created_indexes, 1);
    }

    #[test]
    fn apply_drop_non_linear_algorithm_index_returns_deleted_count() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("nl_drop_store", "seed", 1))
            .expect("seed create_store should succeed");
        sm.apply(&ClientWriteRequest {
            client_id: "nl-seed".to_string(),
            request_id: 2,
            command: DbCommand::CreateNonLinearAlgorithmIndex(
                CreateNonLinearAlgorithmIndex {
                    store: "nl_drop_store".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                }
                .encode_to_vec(),
            ),
        })
        .expect("seed create_non_linear_algorithm_index should succeed");

        let req = DropNonLinearAlgorithmIndex {
            store: "nl_drop_store".to_string(),
            non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
            error_if_not_exists: true,
        };
        let applied = sm
            .apply(&ClientWriteRequest {
                client_id: "nl-drop".to_string(),
                request_id: 3,
                command: DbCommand::DropNonLinearAlgorithmIndex(req.encode_to_vec()),
            })
            .expect("drop_non_linear_algorithm_index apply should succeed");

        assert_eq!(applied.response.kind, DbResponseKind::Del);
        let del =
            server::Del::decode(applied.response.payload.as_slice()).expect("valid Del payload");
        assert_eq!(del.deleted_count, 1);
    }

    #[test]
    fn apply_malformed_payload_returns_error() {
        let mut sm = new_state_machine();
        let initial_count = store_count(&sm.get_snapshot());

        let malformed = ClientWriteRequest {
            client_id: "c3".to_string(),
            request_id: 3,
            command: DbCommand::CreateStore(vec![1, 2, 3]),
        };

        let result = sm.apply(&malformed);
        assert!(result.is_err(), "malformed payload should fail");
        assert_eq!(store_count(&sm.get_snapshot()), initial_count);
    }
}
