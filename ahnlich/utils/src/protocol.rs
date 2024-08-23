use ahnlich_types::bincode::BinCodeSerAndDeser;
use ahnlich_types::bincode::BinCodeSerAndDeserQuery;
use ahnlich_types::bincode::BinCodeSerAndDeserResponse;
use ahnlich_types::bincode::LENGTH_HEADER_SIZE;
use ahnlich_types::bincode::MAGIC_BYTES;
use ahnlich_types::bincode::VERSION_LENGTH;
use ahnlich_types::db::ConnectedClient;
use ahnlich_types::version::Version;
use ahnlich_types::version::VERSION;
use std::fmt::Debug;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Result as IoResult;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::select;
use tokio_graceful::ShutdownGuard;
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[async_trait::async_trait]
pub trait AhnlichProtocol
where
    Self::ServerQuery: BinCodeSerAndDeserQuery + Sized + Debug,
    Self::ServerResponse: BinCodeSerAndDeserResponse + Sized + Debug,
{
    type ServerQuery;
    type ServerResponse;

    fn connected_client(&self) -> &ConnectedClient;
    fn maximum_message_size(&self) -> u64;
    fn reader(&mut self) -> &mut BufReader<TcpStream>;

    fn prefix_log(&self, message: impl std::fmt::Display) -> String {
        format!("ClIENT [{}]: {}", &self.connected_client().address, message)
    }

    /// processes messages from a stream
    async fn process(&mut self, shutdown_guard: ShutdownGuard) -> IoResult<()> {
        let mut magic_bytes_buf = [0u8; MAGIC_BYTES.len()];
        let mut version_buf = [0u8; VERSION_LENGTH];
        let mut length_buf = [0u8; LENGTH_HEADER_SIZE];

        loop {
            select! {
                _ = shutdown_guard.cancelled() => {
                    tracing::debug!("{}", self.prefix_log("Cancelling stream as server is shutting down"));
                    break;
                }
                res = self.reader().read_exact(&mut magic_bytes_buf) => {
                    match res {
                        // reader was closed
                        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                            tracing::debug!("{}", self.prefix_log("Hung up on buffered stream"));
                            break;
                        }
                        Err(e) => {
                            tracing::error!("{}", self.prefix_log(format!("Error reading from task buffered stream {e}")));
                        }
                        Ok(_) => {
                            if magic_bytes_buf != MAGIC_BYTES {
                                return Err(Error::other("Invalid request stream"));
                            }
                            self.reader().read_exact(&mut version_buf).await?;
                            let version = Version::deserialize_magic_bytes(&version_buf).map_err(|a| Error::other(format!("Unable to parse version chunk {a}")))?;
                            if !VERSION.is_compatible(&version) {
                                return Err(Error::other(format!("Incompatible versions, Server: {:?}, Client {version:?}", *VERSION)));
                            }
                            // cap the message size to be of length 1MiB
                            self.reader().read_exact(&mut length_buf).await?;
                            let data_length = u64::from_le_bytes(length_buf);
                            if data_length > self.maximum_message_size() {
                                tracing::error!("{}", self.prefix_log(format!("Message cannot exceed {} bytes, configure `message_size` for higher", self.maximum_message_size())));
                                break
                            }
                            let mut data = Vec::new();
                            if data.try_reserve(data_length as usize).is_err() {
                                tracing::error!("{}", self.prefix_log(format!("Failed to reserve buffer of length {data_length}")));
                                break
                            };
                            data.resize(data_length as usize, 0u8);
                            self.reader().read_exact(&mut data).await?;
                            match Self::ServerQuery::deserialize(&data) {
                                Ok(queries) => {
                                tracing::debug!("Got Queries {:?}", queries);
                                let span = tracing::info_span!("query-processor");
                                if let Some(trace_parent) = queries.get_traceparent() {
                                    let parent_context = tracer::trace_parent_to_span(trace_parent).map_err(|err|Error::new(ErrorKind::Other, err))?;
                                    span.set_parent(parent_context);
                                }
                                let results = self.handle(queries.into_inner()).instrument(span).await;
                                if let Ok(binary_results) = results.serialize() {
                                    self.reader().get_mut().write_all(&binary_results).await?;
                                    tracing::debug!("Sent Response of length {}, {:?}", binary_results.len(), binary_results);
                                }
                            },
                            Err(e) =>{
                                tracing::error!("{} {e}", self.prefix_log("Could not deserialize client message as server query"));
                                let deserialize_error = Self::ServerResponse::from_error("Could not deserialize query, error is {e}".to_string()).serialize().expect("Could not serialize deserialize error");
                                self.reader().get_mut().write_all(&deserialize_error).await?;
                            }
                        }
                    }
                    }
                }
            }
        }
        Ok(())
    }

    async fn handle(
        &self,
        queries: <<Self as AhnlichProtocol>::ServerQuery as BinCodeSerAndDeserQuery>::Inner,
    ) -> Self::ServerResponse;
}
