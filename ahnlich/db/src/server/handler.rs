use super::task::ServerTask;
use crate::cli::ServerConfig;
use crate::engine::store::StoreHandler;
use grpc_types::algorithm::nonlinear::NonLinearAlgorithm;
use grpc_types::client::ConnectedClient;
use grpc_types::keyval::{StoreEntry, StoreKey, StoreName};
// use ahnlich_types::metadata::MetadataKey;
use grpc_types::db::pipeline::db_query::Query;
use grpc_types::db::server::GetSimNEntry;
use grpc_types::services::db_service::db_service_server::{DbService, DbServiceServer};
use grpc_types::shared::info::ErrorResponse;

use grpc_types::db::{pipeline, query, server};
use grpc_types::{client as grpc_types_client, utils as grpc_utils};
use std::future::Future;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use task_manager::TaskManager;
use task_manager::TaskState;
use task_manager::{BlockingTask, Task};
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use utils::allocator::GLOBAL_ALLOCATOR;
use utils::connection_layer::{trace_with_parent, RequestTrackerLayer};
use utils::server::ServerUtilsConfig;
use utils::server::{AhnlichServerUtils, ListenerStreamOrAddress};
use utils::{client::ClientHandler, persistence::Persistence};

const SERVICE_NAME: &str = "ahnlich-db";

#[derive(Debug, Clone)]
pub struct Server {
    listener: ListenerStreamOrAddress,
    store_handler: Arc<StoreHandler>,
    client_handler: Arc<ClientHandler>,
    task_manager: Arc<TaskManager>,
    config: ServerConfig,
}

// maximum_message_size => DbServiceServer(server).max_decoding_message_size
// maximum_clients => At this point yet to figure out but it might be manually implementing
// Server/Interceptor as shown in https://chatgpt.com/share/67abdf0b-72a8-8008-b203-bc8e65b02495
// maximum_concurrency_per_client => we just set this with `concurrency_limit_per_connection`.
// for creating trace functions, we can add `trace_fn` and extract our header from `Request::header` and return the span
#[tonic::async_trait]
impl DbService for Server {
    #[tracing::instrument(skip_all)]
    async fn create_store(
        &self,
        request: tonic::Request<query::CreateStore>,
    ) -> std::result::Result<tonic::Response<server::Unit>, tonic::Status> {
        let create_store_params = request.into_inner();
        let dimensions = match NonZeroUsize::new(create_store_params.dimension as usize) {
            Some(val) => val,
            None => {
                return Err(tonic::Status::invalid_argument(
                    "dimension must be greater than 0",
                ))
            }
        };
        let non_linear_indices = create_store_params
            .non_linear_indices
            .into_iter()
            .filter_map(|index| {
                NonLinearAlgorithm::try_from(index).map_or(None, |converted| Some(converted))
            })
            .collect();

        self.store_handler
            .create_store(
                StoreName {
                    value: create_store_params.store,
                },
                dimensions,
                create_store_params.create_predicates.into_iter().collect(),
                non_linear_indices,
                create_store_params.error_if_exists,
            )
            .map(|_| tonic::Response::new(server::Unit {}))
            .map_err(|err| err.into())
    }

    #[tracing::instrument(skip_all)]
    async fn get_key(
        &self,
        request: tonic::Request<query::GetKey>,
    ) -> std::result::Result<tonic::Response<server::Get>, tonic::Status> {
        let params = request.into_inner();
        let keys = params
            .keys
            .into_iter()
            .map(|key| StoreKey { key: key.key })
            .collect();

        let entries: Vec<StoreEntry> = self
            .store_handler
            .get_key_in_store(
                &StoreName {
                    value: params.store,
                },
                keys,
            )?
            .into_iter()
            .map(|(store_key, store_value)| StoreEntry {
                key: Some(store_key),
                value: Some(store_value),
            })
            .collect();

        Ok(tonic::Response::new(server::Get { entries }))
    }

    #[tracing::instrument(skip_all)]
    async fn get_pred(
        &self,
        request: tonic::Request<query::GetPred>,
    ) -> std::result::Result<tonic::Response<server::Get>, tonic::Status> {
        let params = request.into_inner();

        let condition =
            grpc_types::unwrap_or_invalid!(params.condition, "Predicate Condition is required");

        let entries = self
            .store_handler
            .get_pred_in_store(
                &StoreName {
                    value: params.store,
                },
                &condition,
            )?
            .into_iter()
            .map(|(store_key, store_value)| StoreEntry {
                key: Some(store_key),
                value: Some(store_value),
            })
            .collect();

        Ok(tonic::Response::new(server::Get { entries }))
    }

    #[tracing::instrument(skip_all)]
    async fn get_sim_n(
        &self,
        request: tonic::Request<query::GetSimN>,
    ) -> std::result::Result<tonic::Response<server::GetSimN>, tonic::Status> {
        let params = request.into_inner();
        let search_input =
            grpc_types::unwrap_or_invalid!(params.search_input, "search input is required");

        let search_input = StoreKey {
            key: search_input.key,
        };

        let algorithm = grpc_types::algorithm::algorithms::Algorithm::try_from(params.algorithm)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?
            .into();

        let entries = self
            .store_handler
            .get_sim_in_store(
                &StoreName {
                    value: params.store,
                },
                search_input,
                grpc_utils::convert_to_nonzerousize(params.closest_n)?,
                algorithm,
                params.condition,
            )?
            .into_iter()
            .map(|(store_key, store_value, sim)| GetSimNEntry {
                key: Some(store_key),
                value: Some(store_value),
                similarity: Some(sim),
            })
            .collect();

        Ok(tonic::Response::new(server::GetSimN { entries }))
    }

    #[tracing::instrument(skip_all)]
    async fn ping(
        &self,
        _request: tonic::Request<query::Ping>,
    ) -> std::result::Result<tonic::Response<server::Pong>, tonic::Status> {
        Ok(tonic::Response::new(server::Pong {}))
    }

    #[tracing::instrument(skip_all)]
    async fn create_pred_index(
        &self,
        request: tonic::Request<query::CreatePredIndex>,
    ) -> std::result::Result<tonic::Response<server::CreateIndex>, tonic::Status> {
        let params = request.into_inner();

        let predicates = params
            .predicates
            .into_iter()
            // .map(MetadataKey::new)
            .collect();

        let created = self.store_handler.create_pred_index(
            &StoreName {
                value: params.store,
            },
            predicates,
        )?;

        Ok(tonic::Response::new(server::CreateIndex {
            created_indexes: created as u64,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn create_non_linear_algorithm_index(
        &self,
        request: tonic::Request<query::CreateNonLinearAlgorithmIndex>,
    ) -> std::result::Result<tonic::Response<server::CreateIndex>, tonic::Status> {
        let params = request.into_inner();

        let non_linear_indices = params
            .non_linear_indices
            .into_iter()
            .filter_map(|val| {
                grpc_types::algorithm::nonlinear::NonLinearAlgorithm::try_from(val).ok()
            })
            .map(|val| val.into())
            .collect();

        let created = self.store_handler.create_non_linear_algorithm_index(
            &StoreName {
                value: params.store,
            },
            non_linear_indices,
        )?;

        Ok(tonic::Response::new(server::CreateIndex {
            created_indexes: created as u64,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn drop_pred_index(
        &self,
        request: tonic::Request<query::DropPredIndex>,
    ) -> std::result::Result<tonic::Response<server::Del>, tonic::Status> {
        let params = request.into_inner();

        let predicates = params
            .predicates
            .into_iter()
            // .map(MetadataKey::new)
            .collect();

        let del = self.store_handler.drop_pred_index_in_store(
            &StoreName {
                value: params.store,
            },
            predicates,
            params.error_if_not_exists,
        )?;

        Ok(tonic::Response::new(server::Del {
            deleted_count: del as u64,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn drop_non_linear_algorithm_index(
        &self,
        request: tonic::Request<query::DropNonLinearAlgorithmIndex>,
    ) -> std::result::Result<tonic::Response<server::Del>, tonic::Status> {
        let params = request.into_inner();

        let non_linear_indices = params
            .non_linear_indices
            .into_iter()
            .filter_map(|val| {
                grpc_types::algorithm::nonlinear::NonLinearAlgorithm::try_from(val).ok()
            })
            .map(|val| val.into())
            .collect();

        let del = self.store_handler.drop_non_linear_algorithm_index(
            &StoreName {
                value: params.store,
            },
            non_linear_indices,
            params.error_if_not_exists,
        )?;

        Ok(tonic::Response::new(server::Del {
            deleted_count: del as u64,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn del_key(
        &self,
        request: tonic::Request<query::DelKey>,
    ) -> std::result::Result<tonic::Response<server::Del>, tonic::Status> {
        let params = request.into_inner();

        let keys = params
            .keys
            .into_iter()
            .map(|key| StoreKey { key: key.key })
            .collect();
        let del = self.store_handler.del_key_in_store(
            &StoreName {
                value: params.store,
            },
            keys,
        )?;

        Ok(tonic::Response::new(server::Del {
            deleted_count: del as u64,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn del_pred(
        &self,
        request: tonic::Request<query::DelPred>,
    ) -> std::result::Result<tonic::Response<server::Del>, tonic::Status> {
        let params = request.into_inner();

        let condition =
            grpc_types::unwrap_or_invalid!(params.condition, "Predicate Condition is required");

        let del = self.store_handler.del_pred_in_store(
            &StoreName {
                value: params.store,
            },
            &condition,
        )?;

        Ok(tonic::Response::new(server::Del {
            deleted_count: del as u64,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn drop_store(
        &self,
        request: tonic::Request<query::DropStore>,
    ) -> std::result::Result<tonic::Response<server::Del>, tonic::Status> {
        let drop_store_params = request.into_inner();
        let dropped = self.store_handler.drop_store(
            StoreName {
                value: drop_store_params.store,
            },
            drop_store_params.error_if_not_exists,
        )?;

        Ok(tonic::Response::new(server::Del {
            deleted_count: dropped as u64,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn list_clients(
        &self,
        _request: tonic::Request<query::ListClients>,
    ) -> std::result::Result<tonic::Response<server::ClientList>, tonic::Status> {
        let clients = self
            .client_handler
            .list()
            .into_iter()
            .map(|client| grpc_types_client::ConnectedClient {
                address: client.address,
                time_connected: format!("{:?}", client.time_connected),
            })
            .collect();

        Ok(tonic::Response::new(server::ClientList { clients }))
    }

    #[tracing::instrument(skip_all)]
    async fn list_stores(
        &self,
        _request: tonic::Request<query::ListStores>,
    ) -> std::result::Result<tonic::Response<server::StoreList>, tonic::Status> {
        let stores = self
            .store_handler
            .list_stores()
            .into_iter()
            .map(|store| server::StoreInfo {
                name: store.name.to_string(),
                len: store.len as u64,
                size_in_bytes: store.size_in_bytes as u64,
            })
            .collect();

        Ok(tonic::Response::new(server::StoreList { stores }))
    }

    #[tracing::instrument(skip_all)]
    async fn info_server(
        &self,
        _request: tonic::Request<query::InfoServer>,
    ) -> std::result::Result<tonic::Response<server::InfoServer>, tonic::Status> {
        let version = env!("CARGO_PKG_VERSION").to_string();
        let server_info = grpc_types::shared::info::ServerInfo {
            address: format!("{:?}", self.listener.local_addr()),
            version,
            r#type: grpc_types::server_types::ServerType::Database.into(),
            limit: GLOBAL_ALLOCATOR.limit() as u64,
            remaining: GLOBAL_ALLOCATOR.remaining() as u64,
        };

        let info_server = server::InfoServer {
            info: Some(server_info),
        };

        Ok(tonic::Response::new(info_server))
    }

    #[tracing::instrument(skip_all)]
    async fn set(
        &self,
        _request: tonic::Request<query::Set>,
    ) -> std::result::Result<tonic::Response<server::Set>, tonic::Status> {
        todo!()
    }

    #[tracing::instrument(skip_all)]
    async fn pipeline(
        &self,
        request: tonic::Request<pipeline::DbRequestPipeline>,
    ) -> std::result::Result<tonic::Response<pipeline::DbResponsePipeline>, tonic::Status> {
        let params = request.into_inner();
        let mut response_vec = Vec::with_capacity(params.queries.len());

        for pipeline_query in params.queries {
            let pipeline_query =
                grpc_types::unwrap_or_invalid!(pipeline_query.query, "query is required");

            match pipeline_query {
                Query::Ping(params) => match self.ping(tonic::Request::new(params)).await {
                    Ok(res) => response_vec.push(pipeline::db_server_response::Response::Pong(
                        res.into_inner(),
                    )),
                    Err(err) => {
                        response_vec.push(pipeline::db_server_response::Response::Error(
                            ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            },
                        ));
                    }
                },

                Query::ListClients(params) => {
                    match self.list_clients(tonic::Request::new(params)).await {
                        Ok(res) => response_vec.push(
                            pipeline::db_server_response::Response::ClientList(res.into_inner()),
                        ),
                        Err(err) => {
                            response_vec.push(pipeline::db_server_response::Response::Error(
                                ErrorResponse {
                                    message: err.message().to_string(),
                                    code: err.code().into(),
                                },
                            ));
                        }
                    }
                }

                Query::InfoServer(params) => {
                    match self.info_server(tonic::Request::new(params)).await {
                        Ok(res) => response_vec.push(
                            pipeline::db_server_response::Response::InfoServer(res.into_inner()),
                        ),
                        Err(err) => {
                            response_vec.push(pipeline::db_server_response::Response::Error(
                                ErrorResponse {
                                    message: err.message().to_string(),
                                    code: err.code().into(),
                                },
                            ));
                        }
                    }
                }

                Query::ListStores(params) => {
                    match self.list_stores(tonic::Request::new(params)).await {
                        Ok(res) => response_vec.push(
                            pipeline::db_server_response::Response::StoreList(res.into_inner()),
                        ),
                        Err(err) => {
                            response_vec.push(pipeline::db_server_response::Response::Error(
                                ErrorResponse {
                                    message: err.message().to_string(),
                                    code: err.code().into(),
                                },
                            ));
                        }
                    }
                }

                Query::DelKey(params) => match self.del_key(tonic::Request::new(params)).await {
                    Ok(res) => response_vec.push(pipeline::db_server_response::Response::Del(
                        res.into_inner(),
                    )),
                    Err(err) => {
                        response_vec.push(pipeline::db_server_response::Response::Error(
                            ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            },
                        ));
                    }
                },

                Query::DropNonLinearAlgorithmIndex(params) => {
                    match self
                        .drop_non_linear_algorithm_index(tonic::Request::new(params))
                        .await
                    {
                        Ok(res) => response_vec.push(pipeline::db_server_response::Response::Del(
                            res.into_inner(),
                        )),
                        Err(err) => {
                            response_vec.push(pipeline::db_server_response::Response::Error(
                                ErrorResponse {
                                    message: err.message().to_string(),
                                    code: err.code().into(),
                                },
                            ));
                        }
                    }
                }

                Query::CreateStore(params) => {
                    match self.create_store(tonic::Request::new(params)).await {
                        Ok(res) => response_vec.push(pipeline::db_server_response::Response::Unit(
                            res.into_inner(),
                        )),
                        Err(err) => {
                            response_vec.push(pipeline::db_server_response::Response::Error(
                                ErrorResponse {
                                    message: err.message().to_string(),
                                    code: err.code().into(),
                                },
                            ));
                        }
                    }
                }
                Query::GetSimN(params) => match self.get_sim_n(tonic::Request::new(params)).await {
                    Ok(res) => response_vec.push(pipeline::db_server_response::Response::GetSimN(
                        res.into_inner(),
                    )),
                    Err(err) => {
                        response_vec.push(pipeline::db_server_response::Response::Error(
                            ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            },
                        ));
                    }
                },

                Query::GetKey(params) => match self.get_key(tonic::Request::new(params)).await {
                    Ok(res) => response_vec.push(pipeline::db_server_response::Response::Get(
                        res.into_inner(),
                    )),
                    Err(err) => {
                        response_vec.push(pipeline::db_server_response::Response::Error(
                            ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            },
                        ));
                    }
                },

                Query::GetPred(params) => match self.get_pred(tonic::Request::new(params)).await {
                    Ok(res) => response_vec.push(pipeline::db_server_response::Response::Get(
                        res.into_inner(),
                    )),
                    Err(err) => {
                        response_vec.push(pipeline::db_server_response::Response::Error(
                            ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            },
                        ));
                    }
                },

                Query::CreateNonLinearAlgorithmIndex(params) => {
                    match self
                        .create_non_linear_algorithm_index(tonic::Request::new(params))
                        .await
                    {
                        Ok(res) => response_vec.push(
                            pipeline::db_server_response::Response::CreateIndex(res.into_inner()),
                        ),
                        Err(err) => {
                            response_vec.push(pipeline::db_server_response::Response::Error(
                                ErrorResponse {
                                    message: err.message().to_string(),
                                    code: err.code().into(),
                                },
                            ));
                        }
                    }
                }

                Query::DelPred(params) => match self.del_pred(tonic::Request::new(params)).await {
                    Ok(res) => response_vec.push(pipeline::db_server_response::Response::Del(
                        res.into_inner(),
                    )),
                    Err(err) => {
                        response_vec.push(pipeline::db_server_response::Response::Error(
                            ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            },
                        ));
                    }
                },

                Query::CreatePredIndex(params) => {
                    match self.create_pred_index(tonic::Request::new(params)).await {
                        Ok(res) => response_vec.push(
                            pipeline::db_server_response::Response::CreateIndex(res.into_inner()),
                        ),
                        Err(err) => {
                            response_vec.push(pipeline::db_server_response::Response::Error(
                                ErrorResponse {
                                    message: err.message().to_string(),
                                    code: err.code().into(),
                                },
                            ));
                        }
                    }
                }

                Query::Set(params) => match self.set(tonic::Request::new(params)).await {
                    Ok(res) => response_vec.push(pipeline::db_server_response::Response::Set(
                        res.into_inner(),
                    )),
                    Err(err) => {
                        response_vec.push(pipeline::db_server_response::Response::Error(
                            ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            },
                        ));
                    }
                },

                Query::DropPredIndex(params) => {
                    match self.drop_pred_index(tonic::Request::new(params)).await {
                        Ok(res) => response_vec.push(pipeline::db_server_response::Response::Del(
                            res.into_inner(),
                        )),
                        Err(err) => {
                            response_vec.push(pipeline::db_server_response::Response::Error(
                                ErrorResponse {
                                    message: err.message().to_string(),
                                    code: err.code().into(),
                                },
                            ));
                        }
                    }
                }

                Query::DropStore(params) => {
                    match self.drop_store(tonic::Request::new(params)).await {
                        Ok(res) => response_vec.push(pipeline::db_server_response::Response::Del(
                            res.into_inner(),
                        )),
                        Err(err) => {
                            response_vec.push(pipeline::db_server_response::Response::Error(
                                ErrorResponse {
                                    message: err.message().to_string(),
                                    code: err.code().into(),
                                },
                            ));
                        }
                    }
                }
            }
        }

        let response_vec = response_vec
            .into_iter()
            .map(|res| pipeline::DbServerResponse {
                response: Some(res),
            })
            .collect();
        Ok(tonic::Response::new(pipeline::DbResponsePipeline {
            responses: response_vec,
        }))
    }
}

#[async_trait::async_trait]
impl Task for Server {
    fn task_name(&self) -> String {
        "db-listener".to_string()
    }

    async fn run(&self) -> TaskState {
        match self.listener {
            ListenerStreamOrAddress::ListenerStream(ref stream) => {
                if let Ok((stream, connect_addr)) = stream.as_ref().accept().await {
                    if let Some(connected_client) = self
                        .client_handler
                        .connect(stream.peer_addr().expect("Could not get peer addr"))
                    {
                        log::info!("Connecting to {}", connect_addr);
                        let task = self.create_task(
                            stream,
                            self.listener
                                .local_addr()
                                .expect("Could not get server addr"),
                            connected_client,
                        );
                        self.task_manager.spawn_task_loop(task).await;
                    }
                }
                TaskState::Continue
            }
            _ => TaskState::Break,
        }
    }
}

impl AhnlichServerUtils for Server {
    type PersistenceTask = StoreHandler;

    fn config(&self) -> ServerUtilsConfig {
        ServerUtilsConfig {
            service_name: SERVICE_NAME,
            persist_location: &self.config.common.persist_location,
            persistence_interval: self.config.common.persistence_interval,
            allocator_size: self.config.common.allocator_size,
            threadpool_size: self.config.common.threadpool_size,
        }
    }

    fn cancellation_token(&self) -> CancellationToken {
        self.task_manager.cancellation_token()
    }

    fn store_handler(&self) -> &Arc<Self::PersistenceTask> {
        &self.store_handler
    }

    fn task_manager(&self) -> Arc<TaskManager> {
        self.task_manager.clone()
    }
}

impl Server {
    /// creates a server while injecting a shutdown_token
    pub async fn new_with_config(config: &ServerConfig) -> IoResult<Self> {
        let listener =
            ListenerStreamOrAddress::new(format!("{}:{}", &config.common.host, &config.port))
                .await?;
        let write_flag = Arc::new(AtomicBool::new(false));
        let client_handler = Arc::new(ClientHandler::new(config.common.maximum_clients));
        let mut store_handler = StoreHandler::new(write_flag.clone());
        if let Some(persist_location) = &config.common.persist_location {
            match Persistence::load_snapshot(persist_location) {
                Err(e) => {
                    log::error!("Failed to load snapshot from persist location {e}");
                    if config.common.fail_on_startup_if_persist_load_fails {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        ));
                    }
                }
                Ok(snapshot) => {
                    store_handler.use_snapshot(snapshot);
                }
            }
        };
        Ok(Self {
            listener,
            store_handler: Arc::new(store_handler),
            client_handler,
            task_manager: Arc::new(TaskManager::new()),
            config: config.clone(),
        })
    }

    pub fn client_handler(&self) -> Arc<ClientHandler> {
        Arc::clone(&self.client_handler)
    }

    pub fn local_addr(&self) -> IoResult<SocketAddr> {
        self.listener.local_addr()
    }

    /// initializes a server using server configuration
    pub async fn new(config: &ServerConfig) -> IoResult<Self> {
        // Enable log and tracing
        tracer::init_log_or_trace(
            config.common.enable_tracing,
            SERVICE_NAME,
            &config.common.otel_endpoint,
            &config.common.log_level,
        );
        Self::new_with_config(config).await
    }

    fn create_task(
        &self,
        stream: TcpStream,
        server_addr: SocketAddr,
        connected_client: ConnectedClient,
    ) -> ServerTask {
        let reader = BufReader::new(stream);
        // add client to client_handler
        ServerTask {
            reader: Arc::new(Mutex::new(reader)),
            server_addr,
            connected_client,
            maximum_message_size: self.config.common.message_size as u64,
            // "inexpensive" to clone handlers they can be passed around in an Arc
            client_handler: self.client_handler.clone(),
            store_handler: self.store_handler.clone(),
        }
    }
}

// TODO: next steps:
// - implement ai service for ai
// - implement blocking task for ai server
// - swap out tcp listener for TcpListenerStream
// - rewrite the clients(rust, python) by wrapping around the grpc clients.
// - Everywhere we see tracing, we insert it into the header
// - Remove ahnlich_types, keeping the internal types we'd need
// - rename grpc types to types

#[async_trait::async_trait]
impl BlockingTask for Server {
    fn task_name(&self) -> String {
        "ahnlich-db".to_string()
    }

    async fn run(
        mut self,
        shutdown_signal: std::pin::Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>,
    ) {
        let listener_stream = if let ListenerStreamOrAddress::ListenerStream(stream) = self.listener
        {
            stream
        } else {
            log::error!("listener must be of type listener stream");
            panic!("listener must be of type listener stream")
        };
        let request_tracker = RequestTrackerLayer::new(Arc::clone(&self.client_handler));
        let max_message_size = self.config.common.message_size;
        self.listener = ListenerStreamOrAddress::Address(
            listener_stream
                .as_ref()
                .local_addr()
                .expect("Could not get local address"),
        );

        let db_service = DbServiceServer::new(self).max_decoding_message_size(max_message_size);

        let _ = tonic::transport::Server::builder()
            .layer(request_tracker)
            .trace_fn(trace_with_parent)
            .add_service(db_service)
            .serve_with_incoming_shutdown(listener_stream, shutdown_signal)
            .await;
    }
}
