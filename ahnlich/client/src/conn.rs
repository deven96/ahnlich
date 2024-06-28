use crate::error::AhnlichError;
use ahnlich_types::bincode::BinCodeSerAndDeser;
use ahnlich_types::query::DBQuery;
use ahnlich_types::query::ServerDBQuery;
use ahnlich_types::server::db::ServerResponse;
use ahnlich_types::server::db::ServerResult;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

/// Simple TCP Connection to a host and port
#[derive(Debug)]
pub struct Conn {
    stream: TcpStream,
}

impl Conn {
    pub(crate) async fn new(host: &str, port: u16) -> Result<Self, AhnlichError> {
        println!("Creating pool with {host}:{port}");
        let stream = TcpStream::connect(format!("{host}:{port}")).await?;
        Ok(Self { stream })
    }

    pub(crate) async fn is_db_conn_valid(&mut self) -> Result<(), AhnlichError> {
        let mut queries = ServerDBQuery::with_capacity(1);
        queries.push(DBQuery::Ping);
        let response = self.send_db_query(queries).await?;
        let mut expected_response = ServerResult::with_capacity(1);
        expected_response.push(Ok(ServerResponse::Pong));
        if response != expected_response {
            return Err(AhnlichError::DbError(format!("{:#?}", response)));
        }
        Ok(())
    }

    pub(crate) async fn send_db_query(
        &mut self,
        query: ServerDBQuery,
    ) -> Result<ServerResult, AhnlichError> {
        let serialized_message = query.serialize()?;
        self.stream.write_all(&serialized_message).await?;
        let response: ServerResult = self.deserialize_from_stream().await?;
        Ok(response)
    }

    pub(crate) async fn deserialize_from_stream<T: BinCodeSerAndDeser>(
        &mut self,
    ) -> Result<T, AhnlichError> {
        let mut header = [0u8; ahnlich_types::bincode::RESPONSE_HEADER_LEN];
        self.stream.read_exact(&mut header).await?;
        let mut length_header = [0u8; ahnlich_types::bincode::LENGTH_HEADER_SIZE];
        length_header.copy_from_slice(&header[13..=20]);
        let data_length = u64::from_le_bytes(length_header);
        let mut response = vec![0u8; data_length as usize];
        self.stream.read_exact(&mut response).await?;
        let response = <T as BinCodeSerAndDeser>::deserialize(&response)?;
        Ok(response)
    }
}
