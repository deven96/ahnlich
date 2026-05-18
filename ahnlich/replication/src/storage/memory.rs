use std::collections::BTreeMap;
use std::fmt::Debug;
use std::ops::RangeBounds;
use std::sync::{Arc, Mutex};

use openraft::{
    LogState, OptionalSend, RaftLogId, RaftLogReader, RaftTypeConfig, StorageError, Vote,
};

use super::{LogIdOf, LogStoreOps};

// In-memory log store. Gated by the `test-utils` feature (or `cfg(test)`); not
// for production. A node restart loses all Raft state, which defeats the
// purpose of replication. The presence of two log-store implementations also
// validates the `LogStoreOps` trait shape.

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
    fn lock_inner(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, MemLogStoreInner<C>>, StorageError<C::NodeId>> {
        self.inner.lock().map_err(|_| StorageError::IO {
            source: openraft::StorageIOError::read_logs(&std::io::Error::other(
                "mem log lock poisoned",
            )),
        })
    }

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
        let inner = self.lock_inner()?;
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
        let inner = self.lock_inner()?;
        Ok(LogState {
            last_purged_log_id: inner.last_purged_log_id.clone(),
            last_log_id: Self::last_log_id(&inner),
        })
    }

    fn save_vote_sync(&mut self, vote: &Vote<C::NodeId>) -> Result<(), StorageError<C::NodeId>> {
        let mut inner = self.lock_inner()?;
        inner.vote = Some(vote.clone());
        Ok(())
    }

    fn read_vote_sync(&mut self) -> Result<Option<Vote<C::NodeId>>, StorageError<C::NodeId>> {
        let inner = self.lock_inner()?;
        Ok(inner.vote.clone())
    }

    fn append_to_log_sync<I>(&mut self, entries: I) -> Result<(), StorageError<C::NodeId>>
    where
        I: IntoIterator<Item = C::Entry> + OptionalSend,
    {
        let mut inner = self.lock_inner()?;
        for entry in entries {
            inner.log.insert(entry.get_log_id().index, entry);
        }
        Ok(())
    }

    fn delete_conflict_logs_since_sync(
        &mut self,
        log_id: openraft::LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        let mut inner = self.lock_inner()?;
        let keys: Vec<u64> = inner.log.range(log_id.index..).map(|(k, _)| *k).collect();
        for k in keys {
            inner.log.remove(&k);
        }
        Ok(())
    }

    fn purge_logs_upto_sync(
        &mut self,
        log_id: openraft::LogId<C::NodeId>,
    ) -> Result<(), StorageError<C::NodeId>> {
        let mut inner = self.lock_inner()?;
        let keys: Vec<u64> = inner.log.range(..=log_id.index).map(|(k, _)| *k).collect();
        for k in keys {
            inner.log.remove(&k);
        }
        inner.last_purged_log_id = Some(log_id);
        Ok(())
    }

    fn save_committed_sync(
        &mut self,
        committed: Option<openraft::LogId<C::NodeId>>,
    ) -> Result<(), StorageError<C::NodeId>> {
        let mut inner = self.lock_inner()?;
        inner.committed = committed;
        Ok(())
    }

    fn read_committed_sync(
        &mut self,
    ) -> Result<Option<openraft::LogId<C::NodeId>>, StorageError<C::NodeId>> {
        let inner = self.lock_inner()?;
        Ok(inner.committed.clone())
    }
}
