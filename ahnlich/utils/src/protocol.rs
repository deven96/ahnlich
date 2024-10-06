use ahnlich_types::bincode::BinCodeSerAndDeser;
use ahnlich_types::bincode::BinCodeSerAndDeserQuery;
use ahnlich_types::bincode::BinCodeSerAndDeserResponse;
use ahnlich_types::bincode::LENGTH_HEADER_SIZE;
use ahnlich_types::bincode::MAGIC_BYTES;
use ahnlich_types::bincode::VERSION_LENGTH;
use ahnlich_types::client::ConnectedClient;
use ahnlich_types::version::Version;
use ahnlich_types::version::VERSION;
use fallible_collections::vec::FallibleVec;
use std::fmt::Debug;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::Arc;
use tokio::sync::MutexGuard;

use task_manager::TaskState;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::Instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;

#[async_trait::async_trait]
pub trait AhnlichProtocol
where
    Self::ServerQuery: BinCodeSerAndDeserQuery + Debug,
    Self::ServerResponse: BinCodeSerAndDeserResponse + Debug,
{
    type ServerQuery;
    type ServerResponse;

    fn connected_client(&self) -> &ConnectedClient;
    fn maximum_message_size(&self) -> u64;
    fn reader(&self) -> Arc<Mutex<BufReader<TcpStream>>>;

    fn prefix_log(&self, message: impl std::fmt::Display) -> String {
        format!("ClIENT [{}]: {}", &self.connected_client().address, message)
    }

    /// processes messages from a stream
    async fn process(&self) -> TaskState {
        let mut magic_bytes_buf = [0u8; MAGIC_BYTES.len()];
        let mut version_buf = [0u8; VERSION_LENGTH];
        let mut length_buf = [0u8; LENGTH_HEADER_SIZE];
        let reader = self.reader();
        let mut reader = reader.lock().await;
        match reader.read_exact(&mut magic_bytes_buf).await {
            Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                let error = "Hung up on buffered stream";
                return self.handle_error(reader, error, false).await;
            }
            Err(e) => {
                let error = format!("Error reading from task buffered stream {e}");
                return self.handle_error(reader, error, false).await;
            }
            Ok(_) => {
                if magic_bytes_buf != MAGIC_BYTES {
                    let error = "Invalid request stream".to_string();
                    return self.handle_error(reader, error, false).await;
                }
                if let Err(error) = reader.read_exact(&mut version_buf).await {
                    return self.handle_error(reader, error, false).await;
                }
                let version = match Version::deserialize_magic_bytes(&version_buf) {
                    Ok(version) => version,
                    Err(error) => {
                        let error = format!("Unable to parse version chunk {error}");
                        return self.handle_error(reader, error, false).await;
                    }
                };
                if !VERSION.is_compatible(&version) {
                    let error = format!(
                        "Incompatible versions, Server: {:?}, Client {version:?}",
                        *VERSION
                    );
                    return self.handle_error(reader, error, false).await;
                }
                // cap the message size to be of length 1MiB
                if let Err(error) = reader.read_exact(&mut length_buf).await {
                    return self.handle_error(reader, error, false).await;
                };
                let data_length = u64::from_le_bytes(length_buf);
                if data_length > self.maximum_message_size() {
                    let error = format!(
                        "Message cannot exceed {} bytes, configure `message_size` for higher",
                        self.maximum_message_size()
                    );
                    return self.handle_error(reader, error, true).await;
                };

                let mut data: Vec<_> = match FallibleVec::try_with_capacity(data_length as usize) {
                    Err(error) => {
                        return self
                            .handle_error(
                                reader,
                                format!("Could not allocate buffer for message body {:?}", error),
                                true,
                            )
                            .await;
                    }
                    Ok(data) => data,
                };
                if let Err(error) = data.try_resize(data_length as usize, 0u8) {
                    return self
                        .handle_error(
                            reader,
                            format!("Could not resize buffer for message body {:?}", error),
                            true,
                        )
                        .await;
                };
                if let Err(e) = reader.read_exact(&mut data).await {
                    let error = format!("Could not read data buffer {e}");
                    return self.handle_error(reader, error.to_string(), false).await;
                };
                match Self::ServerQuery::deserialize(&data) {
                    Ok(queries) => {
                        log::debug!("Got Queries {:?}", queries);
                        let span = tracing::info_span!("query-processor");
                        if let Some(trace_parent) = queries.get_traceparent() {
                            let parent_context = match tracer::trace_parent_to_span(trace_parent)
                                .map_err(|err| Error::new(ErrorKind::Other, err))
                            {
                                Ok(parent_context) => parent_context,
                                Err(error) => return self.handle_error(reader, error, false).await,
                            };
                            span.set_parent(parent_context);
                        }
                        let results = self.handle(queries.into_inner()).instrument(span).await;
                        if let Ok(binary_results) = results.serialize() {
                            if let Err(error) = reader.get_mut().write_all(&binary_results).await {
                                return self.handle_error(reader, error, false).await;
                            };
                            log::debug!(
                                "Sent Response of length {}, {:?}",
                                binary_results.len(),
                                binary_results
                            );
                        }
                    }
                    Err(error) => {
                        return self.handle_error(reader, error, true).await;
                    }
                }
            }
        }
        TaskState::Continue
    }

    async fn handle_error(
        &self,
        mut reader: MutexGuard<'_, BufReader<TcpStream>>,
        error: impl ToString + Send,
        respond_with_error: bool,
    ) -> TaskState {
        let error = self.prefix_log(error.to_string());
        log::error!("{error}");
        if respond_with_error {
            match Self::ServerResponse::from_error(format!(
                "Could not deserialize query, error is {error}"
            ))
            .serialize()
            {
                Err(e) => log::error!(
                    "{}",
                    self.prefix_log(format!("Could not deserialize error response, {}", e))
                ),
                Ok(deserialize_error) => {
                    if let Err(error) = reader.get_mut().write_all(&deserialize_error).await {
                        log::error!("{}", self.prefix_log(format!("{error}")));
                    }
                }
            };
        }
        TaskState::Break
    }

    async fn handle(
        &self,
        queries: <<Self as AhnlichProtocol>::ServerQuery as BinCodeSerAndDeserQuery>::Inner,
    ) -> Self::ServerResponse;
}
