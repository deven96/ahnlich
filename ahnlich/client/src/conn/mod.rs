mod db;
pub use db::Conn;

use crate::error::AhnlichError;
use ahnlich_types::bincode::BinCodeSerAndDeser;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[async_trait::async_trait]
pub(crate) trait Connection
where
    Self::ServerQuery: BinCodeSerAndDeser + Sized,
    Self::ServerResult: BinCodeSerAndDeser + Sized,
{
    type ServerQuery;
    type ServerResult;

    fn stream(&mut self) -> &mut TcpStream;

    async fn is_conn_valid(&mut self) -> Result<(), AhnlichError>;

    async fn send_query(
        &mut self,
        query: Self::ServerQuery,
    ) -> Result<Self::ServerResult, AhnlichError> {
        let serialized_message = query.serialize()?;
        self.stream().write_all(&serialized_message).await?;
        let response: Self::ServerResult = self.deserialize_from_stream().await?;
        Ok(response)
    }

    async fn deserialize_from_stream<T: BinCodeSerAndDeser>(&mut self) -> Result<T, AhnlichError> {
        let mut header = [0u8; ahnlich_types::bincode::RESPONSE_HEADER_LEN];
        self.stream().read_exact(&mut header).await?;
        let mut length_header = [0u8; ahnlich_types::bincode::LENGTH_HEADER_SIZE];
        length_header.copy_from_slice(&header[13..=20]);
        let data_length = u64::from_le_bytes(length_header);
        let mut response = vec![0u8; data_length as usize];
        self.stream().read_exact(&mut response).await?;
        let response = <T as BinCodeSerAndDeser>::deserialize(&response)?;
        Ok(response)
    }
}
