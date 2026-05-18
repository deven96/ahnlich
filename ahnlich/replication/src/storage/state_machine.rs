// `clippy::result_large_err` and `clippy::type_complexity` fire on signatures
// that are dictated by openraft traits (`StorageError` is a fat enum from
// upstream; the tuple shape returned by `last_applied_state_sync` is the
// `RaftStorage` trait's contract). Boxing the error or aliasing the tuple
// here would only obscure the openraft-mandated shape.
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
use std::sync::{Arc, Mutex};

use openraft::entry::RaftPayload;
use openraft::{
    Entry, EntryPayload, LogId, RaftLogId, RaftSnapshotBuilder, RaftTypeConfig, Snapshot,
    SnapshotMeta, StorageError, StorageIOError, StoredMembership,
};
use utils::snapshot::{deserialize_snapshot, serialize_snapshot};

use super::{LogIdOf, StateMachineOps};

pub trait StateMachineHandler<C: RaftTypeConfig>: Send + Sync + 'static {
    type Snapshot: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static;

    // Apply a single committed command to application state.
    //
    // `StateMachineStore` replays committed entries sequentially, not
    // transactionally across a batch. Earlier entries in the same batch may
    // already be reflected in application state if a later one fails.
    //
    // Because of that, ordinary domain/precondition validation is expected to
    // happen before replication. Errors returned from `apply()` should
    // therefore represent invariant violations, corruption, or
    // storage/infrastructure failures rather than routine business-rule
    // rejection.
    fn apply(&mut self, data: &C::D) -> Result<C::R, StorageError<C::NodeId>>;
    fn get_snapshot(&self) -> Self::Snapshot;
    fn restore_snapshot(&mut self, snapshot: Self::Snapshot);
}

// State machine store. Generic over a `StateMachineHandler` so the DB and AI
// state machines plug in without re-implementing the
// snapshot bookkeeping.

#[derive(Debug, Clone)]
struct StoredSnapshot<C: RaftTypeConfig> {
    meta: SnapshotMeta<C::NodeId, C::Node>,
    data: Vec<u8>,
}

#[derive(Debug)]
struct StateMachineInner<C: RaftTypeConfig, H: StateMachineHandler<C>> {
    handler: H,
    last_applied: Option<LogIdOf<C>>,
    last_membership: StoredMembership<C::NodeId, C::Node>,
    snapshot_idx: u64,
    current_snapshot: Option<StoredSnapshot<C>>,
}

#[derive(Debug)]
pub struct StateMachineStore<C: RaftTypeConfig, H: StateMachineHandler<C>> {
    inner: Arc<Mutex<StateMachineInner<C, H>>>,
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> Clone for StateMachineStore<C, H> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> StateMachineStore<C, H> {
    pub fn new(handler: H, initial_membership: StoredMembership<C::NodeId, C::Node>) -> Self {
        Self {
            inner: Arc::new(Mutex::new(StateMachineInner {
                handler,
                last_applied: None,
                last_membership: initial_membership,
                snapshot_idx: 0,
                current_snapshot: None,
            })),
        }
    }

    fn lock_inner(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, StateMachineInner<C, H>>, StorageError<C::NodeId>> {
        self.inner.lock().map_err(|_| StorageError::IO {
            source: StorageIOError::read_state_machine(&std::io::Error::other(
                "state machine lock poisoned",
            )),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SnapshotBuilder<C: RaftTypeConfig, H: StateMachineHandler<C>> {
    inner: Arc<Mutex<StateMachineInner<C, H>>>,
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> SnapshotBuilder<C, H> {
    fn lock_inner(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, StateMachineInner<C, H>>, StorageError<C::NodeId>> {
        self.inner.lock().map_err(|_| StorageError::IO {
            source: StorageIOError::read_state_machine(&std::io::Error::other(
                "state machine lock poisoned",
            )),
        })
    }
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> RaftSnapshotBuilder<C> for SnapshotBuilder<C, H>
where
    C::SnapshotData: From<Cursor<Vec<u8>>>,
{
    async fn build_snapshot(&mut self) -> Result<Snapshot<C>, StorageError<C::NodeId>> {
        let (snapshot, meta) = {
            let mut inner = self.lock_inner()?;
            let snapshot = inner.handler.get_snapshot();

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

            (snapshot, meta)
        };

        let encoded = serialize_snapshot(&snapshot).map_err(|e| StorageError::IO {
            source: StorageIOError::write_state_machine(&e),
        })?;

        let mut inner = self.lock_inner()?;
        inner.current_snapshot = Some(StoredSnapshot {
            meta: meta.clone(),
            data: encoded.clone(),
        });

        Ok(Snapshot {
            meta,
            snapshot: Box::new(C::SnapshotData::from(Cursor::new(encoded))),
        })
    }
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> StateMachineOps<C> for StateMachineStore<C, H>
where
    C::Entry: AsRef<Entry<C>> + RaftLogId<C::NodeId> + Clone,
    C::SnapshotData: From<Cursor<Vec<u8>>> + Into<Cursor<Vec<u8>>>,
    C::R: Default,
{
    type SnapshotBuilder = SnapshotBuilder<C, H>;

    fn last_applied_state_sync(
        &mut self,
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

    fn apply_to_state_machine_sync(
        &mut self,
        entries: &[C::Entry],
    ) -> Result<Vec<C::R>, StorageError<C::NodeId>> {
        let mut inner = self.lock_inner()?;
        let mut responses = Vec::new();

        for entry in entries {
            let e = entry.as_ref();

            // Replay is sequential rather than batch-atomic. Handlers are
            // expected to reserve errors for invariant/storage failures, not
            // routine domain validation.
            let log_id = e.get_log_id().clone();

            let response = match &e.payload {
                EntryPayload::Normal(data) => inner.handler.apply(data)?,
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

    fn get_snapshot_builder_sync(&mut self) -> Self::SnapshotBuilder {
        SnapshotBuilder {
            inner: self.inner.clone(),
        }
    }

    fn begin_receiving_snapshot_sync(
        &mut self,
    ) -> Result<Box<C::SnapshotData>, StorageError<C::NodeId>> {
        Ok(Box::new(C::SnapshotData::from(Cursor::new(Vec::new()))))
    }

    fn install_snapshot_sync(
        &mut self,
        meta: &SnapshotMeta<C::NodeId, C::Node>,
        snapshot: Box<C::SnapshotData>,
    ) -> Result<(), StorageError<C::NodeId>> {
        let cursor: Cursor<Vec<u8>> = (*snapshot).into();
        let data = cursor.into_inner();
        let decoded = deserialize_snapshot::<H::Snapshot>(&data).map_err(|e| StorageError::IO {
            source: StorageIOError::read_state_machine(&e),
        })?;

        let mut inner = self.lock_inner()?;
        inner.handler.restore_snapshot(decoded);
        inner.last_applied = meta.last_log_id.clone();
        inner.last_membership = meta.last_membership.clone();
        inner.current_snapshot = Some(StoredSnapshot {
            meta: meta.clone(),
            data,
        });
        Ok(())
    }

    fn get_current_snapshot_sync(
        &mut self,
    ) -> Result<Option<Snapshot<C>>, StorageError<C::NodeId>> {
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

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::io::Cursor;
    use std::sync::{Arc, Mutex};

    use openraft::{CommittedLeaderId, Entry, EntryPayload, Membership};

    use super::*;
    use crate::node::ReplicationNode;

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

        fn get_snapshot(&self) -> Self::Snapshot {
            TestSnapshot {
                applied: self.applied(),
            }
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
    fn failed_apply_does_not_advance_last_applied() {
        let handler = TestHandler::new(Some("boom"));
        let mut store = StateMachineStore::<TestConfig, _>::new(
            handler.clone(),
            StoredMembership::new(None, membership(&[1])),
        );

        let entries = vec![normal_entry(1, 1, 1, "ok"), normal_entry(1, 1, 2, "boom")];

        let err = store
            .apply_to_state_machine_sync(&entries)
            .expect_err("apply should fail");
        assert!(matches!(err, StorageError::IO { .. }));

        let (last_applied, last_membership) = store
            .last_applied_state_sync()
            .expect("last_applied_state should succeed");
        assert_eq!(last_applied, Some(log_id(1, 1, 1)));
        assert_eq!(
            last_membership,
            StoredMembership::new(None, membership(&[1]))
        );
        assert_eq!(handler.applied(), vec!["ok".to_owned()]);
    }

    #[tokio::test]
    async fn snapshot_round_trip_restores_state_and_membership() {
        let handler = TestHandler::new(None);
        let initial_membership = StoredMembership::new(Some(log_id(1, 1, 1)), membership(&[1]));
        let mut source =
            StateMachineStore::<TestConfig, _>::new(handler.clone(), initial_membership);

        let entries = vec![
            normal_entry(1, 1, 2, "alpha"),
            membership_entry(2, 2, 3, &[1, 2]),
            normal_entry(2, 2, 4, "beta"),
        ];
        let responses = source
            .apply_to_state_machine_sync(&entries)
            .expect("apply should succeed");
        assert_eq!(
            responses,
            vec![
                "applied:alpha".to_owned(),
                String::default(),
                "applied:beta".to_owned()
            ]
        );

        let mut builder = source.get_snapshot_builder_sync();
        let snapshot = builder
            .build_snapshot()
            .await
            .expect("snapshot build should succeed");
        let expected_membership = StoredMembership::new(Some(log_id(2, 2, 3)), membership(&[1, 2]));

        let fresh_handler = TestHandler::new(None);
        let mut target = StateMachineStore::<TestConfig, _>::new(
            fresh_handler.clone(),
            StoredMembership::new(None, membership(&[9])),
        );
        target
            .install_snapshot_sync(&snapshot.meta, snapshot.snapshot)
            .expect("install snapshot should succeed");

        let (last_applied, last_membership) = target
            .last_applied_state_sync()
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
}
