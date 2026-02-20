use std::collections::BTreeMap;
use std::fmt::Debug;
use std::io::Cursor;
use std::ops::{Bound, RangeBounds};
#[cfg(feature = "rocksdb")]
use std::path::Path;
use std::sync::{Arc, Mutex};

use bincode::{deserialize, serialize};
use openraft::entry::RaftPayload;
use openraft::{
    Entry, EntryPayload, LogId, LogState, OptionalSend, RaftLogId, RaftLogReader,
    RaftSnapshotBuilder, RaftStorage, RaftTypeConfig, Snapshot, SnapshotMeta, StorageError,
    StorageIOError, StoredMembership, Vote,
};

#[cfg(feature = "rocksdb")]
use rocksdb::{ColumnFamilyDescriptor, IteratorMode, Options, DB};

pub type LogIdOf<C> = LogId<<C as RaftTypeConfig>::NodeId>;

fn index_in_range<RB: RangeBounds<u64>>(range: &RB, idx: u64) -> bool {
    let start_ok = match range.start_bound() {
        Bound::Included(v) => idx >= *v,
        Bound::Excluded(v) => idx > *v,
        Bound::Unbounded => true,
    };
    let end_ok = match range.end_bound() {
        Bound::Included(v) => idx <= *v,
        Bound::Excluded(v) => idx < *v,
        Bound::Unbounded => true,
    };
    start_ok && end_ok
}

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

pub trait StateMachineHandler<C: RaftTypeConfig>: Send + Sync + 'static {
    type Snapshot: serde::Serialize + serde::de::DeserializeOwned + Send + Sync + 'static;
    fn apply(&mut self, data: &C::D) -> Result<C::R, StorageError<C::NodeId>>;
    fn get_snapshot(&self) -> Self::Snapshot;
    fn restore_snapshot(&mut self, snapshot: Self::Snapshot);
}

pub trait StateMachineOps<C: RaftTypeConfig>: Send + Sync + 'static {
    type SnapshotBuilder: RaftSnapshotBuilder<C>;
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

#[derive(Debug, Clone)]
pub struct MemLogStore<C: RaftTypeConfig> {
    inner: Arc<Mutex<MemLogStoreInner<C>>>,
}

#[derive(Debug)]
struct MemLogStoreInner<C: RaftTypeConfig> {
    last_purged_log_id: Option<LogIdOf<C>>,
    log: BTreeMap<u64, C::Entry>,
    committed: Option<LogIdOf<C>>,
    vote: Option<Vote<C::NodeId>>,
}

impl<C: RaftTypeConfig> Default for MemLogStore<C> {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(MemLogStoreInner {
                last_purged_log_id: None,
                log: BTreeMap::new(),
                committed: None,
                vote: None,
            })),
        }
    }
}

impl<C: RaftTypeConfig> MemLogStore<C>
where
    C::Entry: RaftLogId<C::NodeId> + Clone,
{
    fn last_log_id(inner: &MemLogStoreInner<C>) -> Option<LogIdOf<C>> {
        inner
            .log
            .iter()
            .next_back()
            .map(|(_, ent)| ent.get_log_id().clone())
            .or_else(|| inner.last_purged_log_id.clone())
    }
}

impl<C: RaftTypeConfig> RaftLogReader<C> for MemLogStore<C>
where
    C::Entry: RaftLogId<C::NodeId> + Clone,
{
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> Result<Vec<C::Entry>, StorageError<C::NodeId>> {
        let inner = self.inner.lock().expect("mem log lock poisoned");
        let mut out = Vec::new();
        for (_, ent) in inner.log.range(range) {
            out.push(ent.clone());
        }
        Ok(out)
    }
}

impl<C: RaftTypeConfig> LogStoreOps<C> for MemLogStore<C>
where
    C::Entry: RaftLogId<C::NodeId> + Clone,
{
    fn get_log_state_sync(&mut self) -> Result<LogState<C>, StorageError<C::NodeId>> {
        let inner = self.inner.lock().expect("mem log lock poisoned");
        Ok(LogState {
            last_purged_log_id: inner.last_purged_log_id.clone(),
            last_log_id: Self::last_log_id(&inner),
        })
    }

    fn save_vote_sync(&mut self, vote: &Vote<C::NodeId>) -> Result<(), StorageError<C::NodeId>> {
        let mut inner = self.inner.lock().expect("mem log lock poisoned");
        inner.vote = Some(vote.clone());
        Ok(())
    }

    fn read_vote_sync(&mut self) -> Result<Option<Vote<C::NodeId>>, StorageError<C::NodeId>> {
        let inner = self.inner.lock().expect("mem log lock poisoned");
        Ok(inner.vote.clone())
    }

    fn append_to_log_sync<I>(&mut self, entries: I) -> Result<(), StorageError<C::NodeId>>
    where
        I: IntoIterator<Item = C::Entry> + OptionalSend,
    {
        let mut inner = self.inner.lock().expect("mem log lock poisoned");
        for entry in entries {
            inner.log.insert(entry.get_log_id().index, entry);
        }
        Ok(())
    }

    fn delete_conflict_logs_since_sync(
        &mut self,
        log_id: LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        let mut inner = self.inner.lock().expect("mem log lock poisoned");
        let keys: Vec<u64> = inner.log.range(log_id.index..).map(|(k, _)| *k).collect();
        for k in keys {
            inner.log.remove(&k);
        }
        Ok(())
    }

    fn purge_logs_upto_sync(
        &mut self,
        log_id: LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        let mut inner = self.inner.lock().expect("mem log lock poisoned");
        let keys: Vec<u64> = inner.log.range(..=log_id.index).map(|(k, _)| *k).collect();
        for k in keys {
            inner.log.remove(&k);
        }
        inner.last_purged_log_id = Some(log_id);
        Ok(())
    }

    fn save_committed_sync(
        &mut self,
        committed: Option<LogId<C::NodeId>>,
    ) -> Result<(), StorageError<C::NodeId>> {
        let mut inner = self.inner.lock().expect("mem log lock poisoned");
        inner.committed = committed;
        Ok(())
    }

    fn read_committed_sync(&mut self) -> Result<Option<LogId<C::NodeId>>, StorageError<C::NodeId>> {
        let inner = self.inner.lock().expect("mem log lock poisoned");
        Ok(inner.committed.clone())
    }
}

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
}

#[derive(Debug, Clone)]
pub struct SnapshotBuilder<C: RaftTypeConfig, H: StateMachineHandler<C>> {
    inner: Arc<Mutex<StateMachineInner<C, H>>>,
}

impl<C: RaftTypeConfig, H: StateMachineHandler<C>> RaftSnapshotBuilder<C> for SnapshotBuilder<C, H>
where
    C::SnapshotData: From<Cursor<Vec<u8>>>,
{
    async fn build_snapshot(&mut self) -> Result<Snapshot<C>, StorageError<C::NodeId>> {
        let mut inner = self.inner.lock().expect("state machine lock poisoned");
        let encoded = serialize(&inner.handler.get_snapshot()).map_err(|e| StorageError::IO {
            source: StorageIOError::write_state_machine(&e),
        })?;

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
    C::SnapshotData: From<Cursor<Vec<u8>>> + Into<Cursor<Vec<u8>>> + Clone,
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
        let inner = self.inner.lock().expect("state machine lock poisoned");
        Ok((inner.last_applied.clone(), inner.last_membership.clone()))
    }

    fn apply_to_state_machine_sync(
        &mut self,
        entries: &[C::Entry],
    ) -> Result<Vec<C::R>, StorageError<C::NodeId>> {
        let mut inner = self.inner.lock().expect("state machine lock poisoned");
        let mut responses = Vec::new();

        for entry in entries {
            let e = entry.as_ref();
            let log_id = e.get_log_id().clone();
            inner.last_applied = Some(log_id.clone());

            if let Some(m) = e.get_membership() {
                inner.last_membership = StoredMembership::new(Some(log_id), m.clone());
            }

            let response = match &e.payload {
                EntryPayload::Normal(data) => inner.handler.apply(data)?,
                _ => C::R::default(),
            };
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
        let cursor: Cursor<Vec<u8>> = (*snapshot).clone().into();
        let data = cursor.into_inner();
        let decoded = deserialize::<H::Snapshot>(&data).map_err(|e| StorageError::IO {
            source: StorageIOError::read_state_machine(&e),
        })?;

        let mut inner = self.inner.lock().expect("state machine lock poisoned");
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
        let inner = self.inner.lock().expect("state machine lock poisoned");
        let Some(stored) = &inner.current_snapshot else {
            return Ok(None);
        };

        Ok(Some(Snapshot {
            meta: stored.meta.clone(),
            snapshot: Box::new(C::SnapshotData::from(Cursor::new(stored.data.clone()))),
        }))
    }
}

#[cfg(feature = "rocksdb")]
#[derive(Debug, Clone)]
pub struct RocksLogStore<C: RaftTypeConfig> {
    db: Arc<DB>,
    _p: std::marker::PhantomData<C>,
}

#[cfg(feature = "rocksdb")]
impl<C: RaftTypeConfig> RocksLogStore<C> {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError<C::NodeId>> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        let cfs = vec![
            ColumnFamilyDescriptor::new("logs", Options::default()),
            ColumnFamilyDescriptor::new("meta", Options::default()),
        ];
        let db = DB::open_cf_descriptors(&opts, path, cfs).map_err(|e| StorageError::IO {
            source: StorageIOError::read_logs(&e),
        })?;
        Ok(Self {
            db: Arc::new(db),
            _p: std::marker::PhantomData,
        })
    }

    fn cf_logs(&self) -> Result<&rocksdb::ColumnFamily, StorageError<C::NodeId>> {
        self.db.cf_handle("logs").ok_or_else(|| StorageError::IO {
            source: StorageIOError::read_logs(&std::io::Error::other("missing logs cf")),
        })
    }

    fn cf_meta(&self) -> Result<&rocksdb::ColumnFamily, StorageError<C::NodeId>> {
        self.db.cf_handle("meta").ok_or_else(|| StorageError::IO {
            source: StorageIOError::read_logs(&std::io::Error::other("missing meta cf")),
        })
    }

    fn key(index: u64) -> [u8; 8] {
        index.to_be_bytes()
    }

    fn decode_idx(raw: &[u8]) -> Result<u64, StorageError<C::NodeId>> {
        let arr: [u8; 8] = raw.try_into().map_err(|_| StorageError::IO {
            source: StorageIOError::read_logs(&std::io::Error::other("invalid log key")),
        })?;
        Ok(u64::from_be_bytes(arr))
    }

    fn put_meta<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), StorageError<C::NodeId>> {
        let bytes = serialize(value).map_err(|e| StorageError::IO {
            source: StorageIOError::write_logs(&e),
        })?;
        self.db
            .put_cf(self.cf_meta()?, key.as_bytes(), bytes)
            .map_err(|e| StorageError::IO {
                source: StorageIOError::write_logs(&e),
            })
    }

    fn get_meta<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, StorageError<C::NodeId>> {
        let v = self
            .db
            .get_cf(self.cf_meta()?, key.as_bytes())
            .map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            })?;
        match v {
            Some(bytes) => deserialize(&bytes).map(Some).map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            }),
            None => Ok(None),
        }
    }
}

#[cfg(feature = "rocksdb")]
impl<C: RaftTypeConfig> RaftLogReader<C> for RocksLogStore<C>
where
    C::Entry: serde::Serialize + serde::de::DeserializeOwned + RaftLogId<C::NodeId> + Clone,
{
    async fn try_get_log_entries<RB: RangeBounds<u64> + Clone + Debug + OptionalSend>(
        &mut self,
        range: RB,
    ) -> Result<Vec<C::Entry>, StorageError<C::NodeId>> {
        let mut out = Vec::new();
        for item in self.db.iterator_cf(self.cf_logs()?, IteratorMode::Start) {
            let (k, v) = item.map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            })?;
            let idx = Self::decode_idx(k.as_ref())?;
            if !index_in_range(&range, idx) {
                continue;
            }
            let ent = deserialize(v.as_ref()).map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            })?;
            out.push(ent);
        }
        Ok(out)
    }
}

#[cfg(feature = "rocksdb")]
impl<C: RaftTypeConfig> LogStoreOps<C> for RocksLogStore<C>
where
    C::Entry: serde::Serialize + serde::de::DeserializeOwned + RaftLogId<C::NodeId> + Clone,
{
    fn get_log_state_sync(&mut self) -> Result<LogState<C>, StorageError<C::NodeId>> {
        let purged = self.get_meta::<Option<LogIdOf<C>>>("last_purged_log_id")?;
        let mut last = None;
        for item in self.db.iterator_cf(self.cf_logs()?, IteratorMode::End) {
            let (_k, v) = item.map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            })?;
            let ent: C::Entry = deserialize(v.as_ref()).map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            })?;
            last = Some(ent.get_log_id().clone());
            break;
        }
        let purged = purged.unwrap_or(None);
        Ok(LogState {
            last_purged_log_id: purged.clone(),
            last_log_id: last.or(purged),
        })
    }

    fn save_vote_sync(&mut self, vote: &Vote<C::NodeId>) -> Result<(), StorageError<C::NodeId>> {
        self.put_meta("vote", vote)
    }

    fn read_vote_sync(&mut self) -> Result<Option<Vote<C::NodeId>>, StorageError<C::NodeId>> {
        self.get_meta("vote")
    }

    fn append_to_log_sync<I>(&mut self, entries: I) -> Result<(), StorageError<C::NodeId>>
    where
        I: IntoIterator<Item = C::Entry> + OptionalSend,
    {
        let cf = self.cf_logs()?;
        for entry in entries {
            let key = Self::key(entry.get_log_id().index);
            let val = serialize(&entry).map_err(|e| StorageError::IO {
                source: StorageIOError::write_logs(&e),
            })?;
            self.db.put_cf(cf, key, val).map_err(|e| StorageError::IO {
                source: StorageIOError::write_logs(&e),
            })?;
        }
        Ok(())
    }

    fn delete_conflict_logs_since_sync(
        &mut self,
        log_id: LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        let cf = self.cf_logs()?;
        let keys: Vec<Vec<u8>> = self
            .db
            .iterator_cf(cf, IteratorMode::Start)
            .filter_map(|item| {
                let (k, _) = item.ok()?;
                let idx = Self::decode_idx(k.as_ref()).ok()?;
                (idx >= log_id.index).then(|| k.to_vec())
            })
            .collect();
        for key in keys {
            self.db.delete_cf(cf, key).map_err(|e| StorageError::IO {
                source: StorageIOError::write_logs(&e),
            })?;
        }
        Ok(())
    }

    fn purge_logs_upto_sync(
        &mut self,
        log_id: LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        let cf = self.cf_logs()?;
        let keys: Vec<Vec<u8>> = self
            .db
            .iterator_cf(cf, IteratorMode::Start)
            .filter_map(|item| {
                let (k, _) = item.ok()?;
                let idx = Self::decode_idx(k.as_ref()).ok()?;
                (idx <= log_id.index).then(|| k.to_vec())
            })
            .collect();
        for key in keys {
            self.db.delete_cf(cf, key).map_err(|e| StorageError::IO {
                source: StorageIOError::write_logs(&e),
            })?;
        }
        self.put_meta("last_purged_log_id", &Some(log_id))
    }

    fn save_committed_sync(
        &mut self,
        committed: Option<LogId<C::NodeId>>,
    ) -> Result<(), StorageError<C::NodeId>> {
        self.put_meta("committed", &committed)
    }

    fn read_committed_sync(&mut self) -> Result<Option<LogId<C::NodeId>>, StorageError<C::NodeId>> {
        self.get_meta("committed")
    }
}
