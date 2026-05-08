// `clippy::result_large_err` fires on helper signatures that intentionally use
// openraft's upstream `StorageError` type. Boxing or wrapping the error here
// would only obscure the storage-layer contract we already use throughout this
// module.
#![allow(clippy::result_large_err)]

use std::fmt::Debug;
use std::ops::{Bound, RangeBounds};
use std::path::Path;
use std::sync::Arc;

use openraft::{
    LogId, LogState, OptionalSend, RaftLogId, RaftLogReader, RaftTypeConfig, StorageError,
    StorageIOError, Vote,
};
use rocksdb::{ColumnFamilyDescriptor, DB, IteratorMode, Options};
use serde_json::{from_slice, to_vec};

use super::{LogIdOf, LogStoreOps};

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

// RocksDB-backed log store. Production default. Persists Raft log entries and
// metadata (vote, committed index, last_purged_log_id) across restarts.

#[derive(Debug, Clone)]
pub struct RocksLogStore<C: RaftTypeConfig> {
    db: Arc<DB>,
    _p: std::marker::PhantomData<C>,
}

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

    fn collect_log_keys_matching<F>(
        &self,
        mut predicate: F,
    ) -> Result<Vec<Vec<u8>>, StorageError<C::NodeId>>
    where
        F: FnMut(u64) -> bool,
    {
        let mut keys = Vec::new();
        for item in self.db.iterator_cf(self.cf_logs()?, IteratorMode::Start) {
            let (key, _) = item.map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            })?;
            let idx = Self::decode_idx(key.as_ref())?;
            if predicate(idx) {
                keys.push(key.to_vec());
            }
        }
        Ok(keys)
    }

    fn put_meta<T: serde::Serialize>(
        &self,
        key: &str,
        value: &T,
    ) -> Result<(), StorageError<C::NodeId>> {
        let bytes = to_vec(value).map_err(|e| StorageError::IO {
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
            Some(bytes) => from_slice(&bytes).map(Some).map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            }),
            None => Ok(None),
        }
    }
}

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
            let ent = from_slice(v.as_ref()).map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            })?;
            out.push(ent);
        }
        Ok(out)
    }
}

impl<C: RaftTypeConfig> LogStoreOps<C> for RocksLogStore<C>
where
    C::Entry: serde::Serialize + serde::de::DeserializeOwned + RaftLogId<C::NodeId> + Clone,
{
    fn get_log_state_sync(&mut self) -> Result<LogState<C>, StorageError<C::NodeId>> {
        let purged = self.get_meta::<Option<LogIdOf<C>>>("last_purged_log_id")?;
        let last = if let Some(item) = self
            .db
            .iterator_cf(self.cf_logs()?, IteratorMode::End)
            .next()
        {
            let (_k, v) = item.map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            })?;
            let ent: C::Entry = from_slice(v.as_ref()).map_err(|e| StorageError::IO {
                source: StorageIOError::read_logs(&e),
            })?;
            Some(ent.get_log_id().clone())
        } else {
            None
        };
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
            let val = to_vec(&entry).map_err(|e| StorageError::IO {
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
        let keys = self.collect_log_keys_matching(|idx| idx >= log_id.index)?;
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
        let keys = self.collect_log_keys_matching(|idx| idx <= log_id.index)?;
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use openraft::{CommittedLeaderId, Entry, EntryPayload, Vote};

    use super::*;
    use crate::node::ReplicationNode;

    openraft::declare_raft_types!(
        pub TestConfig:
            D = String,
            R = String,
            Node = ReplicationNode,
            SnapshotData = Cursor<Vec<u8>>
    );

    fn log_id(term: u64, node_id: u64, index: u64) -> LogId<u64> {
        LogId::new(CommittedLeaderId::new(term, node_id), index)
    }

    fn normal_entry(term: u64, node_id: u64, index: u64, data: &str) -> Entry<TestConfig> {
        Entry {
            log_id: log_id(term, node_id, index),
            payload: EntryPayload::Normal(data.to_owned()),
        }
    }

    #[test]
    fn rocks_log_store_round_trips_logs_and_meta() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut store =
            RocksLogStore::<TestConfig>::open(dir.path()).expect("open rocks log store");

        let entries = vec![
            normal_entry(1, 1, 1, "alpha"),
            normal_entry(1, 1, 2, "beta"),
            normal_entry(1, 1, 3, "gamma"),
        ];
        store
            .append_to_log_sync(entries.clone())
            .expect("append to log should succeed");

        let vote = Vote::new(7, 2);
        store
            .save_vote_sync(&vote)
            .expect("save vote should succeed");
        store
            .save_committed_sync(Some(log_id(1, 1, 2)))
            .expect("save committed should succeed");

        let mut reader = store.clone();
        let round_tripped = futures::executor::block_on(reader.try_get_log_entries(1..=3))
            .expect("read logs should succeed");
        assert_eq!(round_tripped, entries);
        assert_eq!(
            store.read_vote_sync().expect("read vote should succeed"),
            Some(vote)
        );
        assert_eq!(
            store
                .read_committed_sync()
                .expect("read committed should succeed"),
            Some(log_id(1, 1, 2))
        );

        let state = store
            .get_log_state_sync()
            .expect("log state should succeed");
        assert_eq!(state.last_purged_log_id, None);
        assert_eq!(state.last_log_id, Some(log_id(1, 1, 3)));
    }

    #[test]
    fn rocks_log_store_purge_and_conflict_delete_update_state() {
        let dir = tempfile::tempdir().expect("tempdir");
        let mut store =
            RocksLogStore::<TestConfig>::open(dir.path()).expect("open rocks log store");

        let entries = vec![
            normal_entry(1, 1, 1, "one"),
            normal_entry(1, 1, 2, "two"),
            normal_entry(1, 1, 3, "three"),
            normal_entry(1, 1, 4, "four"),
            normal_entry(1, 1, 5, "five"),
        ];
        store
            .append_to_log_sync(entries)
            .expect("append to log should succeed");

        store
            .delete_conflict_logs_since_sync(log_id(1, 1, 4))
            .expect("delete conflict logs should succeed");
        let mut reader = store.clone();
        let after_conflict = futures::executor::block_on(reader.try_get_log_entries(1..=10))
            .expect("read after conflict delete should succeed");
        assert_eq!(
            after_conflict
                .into_iter()
                .map(|entry| entry.get_log_id().index)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );

        store
            .purge_logs_upto_sync(log_id(1, 1, 2))
            .expect("purge logs should succeed");
        let mut reader = store.clone();
        let after_purge = futures::executor::block_on(reader.try_get_log_entries(1..=10))
            .expect("read after purge should succeed");
        assert_eq!(
            after_purge
                .into_iter()
                .map(|entry| entry.get_log_id().index)
                .collect::<Vec<_>>(),
            vec![3]
        );

        let state = store
            .get_log_state_sync()
            .expect("log state should succeed");
        assert_eq!(state.last_purged_log_id, Some(log_id(1, 1, 2)));
        assert_eq!(state.last_log_id, Some(log_id(1, 1, 3)));
    }
}
