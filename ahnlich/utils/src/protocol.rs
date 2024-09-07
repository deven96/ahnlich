use ahnlich_types::bincode::BinCodeSerAndDeser;
use ahnlich_types::bincode::BinCodeSerAndDeserQuery;
use ahnlich_types::bincode::BinCodeSerAndDeserResponse;
use ahnlich_types::bincode::LENGTH_HEADER_SIZE;
use ahnlich_types::bincode::MAGIC_BYTES;
use ahnlich_types::bincode::VERSION_LENGTH;
use ahnlich_types::client::ConnectedClient;
use ahnlich_types::version::Version;
use ahnlich_types::version::VERSION;
use std::fmt::Debug;
use std::io::Error;
use std::io::ErrorKind;
use std::sync::Arc;
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
                let error = self.prefix_log("Hung up on buffered stream");
                log::error!("{error}");
                return TaskState::Break;
            }
            Err(e) => {
                let error = self.prefix_log(format!("Error reading from task buffered stream {e}"));
                log::error!("{error}");
                return TaskState::Break;
            }
            Ok(_) => {
                if magic_bytes_buf != MAGIC_BYTES {
                    let error = "Invalid request stream".to_string();
                    log::error!("{error}");
                    return TaskState::Break;
                }
                if let Err(e) = reader.read_exact(&mut version_buf).await {
                    log::error!("{}", e.to_string());
                    return TaskState::Break;
                }
                let version = match Version::deserialize_magic_bytes(&version_buf) {
                    Ok(version) => version,
                    Err(error) => {
                        let error = format!("Unable to parse version chunk {error}");
                        log::error!("{error}");
                        return TaskState::Break;
                    }
                };
                if !VERSION.is_compatible(&version) {
                    let error = format!(
                        "Incompatible versions, Server: {:?}, Client {version:?}",
                        *VERSION
                    );
                    log::error!("{error}");
                    return TaskState::Break;
                }
                // cap the message size to be of length 1MiB
                if let Err(e) = reader.read_exact(&mut length_buf).await {
                    let error = format!("Could not read length buffer {e}");
                    log::error!("{error}");
                    return TaskState::Break;
                };
                let data_length = u64::from_le_bytes(length_buf);
                if data_length > self.maximum_message_size() {
                    let error = self.prefix_log(format!(
                        "Message cannot exceed {} bytes, configure `message_size` for higher",
                        self.maximum_message_size()
                    ));
                    log::error!("{error}");
                    return TaskState::Break;
                }
                let mut data = Vec::new();
                if data.try_reserve(data_length as usize).is_err() {
                    let error = self
                        .prefix_log(format!("failed to reserve buffer of length {data_length}"));
                    log::error!("{error}");
                    return TaskState::Break;
                };
                data.resize(data_length as usize, 0u8);
                if let Err(e) = reader.read_exact(&mut data).await {
                    let error = format!("Could not read data buffer {e}");
                    log::error!("{error}");
                    return TaskState::Break;
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
                                Err(error) => {
                                    log::error!("{error}");
                                    return TaskState::Break;
                                }
                            };
                            span.set_parent(parent_context);
                        }
                        let results = self.handle(queries.into_inner()).instrument(span).await;
                        if let Ok(binary_results) = results.serialize() {
                            if let Err(error) = reader.get_mut().write_all(&binary_results).await {
                                log::error!("{error}");
                                return TaskState::Break;
                            };
                            log::debug!(
                                "Sent Response of length {}, {:?}",
                                binary_results.len(),
                                binary_results
                            );
                        }
                    }
                    Err(error) => {
                        let error = self.prefix_log(&format!(
                            "Could not deserialize client message as server query {error}"
                        ));
                        log::error!("{error}");
                        let deserialize_error = Self::ServerResponse::from_error(
                            "Could not deserialize query, error is {e}".to_string(),
                        )
                        .serialize()
                        .expect("Could not serialize deserialize error");
                        if let Err(error) = reader.get_mut().write_all(&deserialize_error).await {
                            log::error!("{error}");
                            return TaskState::Break;
                        };
                    }
                }
            }
        }
        TaskState::Continue
    }

    async fn handle(
        &self,
        queries: <<Self as AhnlichProtocol>::ServerQuery as BinCodeSerAndDeserQuery>::Inner,
    ) -> Self::ServerResponse;
}
