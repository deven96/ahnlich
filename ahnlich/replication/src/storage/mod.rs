#[cfg(any(test, feature = "test-utils"))]
mod memory;
mod rocksdb;
mod state_machine;

use openraft::{LogId, RaftTypeConfig};

#[cfg(any(test, feature = "test-utils"))]
pub use memory::MemLogStore;
pub use rocksdb::RocksLogStore;
pub use state_machine::{
    MemorySnapshotStore, PersistedSnapshot, ReplicationFailureState, SnapshotBuilder,
    StateMachineHandler, StateMachineSnapshotStore, StateMachineStore,
};

pub type LogIdOf<C> = LogId<<C as RaftTypeConfig>::NodeId>;
