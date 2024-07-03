use ahnlich_types::ai::{AIQuery, AIServerQuery, AIServerResponse, AIServerResult};
use ahnlich_types::db::{ConnectedClient, ServerInfo};
use ahnlich_types::version::VERSION;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use utils::client::ClientHandler;
use utils::protocol::AhnlichProtocol;

#[derive(Debug)]
pub(super) struct AIProxyTask {
    pub(super) server_addr: SocketAddr,
    pub(super) reader: BufReader<TcpStream>,
    pub(super) client_handler: Arc<ClientHandler>,
    pub(super) connected_client: ConnectedClient,
    pub(super) maximum_message_size: u64,
}
impl AhnlichProtocol for AIProxyTask {
    type ServerQuery = AIServerQuery;
    type ServerResponse = AIServerResult;

    fn connected_client(&self) -> &ConnectedClient {
        &self.connected_client
    }
    fn maximum_message_size(&self) -> u64 {
        self.maximum_message_size
    }
    fn reader(&mut self) -> &mut BufReader<TcpStream> {
        &mut self.reader
    }

    #[tracing::instrument]
    fn handle(&self, queries: Vec<AIQuery>) -> AIServerResult {
        let mut result = AIServerResult::with_capacity(queries.len());
        for query in queries {
            result.push(match query {
                _ => Ok(AIServerResponse::InfoServer(self.server_info())),
            })
        }
        result
    }
}

impl AIProxyTask {
    fn server_info(&self) -> ServerInfo {
        ServerInfo {
            address: format!("{}", self.server_addr),
            version: *VERSION,
            r#type: ahnlich_types::ServerType::AI,
            limit: 20,
            remaining: 29,
        }
    }
}

impl Drop for AIProxyTask {
    fn drop(&mut self) {
        self.client_handler.disconnect(&self.connected_client);
    }
}
