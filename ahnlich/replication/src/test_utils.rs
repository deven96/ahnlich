//! Test scaffolding shared across the replication, DB, and AI test suites.
//!
//! Milestone 1 ships the module skeleton only. Milestone 2 fills in:
//!
//! * `spawn_clustered_node`: launch a server process bound to OS-assigned
//!   ports for `--port`, `--cluster-addr`, and the admin port.
//! * `bootstrap_three_node_cluster`: spin up a 3-node cluster, return the
//!   handles, and tear down on drop.
//! * `wait_for_leader`: poll cluster metrics until a leader emerges or the
//!   deadline expires.
//!
//! The point is one source of truth for cluster test plumbing

use std::net::{SocketAddr, TcpListener};

/// Bind to `127.0.0.1:0`, drop the listener, and return the kernel-assigned
/// port. Test harnesses rely on this to avoid port collisions when many
/// cluster nodes run in the same process tree.
///
/// # Panics
/// Panics if the loopback interface cannot bind. This should only happen in
/// pathological CI environments, failing fast is better than ignoring it.
pub fn os_assigned_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind 127.0.0.1:0 for OS-assigned port");
    listener
        .local_addr()
        .expect("read local_addr after bind")
        .port()
}

/// Convenience wrapper over [`os_assigned_port`] returning a fully-formed
/// loopback `SocketAddr` so callers can pass it straight to the server CLI.
pub fn os_assigned_loopback() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], os_assigned_port()))
}
