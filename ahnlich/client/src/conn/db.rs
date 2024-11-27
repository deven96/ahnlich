use crate::conn::Connection;
use crate::error::AhnlichError;
use ahnlich_types::db::{DBQuery, ServerDBQuery, ServerResponse, ServerResult};
use tokio::net::TcpStream;

/// Simple TCP Connection to a host and port
#[derive(Debug)]
pub struct DBConn {
    stream: TcpStream,
}

impl DBConn {
    pub(crate) async fn new(host: &str, port: u16) -> Result<Self, AhnlichError> {
        let stream = TcpStream::connect(format!("{host}:{port}")).await?;
        Ok(Self { stream })
    }
}

#[async_trait::async_trait]
impl Connection for DBConn {
    type ServerQuery = ServerDBQuery;
    type ServerResult = ServerResult;

    fn stream(&mut self) -> &mut TcpStream {
        &mut self.stream
    }

    async fn is_conn_valid(&mut self) -> Result<(), AhnlichError> {
        let mut queries = Self::ServerQuery::with_capacity(1)?;
        queries.push(DBQuery::Ping);
        let response = self.send_query(queries).await?;
        let mut expected_response = ServerResult::with_capacity(1);
        expected_response.push(Ok(ServerResponse::Pong));
        if response != expected_response {
            return Err(AhnlichError::DbError(format!("{:#?}", response)));
        }
        Ok(())
    }
}
