//! Per-node mutation permission. No graph-wide lock and no search registration.
use parking_lot::{Condvar, Mutex};
use std::sync::Arc;

#[derive(Debug, Default)]
struct State {
    closing: bool,
    active: usize,
}

#[derive(Debug, Default)]
pub(super) struct NodeGate {
    state: Mutex<State>,
    drained: Condvar,
}

/// Keeps a node logically present while a mutation can publish references to it.
/// The mutex is held only when entering/leaving, never while touching the graph.
#[must_use]
pub(super) struct NodeLease(Arc<NodeGate>);

impl NodeGate {
    pub(super) fn try_enter(self: &Arc<Self>) -> Option<NodeLease> {
        let mut state = self.state.lock();
        if state.closing {
            return None;
        }
        state.active += 1;
        Some(NodeLease(Arc::clone(self)))
    }

    /// Claims cleanup, stops new users, and waits for existing users to finish.
    /// Callers must release their own leases first. Condvar::wait releases the
    /// mutex, so another node's mutation can enter/leave without lock-order cycles.
    pub(super) fn close_and_wait(&self) -> bool {
        let mut state = self.state.lock();
        if state.closing {
            return false;
        }
        state.closing = true;
        while state.active != 0 {
            self.drained.wait(&mut state);
        }
        true
    }

    #[cfg(test)]
    pub(super) fn is_closing(&self) -> bool {
        self.state.lock().closing
    }
}

impl NodeLease {
    pub(super) fn protects(&self, gate: &Arc<NodeGate>) -> bool {
        Arc::ptr_eq(&self.0, gate)
    }
}

impl Drop for NodeLease {
    fn drop(&mut self) {
        let mut state = self.0.state.lock();
        state.active -= 1;
        if state.active == 0 {
            self.0.drained.notify_all();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::{Duration, Instant};

    #[test]
    fn closing_rejects_new_users_and_waits_for_existing_users() {
        let gate = Arc::new(NodeGate::default());
        let first = gate.try_enter().unwrap();
        let second = gate.try_enter().unwrap();
        std::thread::scope(|scope| {
            let (done_tx, done_rx) = mpsc::channel();
            let gate = &gate;
            scope.spawn(move || done_tx.send(gate.close_and_wait()).unwrap());
            let deadline = Instant::now() + Duration::from_secs(5);
            while !gate.is_closing() {
                assert!(Instant::now() < deadline);
                std::thread::yield_now();
            }
            assert!(gate.try_enter().is_none());
            drop(first);
            assert!(matches!(done_rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
            drop(second);
            assert!(done_rx.recv_timeout(Duration::from_secs(5)).unwrap());
        });
        assert!(!gate.close_and_wait());
        assert!(gate.try_enter().is_none());
    }
}
