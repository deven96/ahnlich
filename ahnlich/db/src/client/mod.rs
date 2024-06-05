use flurry::HashSet as ConcurrentHashSet;
use std::collections::HashSet as StdHashSet;
use std::net::SocketAddr;
use std::time::SystemTime;
use types::server::ConnectedClient;

#[derive(Debug)]
pub(crate) struct ClientHandler {
    clients: ConcurrentHashSet<ConnectedClient>,
}

impl ClientHandler {
    pub fn new() -> Self {
        Self {
            clients: ConcurrentHashSet::new(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn connect(&self, addr: SocketAddr) -> ConnectedClient {
        let client = ConnectedClient {
            address: format!("{addr}"),
            time_connected: SystemTime::now(),
        };
        let pinned = self.clients.pin();
        pinned.insert(client.clone());
        client
    }

    #[tracing::instrument(skip(self))]
    pub fn disconnect(&self, client: &ConnectedClient) {
        let pinned = self.clients.pin();
        pinned.remove(client);
    }

    #[tracing::instrument(skip(self))]
    pub fn list(&self) -> StdHashSet<ConnectedClient> {
        let pinned = self.clients.pin();
        pinned.into_iter().cloned().collect()
    }
}
