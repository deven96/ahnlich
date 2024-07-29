use crate::conn::Connection;
use crate::error::AhnlichError;
use ahnlich_types::ai::{AIQuery, AIServerQuery, AIServerResponse, AIServerResult};
use tokio::net::TcpStream;

/// Simple TCP Connection to a host and port
#[derive(Debug)]
pub struct AIConn {
    stream: TcpStream,
}

impl AIConn {
    pub(crate) async fn new(host: &str, port: u16) -> Result<Self, AhnlichError> {
        println!("Creating pool with {host}:{port}");
        let stream = TcpStream::connect(format!("{host}:{port}")).await?;
        Ok(Self { stream })
    }
}

#[async_trait::async_trait]
impl Connection for AIConn {
    type ServerQuery = AIServerQuery;
    type ServerResult = AIServerResult;

    fn stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    async fn is_conn_valid(&mut self) -> Result<(), AhnlichError> {
        let mut queries = Self::ServerQuery::with_capacity(1);
        queries.push(AIQuery::Ping);
        let response = self.send_query(queries).await?;
        let mut expected_response = AIServerResult::with_capacity(1);
        expected_response.push(Ok(AIServerResponse::Pong));
        if response != expected_response {
            return Err(AhnlichError::AIProxyError(format!("{:#?}", response)));
        }
        Ok(())
    }
}
