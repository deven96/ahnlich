// `clippy::result_large_err` and `clippy::type_complexity` fire on signatures
// that are dictated by openraft traits (`StorageError` is a fat enum from
// upstream; the tuple shape returned by `applied_state_sync` mirrors the
// openraft state-machine contract). Boxing the error or aliasing the tuple
// here would only obscure that upstream-mandated shape.
#![allow(clippy::result_large_err, clippy::type_complexity)]

// State machine storage for replicated application state.
//
// Raft splits persistence into two different responsibilities:
//
// 1. the log, which records proposed operations in order; and
// 2. the state machine, which applies committed operations and becomes the
//    authoritative application state.
//
// This module implements the second half for Ahnlich. It does not define the
// DB-specific or AI-specific mutations themselves; those live behind
// `StateMachineHandler`. What it does own is the generic machinery every
// replicated state machine needs:
//
// - applying committed entries in order,
// - tracking `last_applied`,
// - tracking the latest effective membership,
// - building snapshots from the current state, and
// - restoring state from an installed snapshot.
//
// In other words: the log says what happened, and this module is responsible
// for making that history real in application state.

use std::io::Cursor;
use std::sync::RwLock;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

use openraft::entry::RaftPayload;
use openraft::{
    Entry, EntryPayload, LogId, OptionalSend, RaftLogId, RaftSnapshotBuilder, RaftTypeConfig,
    Snapshot, SnapshotMeta, StorageError, StorageIOError, StoredMembership,
    storage::RaftStateMachine,
};
use utils::snapshot::{deserialize_snapshot, serialize_snapshot};

use super::LogIdOf;

#[derive(Debug, Default)]
pub struct ReplicationFailureState {
    failed: AtomicBool,
    reason: RwLock<Option<String>>,
}

impl ReplicationFailureState {
    pub fn mark_failed(&self, reason: impl Into<String>) {
        self.failed.store(true, Ordering::SeqCst);
        *self
            .reason
            .write()
            .expect("replication failure state lock poisoned") = Some(reason.into());
    }

    pub fn failed(&self) -> bool {
        self.failed.load(Ordering::SeqCst)
    }

    pub fn reason(&self) -> Option<String> {
        self.reason
            .read()
            .expect("replication failure state lock poisoned")
            .clone()
    }
}

pub trait StateMachineHandler<C: RaftTypeConfig>: Send + Sync + 'static {
    type Snapshot: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static;

    // Apply a single committed command to application state.
    //
    // Ordinary domain/precondition validation must happen before
    // `Raft::client_write`. By the time a command reaches committed replay, it
    // should already be semantically valid.
    //
    // Errors returned from `apply()` are therefore reserved for genuine
    // storage, corruption, or invariant failures. In practice, such a failure
    // is treated as node-fatal by the surrounding replication runtime.
    fn apply(&mut self, data: &C::D) -> Result<C::R, StorageError<C::NodeId>>;
    fn get_snapshot(&self) -> Result<Self::Snapshot, StorageError<C::NodeId>>;
    fn restore_snapshot(&mut self, snapshot: Self::Snapshot);
}

// State machine store. Generic over a `StateMachineHandler` so the DB and AI
// state machines plug in without re-implementing the
// snapshot bookkeeping.

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersistedSnapshot<C: RaftTypeConfig> {
    pub meta: SnapshotMeta<C::NodeId, C::Node>,
    pub data: Vec<u8>,
    pub snapshot_idx: u64,
}

pub trait StateMachineSnapshotStore<C: RaftTypeConfig>: Send + Sync + 'static {
    fn load_snapshot(&self) -> Result<Option<PersistedSnapshot<C>>, StorageError<C::NodeId>>;
    fn persist_snapshot(
        &self,
        snapshot: &PersistedSnapshot<C>,
    ) -> Result<(), StorageError<C::NodeId>>;
}

#[derive(Debug, Default)]
pub struct MemorySnapshotStore<C: RaftTypeConfig> {
    snapshot: Mutex<Option<PersistedSnapshot<C>>>,
}

impl<C: RaftTypeConfig> StateMachineSnapshotStore<C> for MemorySnapshotStore<C> {
    fn load_snapshot(&self) -> Result<Option<PersistedSnapshot<C>>, StorageError<C::NodeId>> {
        Ok(self
            .snapshot
            .lock()
            .expect("memory snapshot store lock poisoned")
            .clone())
    }

    fn persist_snapshot(
        &self,
        snapshot: &PersistedSnapshot<C>,
    ) -> Result<(), StorageError<C::NodeId>> {
        *self
            .snapshot
            .lock()
            .expect("memory snapshot store lock poisoned") = Some(snapshot.clone());
        Ok(())
    }
}

#[derive(Debug)]
struct StateMachineInner<C: RaftTypeConfig, H: StateMachineHandler<C>> {
    handler: H,
    last_applied: Option<LogIdOf<C>>,
    last_membership: StoredMembership<C::NodeId, C::Node>,
    snapshot_idx: u64,
    current_snapshot: Option<PersistedSnapshot<C>>,
}

pub struct StateMachineStore<C: RaftTypeConfig, H: StateMachineHandler<C>> {
    inner: Arc<Mutex<StateMachineInner<C, H>>>,
    failure_state: Arc<ReplicationFailureState>,
    snapshot_store: Arc<dyn StateMachineSnapshotStore<C>>,
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> std::fmt::Debug for StateMachineStore<C, H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateMachineStore").finish_non_exhaustive()
    }
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> Clone for StateMachineStore<C, H> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            failure_state: self.failure_state.clone(),
            snapshot_store: self.snapshot_store.clone(),
        }
    }
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> StateMachineStore<C, H> {
    pub fn new(
        handler: H,
        initial_membership: StoredMembership<C::NodeId, C::Node>,
        failure_state: Arc<ReplicationFailureState>,
        snapshot_store: Arc<dyn StateMachineSnapshotStore<C>>,
    ) -> Result<Self, StorageError<C::NodeId>> {
        let persisted_snapshot = snapshot_store.load_snapshot()?;
        let mut handler = handler;
        let (last_applied, last_membership, snapshot_idx, current_snapshot) =
            if let Some(snapshot) = persisted_snapshot {
                let decoded = deserialize_snapshot::<H::Snapshot>(&snapshot.data).map_err(|e| {
                    failure_state
                        .mark_failed(format!("failed to restore state machine snapshot: {e}"));
                    StorageError::IO {
                        source: StorageIOError::read_state_machine(&e),
                    }
                })?;
                handler.restore_snapshot(decoded);
                (
                    snapshot.meta.last_log_id.clone(),
                    snapshot.meta.last_membership.clone(),
                    snapshot.snapshot_idx,
                    Some(snapshot),
                )
            } else {
                (None, initial_membership, 0, None)
            };

        Ok(Self {
            inner: Arc::new(Mutex::new(StateMachineInner {
                handler,
                last_applied,
                last_membership,
                snapshot_idx,
                current_snapshot,
            })),
            failure_state,
            snapshot_store,
        })
    }

    pub fn failure_state(&self) -> Arc<ReplicationFailureState> {
        self.failure_state.clone()
    }

    pub fn with_handler<R>(&self, f: impl FnOnce(&H) -> R) -> Result<R, StorageError<C::NodeId>> {
        let inner = self.lock_inner()?;
        Ok(f(&inner.handler))
    }

    fn lock_inner(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, StateMachineInner<C, H>>, StorageError<C::NodeId>> {
        self.inner.lock().map_err(|_| {
            let reason = "state machine lock poisoned";
            self.failure_state.mark_failed(reason);
            StorageError::IO {
                source: StorageIOError::read_state_machine(&std::io::Error::other(reason)),
            }
        })
    }
}

#[derive(Clone)]
pub struct SnapshotBuilder<C: RaftTypeConfig, H: StateMachineHandler<C>> {
    inner: Arc<Mutex<StateMachineInner<C, H>>>,
    failure_state: Arc<ReplicationFailureState>,
    snapshot_store: Arc<dyn StateMachineSnapshotStore<C>>,
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> SnapshotBuilder<C, H> {
    fn lock_inner(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, StateMachineInner<C, H>>, StorageError<C::NodeId>> {
        self.inner.lock().map_err(|_| {
            let reason = "state machine lock poisoned";
            self.failure_state.mark_failed(reason);
            StorageError::IO {
                source: StorageIOError::read_state_machine(&std::io::Error::other(reason)),
            }
        })
    }
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> RaftSnapshotBuilder<C> for SnapshotBuilder<C, H>
where
    C::SnapshotData: From<Cursor<Vec<u8>>>,
{
    async fn build_snapshot(&mut self) -> Result<Snapshot<C>, StorageError<C::NodeId>> {
        let (snapshot, meta, snapshot_idx) = {
            let mut inner = self.lock_inner()?;
            let snapshot = inner.handler.get_snapshot()?;

            inner.snapshot_idx += 1;

            let meta = SnapshotMeta {
                last_log_id: inner.last_applied.clone(),
                last_membership: inner.last_membership.clone(),
                snapshot_id: format!(
                    "{}-{}-{}",
                    inner
                        .last_applied
                        .clone()
                        .map(|l| l.leader_id.term)
                        .unwrap_or_default(),
                    inner
                        .last_applied
                        .clone()
                        .map(|l| l.index)
                        .unwrap_or_default(),
                    inner.snapshot_idx
                ),
            };

            (snapshot, meta, inner.snapshot_idx)
        };

        let encoded = serialize_snapshot(&snapshot).map_err(|e| StorageError::IO {
            source: StorageIOError::write_state_machine(&e),
        })?;

        let stored = PersistedSnapshot {
            meta: meta.clone(),
            data: encoded.clone(),
            snapshot_idx,
        };
        self.snapshot_store.persist_snapshot(&stored)?;

        let mut inner = self.lock_inner()?;
        inner.current_snapshot = Some(stored);

        Ok(Snapshot {
            meta,
            snapshot: Box::new(C::SnapshotData::from(Cursor::new(encoded))),
        })
    }
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> StateMachineStore<C, H>
where
    C::Entry: AsRef<Entry<C>> + RaftLogId<C::NodeId> + Clone,
    C::SnapshotData: From<Cursor<Vec<u8>>> + Into<Cursor<Vec<u8>>>,
    C::R: Default,
{
    fn applied_state_sync(
        &self,
    ) -> Result<
        (
            Option<LogId<C::NodeId>>,
            StoredMembership<C::NodeId, C::Node>,
        ),
        StorageError<C::NodeId>,
    > {
        let inner = self.lock_inner()?;
        Ok((inner.last_applied.clone(), inner.last_membership.clone()))
    }

    fn apply_sync<I>(&self, entries: I) -> Result<Vec<C::R>, StorageError<C::NodeId>>
    where
        I: IntoIterator<Item = C::Entry>,
    {
        let mut inner = self.lock_inner()?;
        let mut responses = Vec::new();

        for entry in entries {
            let e = entry.as_ref();

            // Replay is sequential rather than batch-atomic. Handlers are
            // expected to reserve errors for invariant/storage failures, not
            // routine domain validation.
            let log_id = e.get_log_id().clone();

            let response = match &e.payload {
                EntryPayload::Normal(data) => inner.handler.apply(data).map_err(|err| {
                    self.failure_state
                        .mark_failed(format!("state machine apply failed: {err}"));
                    err
                })?,
                _ => C::R::default(),
            };

            inner.last_applied = Some(log_id.clone());

            if let Some(m) = e.get_membership() {
                inner.last_membership = StoredMembership::new(Some(log_id), m.clone());
            }
            responses.push(response);
        }

        Ok(responses)
    }

    fn get_snapshot_builder_sync(&self) -> SnapshotBuilder<C, H> {
        SnapshotBuilder {
            inner: self.inner.clone(),
            failure_state: self.failure_state.clone(),
            snapshot_store: self.snapshot_store.clone(),
        }
    }

    fn begin_receiving_snapshot_sync(
        &self,
    ) -> Result<Box<C::SnapshotData>, StorageError<C::NodeId>> {
        Ok(Box::new(C::SnapshotData::from(Cursor::new(Vec::new()))))
    }

    fn install_snapshot_sync(
        &self,
        meta: &SnapshotMeta<C::NodeId, C::Node>,
        snapshot: C::SnapshotData,
    ) -> Result<(), StorageError<C::NodeId>> {
        let cursor: Cursor<Vec<u8>> = snapshot.into();
        let data = cursor.into_inner();
        let decoded = deserialize_snapshot::<H::Snapshot>(&data).map_err(|e| StorageError::IO {
            source: StorageIOError::read_state_machine(&e),
        })?;

        let mut inner = self.lock_inner()?;
        inner.snapshot_idx += 1;
        let stored = PersistedSnapshot {
            meta: meta.clone(),
            data,
            snapshot_idx: inner.snapshot_idx,
        };
        self.snapshot_store.persist_snapshot(&stored)?;
        inner.handler.restore_snapshot(decoded);
        inner.last_applied = meta.last_log_id.clone();
        inner.last_membership = meta.last_membership.clone();
        inner.current_snapshot = Some(stored);
        Ok(())
    }

    fn get_current_snapshot_sync(&self) -> Result<Option<Snapshot<C>>, StorageError<C::NodeId>> {
        let inner = self.lock_inner()?;
        let Some(stored) = &inner.current_snapshot else {
            return Ok(None);
        };

        Ok(Some(Snapshot {
            meta: stored.meta.clone(),
            snapshot: Box::new(C::SnapshotData::from(Cursor::new(stored.data.clone()))),
        }))
    }
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> RaftStateMachine<C> for StateMachineStore<C, H>
where
    C::Entry: AsRef<Entry<C>> + RaftLogId<C::NodeId> + Clone,
    C::SnapshotData: From<Cursor<Vec<u8>>> + Into<Cursor<Vec<u8>>>,
    C::R: Default,
{
    type SnapshotBuilder = SnapshotBuilder<C, H>;

    async fn applied_state(
        &mut self,
    ) -> Result<
        (
            Option<LogId<C::NodeId>>,
            StoredMembership<C::NodeId, C::Node>,
        ),
        StorageError<C::NodeId>,
    > {
        self.applied_state_sync()
    }

    async fn apply<I>(&mut self, entries: I) -> Result<Vec<C::R>, StorageError<C::NodeId>>
    where
        I: IntoIterator<Item = C::Entry> + OptionalSend,
        I::IntoIter: OptionalSend,
    {
        self.apply_sync(entries)
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.get_snapshot_builder_sync()
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<C::SnapshotData>, StorageError<C::NodeId>> {
        self.begin_receiving_snapshot_sync()
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<C::NodeId, C::Node>,
        snapshot: Box<C::SnapshotData>,
    ) -> Result<(), StorageError<C::NodeId>> {
        self.install_snapshot_sync(meta, *snapshot)
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<C>>, StorageError<C::NodeId>> {
        self.get_current_snapshot_sync()
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::io::Cursor;
    use std::sync::{Arc, Mutex};

    use openraft::{CommittedLeaderId, Entry, EntryPayload, Membership};

    use super::*;
    use crate::node::ReplicationNode;
    use crate::storage::RocksLogStore;

    openraft::declare_raft_types!(
        pub TestConfig:
            D = String,
            R = String,
            Node = ReplicationNode,
            SnapshotData = Cursor<Vec<u8>>
    );

    #[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
    struct TestSnapshot {
        applied: Vec<String>,
    }

    #[derive(Debug, Default)]
    struct HandlerState {
        applied: Vec<String>,
        fail_on: Option<String>,
    }

    #[derive(Debug, Clone)]
    struct TestHandler {
        state: Arc<Mutex<HandlerState>>,
    }

    impl TestHandler {
        fn new(fail_on: Option<&str>) -> Self {
            Self {
                state: Arc::new(Mutex::new(HandlerState {
                    applied: Vec::new(),
                    fail_on: fail_on.map(ToOwned::to_owned),
                })),
            }
        }

        fn applied(&self) -> Vec<String> {
            self.state
                .lock()
                .expect("test handler lock poisoned")
                .applied
                .clone()
        }
    }

    impl StateMachineHandler<TestConfig> for TestHandler {
        type Snapshot = TestSnapshot;

        fn apply(&mut self, data: &String) -> Result<String, StorageError<u64>> {
            let mut state = self.state.lock().expect("test handler lock poisoned");
            if state.fail_on.as_ref() == Some(data) {
                return Err(StorageError::IO {
                    source: StorageIOError::write_state_machine(&std::io::Error::other(
                        "intentional apply failure",
                    )),
                });
            }
            state.applied.push(data.clone());
            Ok(format!("applied:{data}"))
        }

        fn get_snapshot(&self) -> Result<Self::Snapshot, StorageError<u64>> {
            Ok(TestSnapshot {
                applied: self.applied(),
            })
        }

        fn restore_snapshot(&mut self, snapshot: Self::Snapshot) {
            let mut state = self.state.lock().expect("test handler lock poisoned");
            state.applied = snapshot.applied;
        }
    }

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

    fn normal_entry(term: u64, node_id: u64, index: u64, data: &str) -> Entry<TestConfig> {
        Entry {
            log_id: log_id(term, node_id, index),
            payload: EntryPayload::Normal(data.to_owned()),
        }
    }

    fn membership_entry(term: u64, node_id: u64, index: u64, voters: &[u64]) -> Entry<TestConfig> {
        Entry {
            log_id: log_id(term, node_id, index),
            payload: EntryPayload::Membership(membership(voters)),
        }
    }

    #[test]
    fn poisoned_lock_marks_replication_failure_state() {
        let handler = TestHandler::new(None);
        let failure_state = Arc::new(ReplicationFailureState::default());
        let store = StateMachineStore::<TestConfig, _>::new(
            handler,
            StoredMembership::new(None, membership(&[1])),
            failure_state.clone(),
            Arc::new(MemorySnapshotStore::default()),
        )
        .expect("create state machine store");

        let poison_target = store.clone();
        let _ = std::thread::spawn(move || {
            let _guard = poison_target
                .inner
                .lock()
                .expect("lock should succeed before poison");
            panic!("poison state machine lock");
        })
        .join();

        let err = store
            .applied_state_sync()
            .expect_err("poisoned lock should return storage error");
        assert!(matches!(err, StorageError::IO { .. }));
        assert!(failure_state.failed());
        assert_eq!(
            failure_state.reason(),
            Some("state machine lock poisoned".to_owned())
        );
    }

    #[test]
    fn apply_failure_marks_replication_failure_state() {
        let handler = TestHandler::new(Some("boom"));
        let failure_state = Arc::new(ReplicationFailureState::default());
        let store = StateMachineStore::<TestConfig, _>::new(
            handler,
            StoredMembership::new(None, membership(&[1])),
            failure_state.clone(),
            Arc::new(MemorySnapshotStore::default()),
        )
        .expect("create state machine store");

        let err = store
            .apply_sync(vec![normal_entry(1, 1, 1, "boom")])
            .expect_err("apply should fail");
        assert!(matches!(err, StorageError::IO { .. }));
        assert!(failure_state.failed());
        let reason = failure_state
            .reason()
            .expect("failure reason should be recorded");
        assert!(reason.contains("state machine apply failed"));
        assert!(reason.contains("intentional apply failure"));
    }

    #[tokio::test]
    async fn snapshot_round_trip_restores_state_and_membership() {
        let handler = TestHandler::new(None);
        let initial_membership = StoredMembership::new(Some(log_id(1, 1, 1)), membership(&[1]));
        let mut source = StateMachineStore::<TestConfig, _>::new(
            handler.clone(),
            initial_membership,
            Arc::new(ReplicationFailureState::default()),
            Arc::new(MemorySnapshotStore::default()),
        )
        .expect("create source state machine store");

        let entries = vec![
            normal_entry(1, 1, 2, "alpha"),
            membership_entry(2, 2, 3, &[1, 2]),
            normal_entry(2, 2, 4, "beta"),
        ];
        let responses = source.apply_sync(entries).expect("apply should succeed");
        assert_eq!(
            responses,
            vec![
                "applied:alpha".to_owned(),
                String::default(),
                "applied:beta".to_owned()
            ]
        );

        let mut builder = source.get_snapshot_builder().await;
        let snapshot = builder
            .build_snapshot()
            .await
            .expect("snapshot build should succeed");
        let expected_membership = StoredMembership::new(Some(log_id(2, 2, 3)), membership(&[1, 2]));
        let Snapshot { meta, snapshot } = snapshot;

        let fresh_handler = TestHandler::new(None);
        let target = StateMachineStore::<TestConfig, _>::new(
            fresh_handler.clone(),
            StoredMembership::new(None, membership(&[9])),
            Arc::new(ReplicationFailureState::default()),
            Arc::new(MemorySnapshotStore::default()),
        )
        .expect("create target state machine store");
        target
            .install_snapshot_sync(&meta, *snapshot)
            .expect("install snapshot should succeed");

        let (last_applied, last_membership) = target
            .applied_state_sync()
            .expect("last_applied_state should succeed");
        assert_eq!(last_applied, Some(log_id(2, 2, 4)));
        assert_eq!(last_membership, expected_membership.clone());
        assert_eq!(
            fresh_handler.applied(),
            vec!["alpha".to_owned(), "beta".to_owned()]
        );

        let current_snapshot = target
            .get_current_snapshot_sync()
            .expect("current snapshot should succeed")
            .expect("snapshot should be stored");
        assert_eq!(current_snapshot.meta.last_log_id, Some(log_id(2, 2, 4)));
        assert_eq!(current_snapshot.meta.last_membership, expected_membership);
    }

    #[tokio::test]
    async fn durable_snapshot_restores_after_log_purge_and_reopen() {
        let dir = tempfile::tempdir().expect("tempdir");
        let snapshot_store = Arc::new(
            RocksLogStore::<TestConfig>::open(dir.path()).expect("open rocks snapshot store"),
        );
        let handler = TestHandler::new(None);
        let mut source = StateMachineStore::<TestConfig, _>::new(
            handler,
            StoredMembership::new(None, membership(&[1])),
            Arc::new(ReplicationFailureState::default()),
            snapshot_store.clone(),
        )
        .expect("create source state machine store");
        let entries = vec![
            normal_entry(1, 1, 1, "alpha"),
            membership_entry(1, 1, 2, &[1, 2]),
            normal_entry(1, 1, 3, "beta"),
        ];

        snapshot_store
            .append_to_log_sync(entries.clone())
            .expect("persist raft log entries");
        source.apply_sync(entries).expect("apply entries");
        let mut builder = source.get_snapshot_builder().await;
        builder.build_snapshot().await.expect("build snapshot");
        drop(builder);
        snapshot_store
            .purge_logs_upto_sync(log_id(1, 1, 3))
            .expect("purge compacted logs");
        drop(source);
        drop(snapshot_store);

        let reopened_store = Arc::new(
            RocksLogStore::<TestConfig>::open(dir.path()).expect("reopen rocks snapshot store"),
        );
        let restored_handler = TestHandler::new(None);
        let restored = StateMachineStore::<TestConfig, _>::new(
            restored_handler.clone(),
            StoredMembership::new(None, membership(&[9])),
            Arc::new(ReplicationFailureState::default()),
            reopened_store,
        )
        .expect("restore state machine from durable snapshot");

        assert_eq!(restored_handler.applied(), vec!["alpha", "beta"]);
        let (last_applied, last_membership) = restored
            .applied_state_sync()
            .expect("read restored applied state");
        assert_eq!(last_applied, Some(log_id(1, 1, 3)));
        assert_eq!(
            last_membership,
            StoredMembership::new(Some(log_id(1, 1, 2)), membership(&[1, 2]))
        );
    }

    #[tokio::test]
    async fn installed_snapshot_restores_after_reopen() {
        let source_handler = TestHandler::new(None);
        let mut source = StateMachineStore::<TestConfig, _>::new(
            source_handler,
            StoredMembership::new(None, membership(&[1])),
            Arc::new(ReplicationFailureState::default()),
            Arc::new(MemorySnapshotStore::default()),
        )
        .expect("create source state machine store");
        source
            .apply_sync(vec![normal_entry(1, 1, 1, "alpha")])
            .expect("apply entry");
        let mut builder = source.get_snapshot_builder().await;
        let Snapshot { meta, snapshot } = builder.build_snapshot().await.expect("build snapshot");

        let dir = tempfile::tempdir().expect("tempdir");
        let snapshot_store = Arc::new(
            RocksLogStore::<TestConfig>::open(dir.path()).expect("open rocks snapshot store"),
        );
        let target = StateMachineStore::<TestConfig, _>::new(
            TestHandler::new(None),
            StoredMembership::new(None, membership(&[9])),
            Arc::new(ReplicationFailureState::default()),
            snapshot_store.clone(),
        )
        .expect("create target state machine store");
        target
            .install_snapshot_sync(&meta, *snapshot)
            .expect("persist installed snapshot");
        drop(target);
        drop(snapshot_store);

        let reopened_store = Arc::new(
            RocksLogStore::<TestConfig>::open(dir.path()).expect("reopen rocks snapshot store"),
        );
        let restored_handler = TestHandler::new(None);
        let restored = StateMachineStore::<TestConfig, _>::new(
            restored_handler.clone(),
            StoredMembership::new(None, membership(&[9])),
            Arc::new(ReplicationFailureState::default()),
            reopened_store,
        )
        .expect("restore installed snapshot");

        assert_eq!(restored_handler.applied(), vec!["alpha"]);
        assert_eq!(
            restored
                .applied_state_sync()
                .expect("read restored applied state")
                .0,
            Some(log_id(1, 1, 1))
        );
    }
}
