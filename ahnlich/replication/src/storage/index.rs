// `clippy::result_large_err` and `clippy::type_complexity` fire on signatures
// that are dictated by openraft traits (`StorageError` is a fat enum from
// upstream; the tuple shape returned by `last_applied_state_sync` is the
// `RaftStorage` trait's contract). Boxing the error or aliasing the tuple
// here would only obscure the openraft-mandated shape.
#![allow(clippy::result_large_err, clippy::type_complexity)]

use std::fmt::Debug;
use std::ops::RangeBounds;

use openraft::{
    LogId, LogState, OptionalSend, RaftLogReader, RaftStorage, RaftTypeConfig, Snapshot,
    SnapshotMeta, StorageError, StoredMembership, Vote,
};

pub type LogIdOf<C> = LogId<<C as RaftTypeConfig>::NodeId>;

pub trait LogStoreOps<C: RaftTypeConfig>: RaftLogReader<C> + Send + Sync + Clone + 'static {
    fn get_log_state_sync(&mut self) -> Result<LogState<C>, StorageError<C::NodeId>>;
    fn save_vote_sync(&mut self, vote: &Vote<C::NodeId>) -> Result<(), StorageError<C::NodeId>>;
    fn read_vote_sync(&mut self) -> Result<Option<Vote<C::NodeId>>, StorageError<C::NodeId>>;
    fn append_to_log_sync<I>(&mut self, entries: I) -> Result<(), StorageError<C::NodeId>>
    where
        I: IntoIterator<Item = C::Entry> + OptionalSend;
    fn delete_conflict_logs_since_sync(
        &mut self,
        log_id: LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>>;
    fn purge_logs_upto_sync(
        &mut self,
        log_id: LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>>;
    fn save_committed_sync(
        &mut self,
        committed: Option<LogId<C::NodeId>>,
    ) -> Result<(), StorageError<C::NodeId>>;
    fn read_committed_sync(&mut self) -> Result<Option<LogId<C::NodeId>>, StorageError<C::NodeId>>;
}

pub trait StateMachineOps<C: RaftTypeConfig>: Send + Sync + 'static {
    type SnapshotBuilder: openraft::RaftSnapshotBuilder<C>;
    fn last_applied_state_sync(
        &mut self,
    ) -> Result<
        (
            Option<LogId<C::NodeId>>,
            StoredMembership<C::NodeId, C::Node>,
        ),
        StorageError<C::NodeId>,
    >;
    fn apply_to_state_machine_sync(
        &mut self,
        entries: &[C::Entry],
    ) -> Result<Vec<C::R>, StorageError<C::NodeId>>;
    fn get_snapshot_builder_sync(&mut self) -> Self::SnapshotBuilder;
    fn begin_receiving_snapshot_sync(
        &mut self,
    ) -> Result<Box<C::SnapshotData>, StorageError<C::NodeId>>;
    fn install_snapshot_sync(
        &mut self,
        meta: &SnapshotMeta<C::NodeId, C::Node>,
        snapshot: Box<C::SnapshotData>,
    ) -> Result<(), StorageError<C::NodeId>>;
    fn get_current_snapshot_sync(&mut self)
    -> Result<Option<Snapshot<C>>, StorageError<C::NodeId>>;
}

#[derive(Debug, Clone)]
pub struct CombinedStorage<L, SM> {
    pub log: L,
    pub state_machine: SM,
}

impl<C: RaftTypeConfig, L, SM> RaftLogReader<C> for CombinedStorage<L, SM>
where
    L: LogStoreOps<C>,
    SM: StateMachineOps<C>,
{
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> Result<Vec<C::Entry>, StorageError<C::NodeId>> {
        self.log.try_get_log_entries(range).await
    }
}

impl<C: RaftTypeConfig, L, SM> RaftStorage<C> for CombinedStorage<L, SM>
where
    L: LogStoreOps<C>,
    SM: StateMachineOps<C>,
{
    type LogReader = L;
    type SnapshotBuilder = SM::SnapshotBuilder;

    async fn save_vote(&mut self, vote: &Vote<C::NodeId>) -> Result<(), StorageError<C::NodeId>> {
        self.log.save_vote_sync(vote)
    }

    async fn read_vote(&mut self) -> Result<Option<Vote<C::NodeId>>, StorageError<C::NodeId>> {
        self.log.read_vote_sync()
    }

    async fn save_committed(
        &mut self,
        committed: Option<LogId<C::NodeId>>,
    ) -> Result<(), StorageError<C::NodeId>> {
        self.log.save_committed_sync(committed)
    }

    async fn read_committed(
        &mut self,
    ) -> Result<Option<LogId<C::NodeId>>, StorageError<C::NodeId>> {
        self.log.read_committed_sync()
    }

    async fn get_log_state(&mut self) -> Result<LogState<C>, StorageError<C::NodeId>> {
        self.log.get_log_state_sync()
    }

    async fn get_log_reader(&mut self) -> Self::LogReader {
        self.log.clone()
    }

    async fn append_to_log<I>(&mut self, entries: I) -> Result<(), StorageError<C::NodeId>>
    where
        I: IntoIterator<Item = C::Entry> + OptionalSend,
    {
        self.log.append_to_log_sync(entries)
    }

    async fn delete_conflict_logs_since(
        &mut self,
        log_id: LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        self.log.delete_conflict_logs_since_sync(log_id)
    }

    async fn purge_logs_upto(
        &mut self,
        log_id: LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        self.log.purge_logs_upto_sync(log_id)
    }

    async fn last_applied_state(
        &mut self,
    ) -> Result<
        (
            Option<LogId<C::NodeId>>,
            StoredMembership<C::NodeId, C::Node>,
        ),
        StorageError<C::NodeId>,
    > {
        self.state_machine.last_applied_state_sync()
    }

    async fn apply_to_state_machine(
        &mut self,
        entries: &[C::Entry],
    ) -> Result<Vec<C::R>, StorageError<C::NodeId>> {
        self.state_machine.apply_to_state_machine_sync(entries)
    }

    async fn get_snapshot_builder(&mut self) -> Self::SnapshotBuilder {
        self.state_machine.get_snapshot_builder_sync()
    }

    async fn begin_receiving_snapshot(
        &mut self,
    ) -> Result<Box<C::SnapshotData>, StorageError<C::NodeId>> {
        self.state_machine.begin_receiving_snapshot_sync()
    }

    async fn install_snapshot(
        &mut self,
        meta: &SnapshotMeta<C::NodeId, C::Node>,
        snapshot: Box<C::SnapshotData>,
    ) -> Result<(), StorageError<C::NodeId>> {
        self.state_machine.install_snapshot_sync(meta, snapshot)
    }

    async fn get_current_snapshot(
        &mut self,
    ) -> Result<Option<Snapshot<C>>, StorageError<C::NodeId>> {
        self.state_machine.get_current_snapshot_sync()
    }
}
