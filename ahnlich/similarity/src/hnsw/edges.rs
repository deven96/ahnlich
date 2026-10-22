//! Outgoing edge storage shared by the control and compact HNSW implementations.
use super::{NodeId, NodeIdHashSet};
use crate::error::Error;
use arc_swap::ArcSwap;
use papaya::HashSet;
use seize::Collector;
use std::{collections::hash_map::RandomState, fmt::Debug, num::NonZeroUsize, sync::Arc};

/// Immutable outgoing adjacency snapshot, backed by a contiguous vector.
pub type CompactEdges = ArcSwap<Vec<NodeId>>;

#[doc(hidden)]
pub trait EdgeSnapshot {
    fn iter(&self) -> impl Iterator<Item = &NodeId>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[doc(hidden)]
pub trait EdgeStorage: Debug + Send + Sync + 'static {
    type Snapshot<'a>: EdgeSnapshot
    where
        Self: 'a;
    const HASH_COLLECTION: bool;

    fn from_ids(
        ids: Vec<NodeId>,
        collector: Option<&Arc<Collector>>,
        shards: Option<NonZeroUsize>,
    ) -> Self;
    fn snapshot(&self) -> Self::Snapshot<'_>;
    fn insert_edge(&self, id: NodeId);
    fn remove_edge(&self, id: NodeId);

    /// Hash sets return a replacement for the outer map, preserving the control
    /// path. Compact lists publish in place and retry selection after a lost CAS.
    fn prune(
        &self,
        maximum: usize,
        collector: Option<&Arc<Collector>>,
        shards: Option<NonZeroUsize>,
        select: impl FnMut(&[NodeId]) -> Result<Vec<NodeId>, Error>,
    ) -> Result<Option<Self>, Error>
    where
        Self: Sized;
}

impl EdgeSnapshot for papaya::HashSetRef<'_, NodeId, RandomState, seize::LocalGuard<'_>> {
    fn iter(&self) -> impl Iterator<Item = &NodeId> {
        self.iter()
    }
    fn len(&self) -> usize {
        self.len()
    }
}

impl EdgeStorage for HashSet<NodeId> {
    type Snapshot<'a> = papaya::HashSetRef<'a, NodeId, RandomState, seize::LocalGuard<'a>>;
    const HASH_COLLECTION: bool = true;

    fn from_ids(
        ids: Vec<NodeId>,
        collector: Option<&Arc<Collector>>,
        shards: Option<NonZeroUsize>,
    ) -> Self {
        super::index::node_hashset_from_iter(ids, collector, shards)
    }
    fn snapshot(&self) -> Self::Snapshot<'_> {
        self.pin()
    }
    fn insert_edge(&self, id: NodeId) {
        self.pin().insert(id);
    }
    fn remove_edge(&self, id: NodeId) {
        self.pin().remove(&id);
    }
    fn prune(
        &self,
        maximum: usize,
        collector: Option<&Arc<Collector>>,
        shards: Option<NonZeroUsize>,
        mut select: impl FnMut(&[NodeId]) -> Result<Vec<NodeId>, Error>,
    ) -> Result<Option<Self>, Error> {
        let snapshot = self.pin();
        if snapshot.len() <= maximum {
            return Ok(None);
        }
        let ids = snapshot.iter().copied().collect::<Vec<_>>();
        Ok(Some(Self::from_ids(select(&ids)?, collector, shards)))
    }
}

impl EdgeSnapshot for arc_swap::Guard<Arc<Vec<NodeId>>> {
    fn iter(&self) -> impl Iterator<Item = &NodeId> {
        self.as_slice().iter()
    }
    fn len(&self) -> usize {
        self.as_slice().len()
    }
}

impl EdgeStorage for CompactEdges {
    type Snapshot<'a> = arc_swap::Guard<Arc<Vec<NodeId>>>;
    const HASH_COLLECTION: bool = false;

    fn from_ids(mut ids: Vec<NodeId>, _: Option<&Arc<Collector>>, _: Option<NonZeroUsize>) -> Self {
        let mut seen = NodeIdHashSet::default();
        ids.retain(|id| seen.insert(*id));
        Self::from_pointee(ids)
    }
    fn snapshot(&self) -> Self::Snapshot<'_> {
        self.load()
    }
    fn insert_edge(&self, id: NodeId) {
        let mut current = self.load();
        loop {
            if current.contains(&id) {
                return;
            }
            let mut next = Vec::with_capacity(current.len() + 1);
            next.extend_from_slice(&current);
            next.push(id);
            let previous = self.compare_and_swap(&*current, Arc::new(next));
            if Arc::ptr_eq(&current, &previous) {
                return;
            }
            current = previous;
        }
    }
    fn remove_edge(&self, id: NodeId) {
        let mut current = self.load();
        loop {
            let Some(index) = current.iter().position(|edge| *edge == id) else {
                return;
            };
            let mut next = Vec::with_capacity(current.len() - 1);
            next.extend_from_slice(&current[..index]);
            next.extend_from_slice(&current[index + 1..]);
            let previous = self.compare_and_swap(&*current, Arc::new(next));
            if Arc::ptr_eq(&current, &previous) {
                return;
            }
            current = previous;
        }
    }
    fn prune(
        &self,
        maximum: usize,
        _: Option<&Arc<Collector>>,
        _: Option<NonZeroUsize>,
        mut select: impl FnMut(&[NodeId]) -> Result<Vec<NodeId>, Error>,
    ) -> Result<Option<Self>, Error> {
        let mut current = self.load();
        loop {
            if current.len() <= maximum {
                return Ok(None);
            }
            let selected = select(&current)?;
            let previous = self.compare_and_swap(&*current, Arc::new(selected));
            if Arc::ptr_eq(&current, &previous) {
                return Ok(None);
            }
            current = previous;
        }
    }
}
