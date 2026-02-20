use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;

use ahnlich_replication::storage::{
    CombinedStorage, MemLogStore, RocksLogStore, StateMachineHandler, StateMachineStore,
};
use ahnlich_replication::types::{
    AiCommand, AiResponse, AiResponseKind, ClientWriteRequest, ClientWriteResponse,
};
use ahnlich_types::ai::{query, server};
use ahnlich_types::keyval::StoreName;
use openraft::OptionalSend;
use openraft::{
    BasicNode, Entry, LogState, RaftLogReader, RaftStorage, RaftTypeConfig, Snapshot, SnapshotMeta,
    StorageError, StorageIOError, StoredMembership, Vote, declare_raft_types,
};
use prost::Message;

use crate::engine::store::{AIStoreHandler, AIStores};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AiSnapshot {
    pub stores: AIStores,
    pub client_last: HashMap<String, (u64, AiResponse)>,
}

#[derive(Debug)]
pub struct AiStateMachineHandler {
    store_handler: Arc<AIStoreHandler>,
    client_last: HashMap<String, (u64, AiResponse)>,
}

impl AiStateMachineHandler {
    pub fn new(store_handler: Arc<AIStoreHandler>) -> Self {
        Self {
            store_handler,
            client_last: HashMap::new(),
        }
    }
}

declare_raft_types!(
    pub TypeConfig:
        D = ClientWriteRequest<AiCommand>,
        R = ClientWriteResponse<AiResponse>,
        NodeId = u64,
        Node = BasicNode,
        SnapshotData = Cursor<Vec<u8>>,
        Entry = Entry<TypeConfig>
);

impl StateMachineHandler<TypeConfig> for AiStateMachineHandler {
    type Snapshot = AiSnapshot;

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
            AiCommand::CreateStore(payload) => {
                let req =
                    query::CreateStore::decode(payload.as_slice()).map_err(map_storage_error)?;
                let query_model = req.query_model.try_into().map_err(map_storage_error)?;
                let index_model = req.index_model.try_into().map_err(map_storage_error)?;

                self.store_handler
                    .create_store(
                        StoreName { value: req.store },
                        query_model,
                        index_model,
                        req.error_if_exists,
                        req.store_original,
                    )
                    .map_err(map_storage_error)?;
                AiResponse {
                    kind: AiResponseKind::Unit,
                    payload: server::Unit {}.encode_to_vec(),
                }
            }
            AiCommand::DropStore(payload) => {
                let req =
                    query::DropStore::decode(payload.as_slice()).map_err(map_storage_error)?;
                let deleted = self
                    .store_handler
                    .drop_store(StoreName { value: req.store }, req.error_if_not_exists)
                    .map_err(map_storage_error)?;
                AiResponse {
                    kind: AiResponseKind::Del,
                    payload: server::Del {
                        deleted_count: deleted as u64,
                    }
                    .encode_to_vec(),
                }
            }
            AiCommand::PurgeStores(payload) => {
                let _ =
                    query::PurgeStores::decode(payload.as_slice()).map_err(map_storage_error)?;
                let deleted = self.store_handler.purge_stores();
                AiResponse {
                    kind: AiResponseKind::Del,
                    payload: server::Del {
                        deleted_count: deleted as u64,
                    }
                    .encode_to_vec(),
                }
            }
            _ => {
                return Err(map_storage_error(
                    "unsupported command in AI raft state machine",
                ));
            }
        };

        self.client_last
            .insert(data.client_id.clone(), (data.request_id, response.clone()));

        Ok(ClientWriteResponse { response })
    }

    fn get_snapshot(&self) -> Self::Snapshot {
        AiSnapshot {
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
    store_handler: Arc<AIStoreHandler>,
) -> StateMachineStore<TypeConfig, AiStateMachineHandler> {
    let handler = AiStateMachineHandler::new(store_handler);
    let membership = StoredMembership::default();
    StateMachineStore::new(handler, membership)
}

pub fn make_mem_storage(
    store_handler: Arc<AIStoreHandler>,
) -> CombinedStorage<MemLogStore<TypeConfig>, StateMachineStore<TypeConfig, AiStateMachineHandler>>
{
    CombinedStorage {
        log: MemLogStore::default(),
        state_machine: make_state_machine_store(store_handler),
    }
}

pub type MemStorage =
    CombinedStorage<MemLogStore<TypeConfig>, StateMachineStore<TypeConfig, AiStateMachineHandler>>;

pub type RocksStorage = CombinedStorage<
    RocksLogStore<TypeConfig>,
    StateMachineStore<TypeConfig, AiStateMachineHandler>,
>;

#[derive(Debug, Clone)]
pub enum AiStorage {
    Mem(MemStorage),
    Rocks(RocksStorage),
}

impl RaftLogReader<TypeConfig> for AiStorage {
    async fn try_get_log_entries<
        RB: std::ops::RangeBounds<u64> + Clone + std::fmt::Debug + OptionalSend,
    >(
        &mut self,
        range: RB,
    ) -> Result<Vec<<TypeConfig as RaftTypeConfig>::Entry>, StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.try_get_log_entries(range).await,
            AiStorage::Rocks(s) => s.try_get_log_entries(range).await,
        }
    }
}

impl RaftStorage<TypeConfig> for AiStorage {
    type LogReader = AiStorage;
    type SnapshotBuilder = <MemStorage as RaftStorage<TypeConfig>>::SnapshotBuilder;

    async fn save_vote(&mut self, vote: &Vote<u64>) -> Result<(), StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.save_vote(vote).await,
            AiStorage::Rocks(s) => s.save_vote(vote).await,
        }
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<u64>>, StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.read_vote().await,
            AiStorage::Rocks(s) => s.read_vote().await,
        }
    }

    async fn save_committed(
        &mut self,
        committed: Option<openraft::LogId<u64>>,
    ) -> Result<(), StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.save_committed(committed).await,
            AiStorage::Rocks(s) => s.save_committed(committed).await,
        }
    }

    async fn read_committed(&mut self) -> Result<Option<openraft::LogId<u64>>, StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.read_committed().await,
            AiStorage::Rocks(s) => s.read_committed().await,
        }
    }

    async fn get_log_state(&mut self) -> Result<LogState<TypeConfig>, StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.get_log_state().await,
            AiStorage::Rocks(s) => s.get_log_state().await,
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
            AiStorage::Mem(s) => s.append_to_log(entries).await,
            AiStorage::Rocks(s) => s.append_to_log(entries).await,
        }
    }

    async fn delete_conflict_logs_since(
        &mut self,
        log_id: openraft::LogId<u64>,
    ) -> Result<(), StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.delete_conflict_logs_since(log_id).await,
            AiStorage::Rocks(s) => s.delete_conflict_logs_since(log_id).await,
        }
    }

    async fn purge_logs_upto(
        &mut self,
        log_id: openraft::LogId<u64>,
    ) -> Result<(), StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.purge_logs_upto(log_id).await,
            AiStorage::Rocks(s) => s.purge_logs_upto(log_id).await,
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
            AiStorage::Mem(s) => s.last_applied_state().await,
            AiStorage::Rocks(s) => s.last_applied_state().await,
        }
    }

    async fn apply_to_state_machine(
        &mut self,
        entries: &[<TypeConfig as RaftTypeConfig>::Entry],
    ) -> Result<Vec<<TypeConfig as RaftTypeConfig>::R>, StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.apply_to_state_machine(entries).await,
            AiStorage::Rocks(s) => s.apply_to_state_machine(entries).await,
        }
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        match self {
            AiStorage::Mem(s) => s.get_snapshot_builder().await,
            AiStorage::Rocks(s) => s.get_snapshot_builder().await,
        }
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<<TypeConfig as RaftTypeConfig>::SnapshotData>, StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.begin_receiving_snapshot().await,
            AiStorage::Rocks(s) => s.begin_receiving_snapshot().await,
        }
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<u64, BasicNode>,
        snapshot: Box<<TypeConfig as RaftTypeConfig>::SnapshotData>,
    ) -> Result<(), StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.install_snapshot(meta, snapshot).await,
            AiStorage::Rocks(s) => s.install_snapshot(meta, snapshot).await,
        }
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<TypeConfig>>, StorageError<u64>> {
        match self {
            AiStorage::Mem(s) => s.get_current_snapshot().await,
            AiStorage::Rocks(s) => s.get_current_snapshot().await,
        }
    }
}

pub fn make_rocks_storage(
    store_handler: Arc<AIStoreHandler>,
    path: impl AsRef<std::path::Path>,
) -> Result<
    CombinedStorage<
        RocksLogStore<TypeConfig>,
        StateMachineStore<TypeConfig, AiStateMachineHandler>,
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
    use crate::cli::server::SupportedModels;
    use ahnlich_types::ai::models::AiModel;
    use ahnlich_types::ai::query::{CreateStore, DelKey, DropStore, PurgeStores};
    use ahnlich_types::keyval::StoreName;
    use prost::Message;
    use std::sync::atomic::AtomicBool;

    fn new_state_machine() -> AiStateMachineHandler {
        let write_flag = Arc::new(AtomicBool::new(false));
        let store_handler = Arc::new(AIStoreHandler::new(
            write_flag,
            vec![SupportedModels::AllMiniLML6V2],
        ));
        AiStateMachineHandler::new(store_handler)
    }

    fn create_store_command(store: &str) -> ClientWriteRequest<AiCommand> {
        let req = CreateStore {
            store: store.to_string(),
            query_model: AiModel::AllMiniLmL6V2 as i32,
            index_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };

        ClientWriteRequest {
            client_id: format!("client-{store}"),
            request_id: 1,
            command: AiCommand::CreateStore(req.encode_to_vec()),
        }
    }

    fn store_count(snapshot: &AiSnapshot) -> usize {
        let guard = snapshot.stores.guard();
        snapshot.stores.iter(&guard).count()
    }

    #[test]
    fn apply_create_store_writes_store() {
        let mut sm = new_state_machine();
        let applied = sm
            .apply(&create_store_command("test_store"))
            .expect("create_store apply should succeed");

        assert_eq!(applied.response.kind, AiResponseKind::Unit);
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
        sm.apply(&create_store_command("drop_me"))
            .expect("seed create_store should succeed");

        let req = DropStore {
            store: "drop_me".to_string(),
            error_if_not_exists: true,
        };
        let applied = sm
            .apply(&ClientWriteRequest {
                client_id: "c2".to_string(),
                request_id: 2,
                command: AiCommand::DropStore(req.encode_to_vec()),
            })
            .expect("drop_store apply should succeed");

        assert_eq!(applied.response.kind, AiResponseKind::Del);
        let del =
            server::Del::decode(applied.response.payload.as_slice()).expect("valid Del payload");
        assert_eq!(del.deleted_count, 1);
        assert_eq!(store_count(&sm.get_snapshot()), 0);
    }

    #[test]
    fn apply_purge_stores_deletes_all() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("s1"))
            .expect("seed create_store s1 should succeed");
        sm.apply(&create_store_command("s2"))
            .expect("seed create_store s2 should succeed");
        assert_eq!(store_count(&sm.get_snapshot()), 2);

        let applied = sm
            .apply(&ClientWriteRequest {
                client_id: "c3".to_string(),
                request_id: 3,
                command: AiCommand::PurgeStores(PurgeStores {}.encode_to_vec()),
            })
            .expect("purge_stores apply should succeed");

        assert_eq!(applied.response.kind, AiResponseKind::Del);
        let del =
            server::Del::decode(applied.response.payload.as_slice()).expect("valid Del payload");
        assert_eq!(del.deleted_count, 2);
        assert_eq!(store_count(&sm.get_snapshot()), 0);
    }

    #[test]
    fn apply_idempotent_replay_returns_cached_response() {
        let mut sm = new_state_machine();
        sm.apply(&create_store_command("replay_store"))
            .expect("seed create_store should succeed");
        assert_eq!(store_count(&sm.get_snapshot()), 1);

        let request = ClientWriteRequest {
            client_id: "same-client".to_string(),
            request_id: 10,
            command: AiCommand::PurgeStores(PurgeStores {}.encode_to_vec()),
        };

        let first = sm
            .apply(&request)
            .expect("first purge apply should succeed")
            .response;
        let count_after_first = store_count(&sm.get_snapshot());
        let second = sm
            .apply(&request)
            .expect("second purge apply should return cached response")
            .response;
        let count_after_second = store_count(&sm.get_snapshot());

        assert_eq!(first.kind, second.kind);
        assert_eq!(first.payload, second.payload);
        assert_eq!(count_after_first, 0);
        assert_eq!(count_after_second, 0);
    }

    #[test]
    fn apply_unsupported_command_returns_error() {
        let mut sm = new_state_machine();
        let initial_count = store_count(&sm.get_snapshot());

        let unsupported = ClientWriteRequest {
            client_id: "c4".to_string(),
            request_id: 4,
            command: AiCommand::DelKey(
                DelKey {
                    store: "unused".to_string(),
                    keys: vec![],
                }
                .encode_to_vec(),
            ),
        };

        let result = sm.apply(&unsupported);
        assert!(result.is_err(), "unsupported command should fail");
        assert_eq!(store_count(&sm.get_snapshot()), initial_count);
    }
}
