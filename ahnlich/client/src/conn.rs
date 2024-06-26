use crate::error::AhnlichError;
use std::io::Read;
use std::io::Write;
use std::net::TcpStream;
use types::bincode::BinCodeSerAndDeser;
use types::query::Query;
use types::query::ServerQuery;
use types::server::ServerResponse;
use types::server::ServerResult;

/// Simple TCP Connection to a host and port
#[derive(Debug)]
pub struct Conn {
    stream: TcpStream,
}

impl Conn {
    pub(crate) fn new(host: &str, port: u16) -> Result<Self, AhnlichError> {
        let stream = TcpStream::connect(format!("{host}:{port}"))?;
        Ok(Self { stream })
    }

    pub(crate) fn is_db_conn_broken(&mut self) -> bool {
        self.is_db_conn_valid().is_err()
    }

    pub(crate) fn is_db_conn_valid(&mut self) -> Result<(), AhnlichError> {
        let mut queries = ServerQuery::with_capacity(1);
        queries.push(Query::Ping);
        let response = self.send_db_query(queries)?;
        let mut expected_response = ServerResult::with_capacity(1);
        expected_response.push(Ok(ServerResponse::Pong));
        if response != expected_response {
            return Err(AhnlichError::DbError(format!("{:#?}", response)));
        }
        Ok(())
    }

    pub(crate) fn send_db_query(
        &mut self,
        query: ServerQuery,
    ) -> Result<ServerResult, AhnlichError> {
        let serialized_message = query.serialize()?;
        self.stream.write_all(&serialized_message)?;
        let response: ServerResult = self.deserialize_from_stream()?;
        Ok(response)
    }

    pub(crate) fn deserialize_from_stream<T: BinCodeSerAndDeser>(
        &mut self,
    ) -> Result<T, AhnlichError> {
        let mut header = [0u8; types::bincode::RESPONSE_HEADER_LEN];
        self.stream.read_exact(&mut header)?;
        let mut length_header = [0u8; types::bincode::LENGTH_HEADER_SIZE];
        length_header.copy_from_slice(&header[13..=20]);
        let data_length = u64::from_le_bytes(length_header);
        let mut response = vec![0u8; data_length as usize];
        self.stream.read_exact(&mut response)?;
        let response = <T as BinCodeSerAndDeser>::deserialize(&response)?;
        Ok(response)
    }
}
