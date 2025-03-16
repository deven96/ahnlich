use flurry::HashSet as ConcurrentHashSet;
use grpc_types::client::ConnectedClient;
use std::collections::HashSet as StdHashSet;
use std::net::SocketAddr;
use std::time::SystemTime;

/// Datastructure to keep track of clients that have connected to a server while allowing limiting
/// the maximum number
#[derive(Debug)]
pub struct ClientHandler {
    clients: ConcurrentHashSet<ConnectedClient>,
    maximum_clients: usize,
}

impl ClientHandler {
    pub fn new(maximum_clients: usize) -> Self {
        Self {
            clients: ConcurrentHashSet::with_capacity(maximum_clients),
            maximum_clients,
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn connect(&self, addr: SocketAddr) -> Option<ConnectedClient> {
        let pinned = self.clients.pin();
        log::debug!("Current client len {}", pinned.len());
        if self.is_maxed_out() {
            log::error!(
                "Maximum clients count {} reached or exceeded with {}",
                pinned.len(),
                self.maximum_clients
            );
            return None;
        };
        let client = ConnectedClient {
            address: format!("{addr}"),
            time_connected: format!("{:?}", SystemTime::now()),
        };
        pinned.insert(client.clone());
        Some(client)
    }

    #[tracing::instrument(skip(self))]
    pub fn disconnect(&self, client: &ConnectedClient) {
        let pinned = self.clients.pin();
        pinned.remove(client);
    }

    #[tracing::instrument(skip(self))]
    pub fn is_maxed_out(&self) -> bool {
        let pinned = self.clients.pin();
        if pinned.len() >= self.maximum_clients {
            return true;
        }
        false
    }

    #[tracing::instrument(skip(self))]
    pub fn list(&self) -> StdHashSet<ConnectedClient> {
        let pinned = self.clients.pin();
        pinned.into_iter().cloned().collect()
    }
}
