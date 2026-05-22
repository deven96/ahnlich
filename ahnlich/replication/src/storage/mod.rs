#[cfg(any(test, feature = "test-utils"))]
mod memory;
mod rocksdb;
mod state_machine;

use openraft::{LogId, RaftTypeConfig};

#[cfg(any(test, feature = "test-utils"))]
pub use memory::MemLogStore;
pub use rocksdb::RocksLogStore;
pub use state_machine::{SnapshotBuilder, StateMachineHandler, StateMachineStore};

pub type LogIdOf<C> = LogId<<C as RaftTypeConfig>::NodeId>;
