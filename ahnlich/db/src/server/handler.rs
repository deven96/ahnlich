use crate::cli::ServerConfig;
use crate::engine::store::StoreHandler;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use ahnlich_types::db::pipeline::db_query::Query;
use ahnlich_types::db::server::GetSimNEntry;
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreName, StoreValue};
use ahnlich_types::services::db_service::db_service_server::{DbService, DbServiceServer};
use ahnlich_types::shared::info::ErrorResponse;

use ahnlich_replication::admin::{ClusterAdmin, NodeRegistry};
use ahnlich_replication::config::RaftStorageEngine;
use ahnlich_replication::network::{GrpcRaftNetworkFactory, GrpcRaftService, map_raft_error};
use ahnlich_replication::proto::cluster_admin::cluster_admin_service_client::ClusterAdminServiceClient;
use ahnlich_replication::proto::cluster_admin::cluster_admin_service_server::ClusterAdminServiceServer;
use ahnlich_replication::proto::cluster_admin::{AddLearnerRequest, NodeInfo};
use ahnlich_replication::proto::raft_internal::raft_internal_service_server::RaftInternalServiceServer;
use ahnlich_replication::types::{ClientWriteRequest, DbCommand, DbResponse, DbResponseKind};
use ahnlich_types::db::{pipeline, query, server};
use ahnlich_types::{client as types_client, utils as types_utils};
use itertools::Itertools;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::collections::HashMap;
use std::future::Future;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::{AtomicU64, Ordering};
use task_manager::BlockingTask;
use task_manager::TaskManager;

use tokio_util::sync::CancellationToken;
use utils::allocator::GLOBAL_ALLOCATOR;
use utils::connection_layer::trace_with_parent;

use openraft::storage::Adaptor;
use openraft::{Config as RaftConfig, Raft, SnapshotPolicy};
use prost::Message;
use utils::server::{AhnlichServerUtils, ListenerStreamOrAddress, ServerUtilsConfig};
use utils::{client::ClientHandler, persistence::Persistence};

const SERVICE_NAME: &str = "ahnlich-db";

pub struct Server {
    listener: ListenerStreamOrAddress,
    store_handler: Arc<StoreHandler>,
    client_handler: Arc<ClientHandler>,
    task_manager: Arc<TaskManager>,
    config: ServerConfig,
    raft: Option<Arc<Raft<crate::replication::TypeConfig>>>,
    raft_registry: Option<NodeRegistry>,
    request_seq: AtomicU64,
    pending_join: Option<String>,
}

impl std::fmt::Debug for Server {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AIProxyServer")
            .field("listener", &self.listener)
            .field("store_handler", &self.store_handler)
            .field("client_handler", &self.client_handler)
            .field("task_manager", &self.task_manager)
            .field("config", &self.config)
            .field("raft", &self.raft.as_ref().map(|_| "Raft<...>"))
            .field("raft_registry", &self.raft_registry)
            .field("request_seq", &self.request_seq)
            .field("pending_join", &self.pending_join)
            .finish()
    }
}

#[tonic::async_trait]
impl DbService for Server {
    #[tracing::instrument(skip_all)]
    async fn create_store(
        &self,
        request: tonic::Request<query::CreateStore>,
    ) -> std::result::Result<tonic::Response<server::Unit>, tonic::Status> {
        let client_id = Self::client_id(&request);
        let create_store_params = request.into_inner();
        let raft_payload = self
            .raft
            .is_some()
            .then(|| create_store_params.encode_to_vec());
        let dimensions = match NonZeroUsize::new(create_store_params.dimension as usize) {
            Some(val) => val,
            None => {
                return Err(tonic::Status::invalid_argument(
                    "dimension must be greater than 0",
                ));
            }
        };
        let non_linear_indices = create_store_params
            .non_linear_indices
            .into_iter()
            .filter_map(|index| NonLinearAlgorithm::try_from(index).ok())
            .collect();

        if let Some(raft) = &self.raft {
            let response = raft
                .client_write(ClientWriteRequest {
                    client_id,
                    request_id: self.next_request_id(),
                    command: DbCommand::CreateStore(raft_payload.unwrap()),
                })
                .await
                .map_err(map_raft_error)?;
            let _: server::Unit =
                Self::decode_db_response(response.data.response, DbResponseKind::Unit)?;
            return Ok(tonic::Response::new(server::Unit {}));
        }

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
        self.ensure_linearizable().await?;
        let params = request.into_inner();
        let keys = params
            .keys
            .into_iter()
            .map(|key| StoreKey { key: key.key })
            .collect();

        let entries: Vec<DbStoreEntry> = self
            .store_handler
            .get_key_in_store(
                &StoreName {
                    value: params.store,
                },
                keys,
            )?
            .into_iter()
            .map(|(store_key, store_value)| DbStoreEntry {
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
        self.ensure_linearizable().await?;
        let params = request.into_inner();

        let condition =
            ahnlich_types::unwrap_or_invalid!(params.condition, "Predicate Condition is required");

        let entries = self
            .store_handler
            .get_pred_in_store(
                &StoreName {
                    value: params.store,
                },
                &condition,
            )?
            .into_iter()
            .map(|(store_key, store_value)| DbStoreEntry {
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
        self.ensure_linearizable().await?;
        let params = request.into_inner();
        let search_input =
            ahnlich_types::unwrap_or_invalid!(params.search_input, "search input is required");

        let search_input = StoreKey {
            key: search_input.key,
        };

        let algorithm = ahnlich_types::algorithm::algorithms::Algorithm::try_from(params.algorithm)
            .map_err(|err| tonic::Status::invalid_argument(err.to_string()))?;

        let entries = self
            .store_handler
            .get_sim_in_store(
                &StoreName {
                    value: params.store,
                },
                search_input,
                types_utils::convert_to_nonzerousize(params.closest_n)
                    .map_err(tonic::Status::invalid_argument)?,
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
        self.ensure_linearizable().await?;
        Ok(tonic::Response::new(server::Pong {}))
    }

    #[tracing::instrument(skip_all)]
    async fn create_pred_index(
        &self,
        request: tonic::Request<query::CreatePredIndex>,
    ) -> std::result::Result<tonic::Response<server::CreateIndex>, tonic::Status> {
        let client_id = Self::client_id(&request);
        let params = request.into_inner();
        let raft_payload = self.raft.is_some().then(|| params.encode_to_vec());

        let predicates = params
            .predicates
            .into_iter()
            // .map(MetadataKey::new)
            .collect();

        if let Some(raft) = &self.raft {
            let response = raft
                .client_write(ClientWriteRequest {
                    client_id,
                    request_id: self.next_request_id(),
                    command: DbCommand::CreatePredIndex(raft_payload.unwrap()),
                })
                .await
                .map_err(map_raft_error)?;
            let out: server::CreateIndex =
                Self::decode_db_response(response.data.response, DbResponseKind::CreateIndex)?;
            return Ok(tonic::Response::new(out));
        }

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
        let client_id = Self::client_id(&request);
        let params = request.into_inner();
        let raft_payload = self.raft.is_some().then(|| params.encode_to_vec());

        let non_linear_indices = params
            .non_linear_indices
            .into_iter()
            .filter_map(|val| {
                ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm::try_from(val).ok()
            })
            .collect();

        if let Some(raft) = &self.raft {
            let response = raft
                .client_write(ClientWriteRequest {
                    client_id,
                    request_id: self.next_request_id(),
                    command: DbCommand::CreateNonLinearAlgorithmIndex(raft_payload.unwrap()),
                })
                .await
                .map_err(map_raft_error)?;
            let out: server::CreateIndex =
                Self::decode_db_response(response.data.response, DbResponseKind::CreateIndex)?;
            return Ok(tonic::Response::new(out));
        }

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
        let client_id = Self::client_id(&request);
        let params = request.into_inner();
        let raft_payload = self.raft.is_some().then(|| params.encode_to_vec());

        let predicates = params
            .predicates
            .into_iter()
            // .map(MetadataKey::new)
            .collect();

        if let Some(raft) = &self.raft {
            let response = raft
                .client_write(ClientWriteRequest {
                    client_id,
                    request_id: self.next_request_id(),
                    command: DbCommand::DropPredIndex(raft_payload.unwrap()),
                })
                .await
                .map_err(map_raft_error)?;
            let out: server::Del =
                Self::decode_db_response(response.data.response, DbResponseKind::Del)?;
            return Ok(tonic::Response::new(out));
        }

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
        let client_id = Self::client_id(&request);
        let params = request.into_inner();
        let raft_payload = self.raft.is_some().then(|| params.encode_to_vec());

        let non_linear_indices = params
            .non_linear_indices
            .into_iter()
            .filter_map(|val| {
                ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm::try_from(val).ok()
            })
            .collect();

        if let Some(raft) = &self.raft {
            let response = raft
                .client_write(ClientWriteRequest {
                    client_id,
                    request_id: self.next_request_id(),
                    command: DbCommand::DropNonLinearAlgorithmIndex(raft_payload.unwrap()),
                })
                .await
                .map_err(map_raft_error)?;
            let out: server::Del =
                Self::decode_db_response(response.data.response, DbResponseKind::Del)?;
            return Ok(tonic::Response::new(out));
        }

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
        let client_id = Self::client_id(&request);
        let params = request.into_inner();
        let raft_payload = self.raft.is_some().then(|| params.encode_to_vec());

        let keys = params
            .keys
            .into_iter()
            .map(|key| StoreKey { key: key.key })
            .collect();

        if let Some(raft) = &self.raft {
            let response = raft
                .client_write(ClientWriteRequest {
                    client_id,
                    request_id: self.next_request_id(),
                    command: DbCommand::DelKey(raft_payload.unwrap()),
                })
                .await
                .map_err(map_raft_error)?;
            let out: server::Del =
                Self::decode_db_response(response.data.response, DbResponseKind::Del)?;
            return Ok(tonic::Response::new(out));
        }

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
        let client_id = Self::client_id(&request);
        let params = request.into_inner();
        let raft_payload = self.raft.is_some().then(|| params.encode_to_vec());

        let condition =
            ahnlich_types::unwrap_or_invalid!(params.condition, "Predicate Condition is required");

        if let Some(raft) = &self.raft {
            let response = raft
                .client_write(ClientWriteRequest {
                    client_id,
                    request_id: self.next_request_id(),
                    command: DbCommand::DelPred(raft_payload.unwrap()),
                })
                .await
                .map_err(map_raft_error)?;
            let out: server::Del =
                Self::decode_db_response(response.data.response, DbResponseKind::Del)?;
            return Ok(tonic::Response::new(out));
        }

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
        let client_id = Self::client_id(&request);
        let drop_store_params = request.into_inner();
        let raft_payload = self
            .raft
            .is_some()
            .then(|| drop_store_params.encode_to_vec());

        if let Some(raft) = &self.raft {
            let response = raft
                .client_write(ClientWriteRequest {
                    client_id,
                    request_id: self.next_request_id(),
                    command: DbCommand::DropStore(raft_payload.unwrap()),
                })
                .await
                .map_err(map_raft_error)?;
            let out: server::Del =
                Self::decode_db_response(response.data.response, DbResponseKind::Del)?;
            return Ok(tonic::Response::new(out));
        }

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
        self.ensure_linearizable().await?;
        let clients = self
            .client_handler
            .list()
            .into_iter()
            .map(|client| types_client::ConnectedClient {
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
        self.ensure_linearizable().await?;
        let stores = self
            .store_handler
            .list_stores()
            .into_iter()
            .map(|store| server::StoreInfo {
                name: store.name.to_string(),
                len: store.len,
                size_in_bytes: store.size_in_bytes,
            })
            .sorted()
            .collect();

        Ok(tonic::Response::new(server::StoreList { stores }))
    }

    #[tracing::instrument(skip_all)]
    async fn info_server(
        &self,
        _request: tonic::Request<query::InfoServer>,
    ) -> std::result::Result<tonic::Response<server::InfoServer>, tonic::Status> {
        self.ensure_linearizable().await?;
        let version = env!("CARGO_PKG_VERSION").to_string();
        let server_info = ahnlich_types::shared::info::ServerInfo {
            address: format!("{:?}", self.listener.local_addr()?),
            version,
            r#type: ahnlich_types::server_types::ServerType::Database.into(),
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
        request: tonic::Request<query::Set>,
    ) -> std::result::Result<tonic::Response<server::Set>, tonic::Status> {
        let client_id = Self::client_id(&request);
        let params = request.into_inner();
        let raft_payload = self.raft.is_some().then(|| params.encode_to_vec());
        let inputs = params
            .inputs
            .into_par_iter()
            .filter_map(|entry| match (entry.key, entry.value) {
                (Some(key), Some(value)) => Some((key, value)),
                (Some(key), None) => Some((
                    key,
                    StoreValue {
                        value: HashMap::new(),
                    },
                )),
                _ => None,
            })
            .collect();

        if let Some(raft) = &self.raft {
            let response = raft
                .client_write(ClientWriteRequest {
                    client_id,
                    request_id: self.next_request_id(),
                    command: DbCommand::Set(raft_payload.unwrap()),
                })
                .await
                .map_err(map_raft_error)?;
            let out: server::Set =
                Self::decode_db_response(response.data.response, DbResponseKind::Set)?;
            return Ok(tonic::Response::new(out));
        }

        let set = self.store_handler.set_in_store(
            &StoreName {
                value: params.store,
            },
            inputs,
        )?;

        Ok(tonic::Response::new(server::Set { upsert: Some(set) }))
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
                ahnlich_types::unwrap_or_invalid!(pipeline_query.query, "query is required");

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

impl AhnlichServerUtils for Server {
    type PersistenceTask = StoreHandler;

    fn config(&self) -> ServerUtilsConfig<'_> {
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
        let client_handler = Arc::new(ClientHandler::new(config.common.maximum_clients));
        let listener = ListenerStreamOrAddress::new(
            format!("{}:{}", &config.common.host, &config.port),
            client_handler.clone(),
        )
        .await?;
        let write_flag = Arc::new(AtomicBool::new(false));
        let store_handler = Arc::new(StoreHandler::new(write_flag.clone()));
        if let Some(persist_location) = &config.common.persist_location {
            match Persistence::load_snapshot(persist_location) {
                Err(e) => {
                    log::error!("Failed to load snapshot from persist location {e}");
                    if config.common.fail_on_startup_if_persist_load_fails {
                        return Err(std::io::Error::other(e.to_string()));
                    }
                }
                Ok(snapshot) => {
                    store_handler.use_snapshot(snapshot);
                }
            }
        };
        let mut raft: Option<Arc<Raft<crate::replication::TypeConfig>>> = None;
        let mut raft_registry: Option<NodeRegistry> = None;
        let mut pending_join: Option<String> = None;
        if config.cluster_enabled {
            let raft_storage = match config.raft_storage {
                RaftStorageEngine::Memory => crate::replication::DbStorage::Mem(
                    crate::replication::make_mem_storage(store_handler.clone()),
                ),
                RaftStorageEngine::RocksDb => {
                    let dir = config.raft_data_dir.clone().ok_or_else(|| {
                        std::io::Error::other("raft_data_dir required for rocksdb storage")
                    })?;
                    let storage =
                        crate::replication::make_rocks_storage(store_handler.clone(), dir)
                            .map_err(|e| std::io::Error::other(e.to_string()))?;
                    crate::replication::DbStorage::Rocks(storage)
                }
            };
            let mut raft_config = RaftConfig::default();
            raft_config.cluster_name = "ahnlich-db".to_string();
            raft_config.snapshot_policy = SnapshotPolicy::LogsSinceLast(config.raft_snapshot_logs);
            let raft_config = Arc::new(
                raft_config
                    .validate()
                    .map_err(|e| std::io::Error::other(format!("invalid raft config: {e}")))?,
            );
            let network = GrpcRaftNetworkFactory::<crate::replication::TypeConfig>::default();
            let (log_store, state_machine) = Adaptor::<
                crate::replication::TypeConfig,
                crate::replication::DbStorage,
            >::new(raft_storage);
            let raft_node = Raft::new(
                config.raft_node_id,
                raft_config,
                network,
                log_store,
                state_machine,
            )
            .await
            .map_err(|e| std::io::Error::other(e.to_string()))?;
            let registry = NodeRegistry::new();
            registry.insert(&NodeInfo {
                id: config.raft_node_id,
                raft_addr: config.raft_addr.clone(),
                admin_addr: config.admin_addr.clone(),
                service_addr: format!("{}:{}", &config.common.host, &config.port),
            });
            pending_join = config.raft_join.clone();
            raft = Some(Arc::new(raft_node));
            raft_registry = Some(registry);
        }
        Ok(Self {
            listener,
            store_handler,
            client_handler,
            task_manager: Arc::new(TaskManager::new()),
            config: config.clone(),
            raft,
            raft_registry,
            request_seq: AtomicU64::new(0),
            pending_join,
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

    fn next_request_id(&self) -> u64 {
        self.request_seq.fetch_add(1, Ordering::SeqCst)
    }

    fn client_id<T>(request: &tonic::Request<T>) -> String {
        request
            .remote_addr()
            .map(|addr| addr.to_string())
            .unwrap_or_else(|| "unknown".to_string())
    }

    fn decode_db_response<T: Message + Default>(
        response: DbResponse,
        expected: DbResponseKind,
    ) -> Result<T, tonic::Status> {
        if response.kind != expected {
            return Err(tonic::Status::internal("unexpected raft response type"));
        }
        T::decode(response.payload.as_slice()).map_err(|e| tonic::Status::internal(e.to_string()))
    }

    async fn ensure_linearizable(&self) -> Result<(), tonic::Status> {
        if let Some(raft) = &self.raft {
            raft.ensure_linearizable().await.map_err(map_raft_error)?;
        }
        Ok(())
    }
}

#[async_trait::async_trait]
impl BlockingTask for Server {
    fn task_name(&self) -> String {
        "ahnlich-db".to_string()
    }

    async fn run(
        mut self,
        shutdown_signal: std::pin::Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>,
    ) {
        let token = self.cancellation_token();
        let token_clone = token.clone();
        tokio::spawn(async move {
            shutdown_signal.await;
            token_clone.cancel();
        });
        let listener_stream = if let ListenerStreamOrAddress::ListenerStream(stream) = self.listener
        {
            stream
        } else {
            log::error!("listener must be of type listener stream");
            panic!("listener must be of type listener stream")
        };
        let max_message_size = self.config.common.message_size;
        self.listener = ListenerStreamOrAddress::Address(
            listener_stream
                .as_ref()
                .local_addr()
                .expect("Could not get local address"),
        );

        if let Some(raft) = &self.raft {
            let raft_addr: SocketAddr = self.config.raft_addr.parse().expect("invalid raft_addr");
            let admin_addr: SocketAddr =
                self.config.admin_addr.parse().expect("invalid admin_addr");
            let raft_service = GrpcRaftService::new(raft.clone());
            let raft_svc = RaftInternalServiceServer::new(raft_service);
            let admin_registry = self.raft_registry.clone().unwrap_or_else(NodeRegistry::new);
            let admin_service = ClusterAdmin::new(raft.clone(), admin_registry);
            let admin_svc = ClusterAdminServiceServer::new(admin_service);

            let token_raft = token.clone();
            tokio::spawn(async move {
                let _ = tonic::transport::Server::builder()
                    .trace_fn(trace_with_parent)
                    .add_service(raft_svc)
                    .serve_with_shutdown(raft_addr, token_raft.cancelled())
                    .await;
            });

            let token_admin = token.clone();
            tokio::spawn(async move {
                let _ = tonic::transport::Server::builder()
                    .trace_fn(trace_with_parent)
                    .add_service(admin_svc)
                    .serve_with_shutdown(admin_addr, token_admin.cancelled())
                    .await;
            });
        }

        if let Some(join) = self.pending_join.clone() {
            let node = NodeInfo {
                id: self.config.raft_node_id,
                raft_addr: self.config.raft_addr.clone(),
                admin_addr: self.config.admin_addr.clone(),
                service_addr: format!("{}:{}", &self.config.common.host, &self.config.port),
            };
            tokio::spawn(async move {
                if let Ok(mut client) =
                    ClusterAdminServiceClient::connect(format!("http://{join}")).await
                {
                    let _ = client
                        .add_learner(tonic::Request::new(AddLearnerRequest { node: Some(node) }))
                        .await;
                }
            });
        }

        let db_service = DbServiceServer::new(self).max_decoding_message_size(max_message_size);

        let _ = tonic::transport::Server::builder()
            .trace_fn(trace_with_parent)
            .add_service(db_service)
            .serve_with_incoming_shutdown(listener_stream, token.cancelled())
            .await;
    }
}
