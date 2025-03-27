use crate::cli::server::ModelConfig;
use crate::cli::server::SupportedModels;
use crate::cli::AIProxyConfig;
use crate::engine::ai::models::Model;
use crate::engine::ai::models::ModelDetails;
use crate::engine::store::AIStoreHandler;
use crate::error::AIProxyError;
use crate::manager::ModelManager;
use crate::AHNLICH_AI_RESERVED_META_KEY;
use grpc_types::ai::models::AiModel;
use grpc_types::ai::pipeline::ai_query::Query;
use grpc_types::ai::pipeline::ai_server_response;
use grpc_types::ai::pipeline::AiRequestPipeline;
use grpc_types::ai::pipeline::AiResponsePipeline;
use grpc_types::ai::pipeline::AiServerResponse;
use grpc_types::ai::preprocess::PreprocessAction;
use grpc_types::ai::query::CreateNonLinearAlgorithmIndex;
use grpc_types::ai::query::CreatePredIndex;
use grpc_types::ai::query::CreateStore;
use grpc_types::ai::query::DelKey;
use grpc_types::ai::query::DropNonLinearAlgorithmIndex;
use grpc_types::ai::query::DropPredIndex;
use grpc_types::ai::query::DropStore;
use grpc_types::ai::query::GetKey;
use grpc_types::ai::query::GetPred;
use grpc_types::ai::query::GetSimN;
use grpc_types::ai::query::InfoServer;
use grpc_types::ai::query::ListClients;
use grpc_types::ai::query::ListStores;
use grpc_types::ai::query::Ping;
use grpc_types::ai::query::PurgeStores;
use grpc_types::ai::query::Set;
use grpc_types::ai::server;
use grpc_types::ai::server::ClientList;
use grpc_types::ai::server::CreateIndex;
use grpc_types::ai::server::Del;
use grpc_types::ai::server::Get;
use grpc_types::ai::server::Pong;
use grpc_types::ai::server::StoreList;
use grpc_types::ai::server::Unit;
use grpc_types::db::pipeline::db_server_response::Response as DbResponse;
use grpc_types::db::pipeline::DbServerResponse;
use grpc_types::db::query::CreateNonLinearAlgorithmIndex as DbCreateNonLinearAlgorithmIndex;
use grpc_types::db::query::CreatePredIndex as DbCreatePredIndex;
use grpc_types::db::query::CreateStore as DbCreateStore;
use grpc_types::db::query::DelPred;
use grpc_types::db::query::DropNonLinearAlgorithmIndex as DbDropNonLinearAlgorithmIndex;
use grpc_types::db::query::DropPredIndex as DbDropPredIndex;
use grpc_types::db::query::GetPred as DbGetPred;
use grpc_types::db::query::GetSimN as DbGetSimN;
use grpc_types::db::server::Set as DbSet;
use grpc_types::keyval::StoreName;
use grpc_types::keyval::StoreValue;
use grpc_types::metadata::MetadataValue;
use grpc_types::predicates::predicate::Kind as PredicateKind;
use grpc_types::predicates::predicate_condition::Kind;
use grpc_types::predicates::In;
use grpc_types::predicates::Predicate;
use grpc_types::predicates::PredicateCondition;
use grpc_types::services::ai_service::ai_service_server::AiService;
use grpc_types::services::ai_service::ai_service_server::AiServiceServer;
use grpc_types::shared::info::ErrorResponse;
use rayon::prelude::*;
use std::error::Error;
use std::future::Future;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use task_manager::BlockingTask;
use task_manager::TaskManager;
use tokio_util::sync::CancellationToken;
use utils::allocator::GLOBAL_ALLOCATOR;
use utils::client::ClientHandler;
use utils::connection_layer::trace_with_parent;
use utils::connection_layer::RequestTrackerLayer;
use utils::persistence::Persistence;
use utils::server::AhnlichServerUtils;
use utils::server::ListenerStreamOrAddress;
use utils::server::ServerUtilsConfig;

use ahnlich_client_rs::grpc::db::DbClient;

const SERVICE_NAME: &str = "ahnlich-ai";

#[derive(Debug, Clone)]
pub struct AIProxyServer {
    listener: ListenerStreamOrAddress,
    config: AIProxyConfig,
    client_handler: Arc<ClientHandler>,
    store_handler: Arc<AIStoreHandler>,
    task_manager: Arc<TaskManager>,
    db_client: Arc<DbClient>,
    model_manager: Arc<ModelManager>,
}

#[async_trait::async_trait]
impl BlockingTask for AIProxyServer {
    fn task_name(&self) -> String {
        "ahnlich-ai".to_string()
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

        let db_service = AiServiceServer::new(self).max_decoding_message_size(max_message_size);

        let _ = tonic::transport::Server::builder()
            .layer(request_tracker)
            .trace_fn(trace_with_parent)
            .add_service(db_service)
            .serve_with_incoming_shutdown(listener_stream, shutdown_signal)
            .await;
    }
}

#[tonic::async_trait]
impl AiService for AIProxyServer {
    #[tracing::instrument(skip_all)]
    async fn info_server(
        &self,
        _request: tonic::Request<InfoServer>,
    ) -> std::result::Result<tonic::Response<server::InfoServer>, tonic::Status> {
        let version = env!("CARGO_PKG_VERSION").to_string();
        let server_info = grpc_types::shared::info::ServerInfo {
            address: format!("{:?}", self.listener.local_addr()),
            version,
            r#type: grpc_types::server_types::ServerType::Ai.into(),
            limit: GLOBAL_ALLOCATOR.limit() as u64,
            remaining: GLOBAL_ALLOCATOR.remaining() as u64,
        };

        let info_server = server::InfoServer {
            info: Some(server_info),
        };

        Ok(tonic::Response::new(info_server))
    }

    #[tracing::instrument(skip_all)]
    async fn create_store(
        &self,
        request: tonic::Request<CreateStore>,
    ) -> Result<tonic::Response<Unit>, tonic::Status> {
        let params = request.into_inner();
        let default_metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;
        let mut predicates = params.predicates;
        if params.store_original {
            predicates.push(default_metadata_key.to_string());
        }
        let index_model: AiModel = params
            .index_model
            .try_into()
            .map_err(|_| AIProxyError::InputNotSpecified("Index model".to_string()))?;
        let query_model: AiModel = params
            .query_model
            .try_into()
            .map_err(|_| AIProxyError::InputNotSpecified("Query model".to_string()))?;
        let model: ModelDetails = SupportedModels::from(&index_model).to_model_details();
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        let _ = self
            .db_client
            .create_store(
                DbCreateStore {
                    store: params.store.clone(),
                    dimension: model.embedding_size.get() as u32,
                    create_predicates: predicates,
                    non_linear_indices: params.non_linear_indices,
                    error_if_exists: params.error_if_exists,
                },
                parent_id,
            )
            .await?;
        let _ = self.store_handler.create_store(
            StoreName {
                value: params.store,
            },
            query_model,
            index_model,
            params.error_if_exists,
            params.store_original,
        )?;
        Ok(tonic::Response::new(Unit {}))
    }

    #[tracing::instrument(skip_all)]
    async fn create_pred_index(
        &self,
        request: tonic::Request<CreatePredIndex>,
    ) -> Result<tonic::Response<CreateIndex>, tonic::Status> {
        let params = request.into_inner();
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        let res = self
            .db_client
            .create_pred_index(
                DbCreatePredIndex {
                    store: params.store,
                    predicates: params.predicates,
                },
                parent_id,
            )
            .await?;
        Ok(tonic::Response::new(CreateIndex {
            created_indexes: res.created_indexes,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn create_non_linear_algorithm_index(
        &self,
        request: tonic::Request<CreateNonLinearAlgorithmIndex>,
    ) -> Result<tonic::Response<CreateIndex>, tonic::Status> {
        let params = request.into_inner();
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        let res = self
            .db_client
            .create_non_linear_algorithm_index(
                DbCreateNonLinearAlgorithmIndex {
                    store: params.store,
                    non_linear_indices: params.non_linear_indices,
                },
                parent_id,
            )
            .await?;
        Ok(tonic::Response::new(CreateIndex {
            created_indexes: res.created_indexes,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn get_key(
        &self,
        request: tonic::Request<GetKey>,
    ) -> Result<tonic::Response<Get>, tonic::Status> {
        let params = request.into_inner();
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        let values = params
            .keys
            .into_par_iter()
            .filter_map(|entry| entry.try_into().ok())
            .collect();
        let condition = Some(PredicateCondition {
            kind: Some(Kind::Value(Predicate {
                kind: Some(PredicateKind::In(In {
                    key: AHNLICH_AI_RESERVED_META_KEY.to_string(),
                    values,
                })),
            })),
        });
        let res = self
            .db_client
            .get_pred(
                DbGetPred {
                    store: params.store,
                    condition,
                },
                parent_id,
            )
            .await?;
        let entries = self
            .store_handler
            .db_store_entry_to_store_get_key(res.entries);
        Ok(tonic::Response::new(Get { entries }))
    }

    #[tracing::instrument(skip_all)]
    async fn get_pred(
        &self,
        request: tonic::Request<GetPred>,
    ) -> Result<tonic::Response<Get>, tonic::Status> {
        let params = request.into_inner();
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        let res = self
            .db_client
            .get_pred(
                DbGetPred {
                    store: params.store,
                    condition: params.condition,
                },
                parent_id,
            )
            .await?;
        let entries = self
            .store_handler
            .db_store_entry_to_store_get_key(res.entries);
        Ok(tonic::Response::new(Get { entries }))
    }

    #[tracing::instrument(skip_all)]
    async fn get_sim_n(
        &self,
        request: tonic::Request<GetSimN>,
    ) -> Result<tonic::Response<server::GetSimN>, tonic::Status> {
        let params = request.into_inner();
        let search_input = params
            .search_input
            .ok_or_else(|| AIProxyError::InputNotSpecified("Search".to_string()))?;
        let search_input = self
            .store_handler
            .get_ndarray_repr_for_store(
                &StoreName {
                    value: params.store.clone(),
                },
                search_input,
                &self.model_manager,
                TryInto::<PreprocessAction>::try_into(params.preprocess_action)
                    .map_err(AIProxyError::from)?,
                params.execution_provider.and_then(|a| a.try_into().ok()),
            )
            .await?;
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        let get_sim_n_params = DbGetSimN {
            store: params.store,
            search_input: Some(search_input),
            closest_n: params.closest_n,
            algorithm: params.algorithm,
            condition: params.condition,
        };
        let response = self
            .db_client
            .get_sim_n(get_sim_n_params, parent_id)
            .await?;
        let (store_key_input, similarities): (Vec<_>, Vec<_>) = response
            .entries
            .into_par_iter()
            .flat_map(|entry| {
                if let (Some(key), Some(value), Some(similarity)) =
                    (entry.key, entry.value, entry.similarity)
                {
                    Some(((key, value), similarity))
                } else {
                    None
                }
            })
            .unzip();
        let entries = self
            .store_handler
            .store_key_val_to_store_input_val(store_key_input)
            .into_par_iter()
            .zip(similarities.into_par_iter())
            .map(|((key, value), similarity)| server::GetSimNEntry {
                key,
                value: Some(value),
                similarity: Some(similarity),
            })
            .collect();
        Ok(tonic::Response::new(server::GetSimN { entries }))
    }

    #[tracing::instrument(skip_all)]
    async fn set(
        &self,
        request: tonic::Request<Set>,
    ) -> Result<tonic::Response<server::Set>, tonic::Status> {
        let params = request.into_inner();
        let model_manager = &self.model_manager;
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        let (db_inputs, delete_hashset) = self
            .store_handler
            .set(
                &StoreName {
                    value: params.store.clone(),
                },
                params
                    .inputs
                    .into_par_iter()
                    .flat_map(|a| a.key.map(|b| (b, StoreValue { value: a.value })))
                    .collect(),
                model_manager,
                TryInto::<PreprocessAction>::try_into(params.preprocess_action)
                    .map_err(AIProxyError::from)?,
                params.execution_provider.and_then(|a| a.try_into().ok()),
            )
            .await?;
        let mut pipeline = self.db_client.pipeline(parent_id);
        if let Some(del_hashset) = delete_hashset {
            let default_metadatakey = &*AHNLICH_AI_RESERVED_META_KEY;
            let delete_condition = DelPred {
                store: params.store.clone(),
                condition: Some(PredicateCondition {
                    kind: Some(Kind::Value(Predicate {
                        kind: Some(PredicateKind::In(In {
                            key: default_metadatakey.to_string(),
                            values: del_hashset.into_iter().collect(),
                        })),
                    })),
                }),
            };
            pipeline.del_pred(delete_condition);
        }
        let set_params = grpc_types::db::query::Set {
            store: params.store,
            inputs: db_inputs,
        };
        pipeline.set(set_params);
        match pipeline.exec().await?.responses.as_slice() {
            [DbServerResponse {
                response: Some(DbResponse::Set(DbSet { upsert })),
            }]
            | [DbServerResponse {
                response: Some(DbResponse::Del(_)),
            }, DbServerResponse {
                response: Some(DbResponse::Set(DbSet { upsert })),
            }] => Ok(tonic::Response::new(server::Set { upsert: *upsert })),
            e => return Err(AIProxyError::UnexpectedDBResponse(format!("{e:?}")).into()),
        }
    }

    #[tracing::instrument(skip_all)]
    async fn drop_pred_index(
        &self,
        request: tonic::Request<DropPredIndex>,
    ) -> Result<tonic::Response<Del>, tonic::Status> {
        let mut params = request.into_inner();
        let default_metadatakey = &*AHNLICH_AI_RESERVED_META_KEY;
        params.predicates.retain(|val| val != default_metadatakey);
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        let res = self
            .db_client
            .drop_pred_index(
                DbDropPredIndex {
                    store: params.store,
                    predicates: params.predicates,
                    error_if_not_exists: params.error_if_not_exists,
                },
                parent_id,
            )
            .await?;
        Ok(tonic::Response::new(Del {
            deleted_count: res.deleted_count,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn drop_non_linear_algorithm_index(
        &self,
        request: tonic::Request<DropNonLinearAlgorithmIndex>,
    ) -> Result<tonic::Response<Del>, tonic::Status> {
        let params = request.into_inner();
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        let res = self
            .db_client
            .drop_non_linear_algorithm_index(
                DbDropNonLinearAlgorithmIndex {
                    store: params.store,
                    non_linear_indices: params.non_linear_indices,
                    error_if_not_exists: params.error_if_not_exists,
                },
                parent_id,
            )
            .await?;
        Ok(tonic::Response::new(Del {
            deleted_count: res.deleted_count,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn del_key(
        &self,
        request: tonic::Request<DelKey>,
    ) -> Result<tonic::Response<Del>, tonic::Status> {
        let params = request.into_inner();
        let store_original = self.store_handler.store_original(StoreName {
            value: params.store.clone(),
        })?;
        if !store_original {
            return Err(AIProxyError::DelKeyError.into());
        } else {
            let default_metadatakey = &*AHNLICH_AI_RESERVED_META_KEY;
            let key = params
                .key
                .ok_or_else(|| AIProxyError::InputNotSpecified("Del key".to_string()))?;

            let metadata_value: MetadataValue = key
                .try_into()
                .map_err(|_| AIProxyError::InputNotSpecified("Store Input Value".to_string()))?;

            let del_pred_params = DelPred {
                store: params.store,
                condition: Some(PredicateCondition {
                    kind: Some(Kind::Value(Predicate {
                        kind: Some(PredicateKind::In(In {
                            key: default_metadatakey.to_string(),
                            values: vec![metadata_value],
                        })),
                    })),
                }),
            };
            let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
            let res = self.db_client.del_pred(del_pred_params, parent_id).await?;
            Ok(tonic::Response::new(Del {
                deleted_count: res.deleted_count,
            }))
        }
    }

    #[tracing::instrument(skip_all)]
    async fn drop_store(
        &self,
        request: tonic::Request<DropStore>,
    ) -> Result<tonic::Response<Del>, tonic::Status> {
        let drop_store_params = request.into_inner();
        let dropped = self.store_handler.drop_store(
            StoreName {
                value: drop_store_params.store,
            },
            drop_store_params.error_if_not_exists,
        )?;
        Ok(tonic::Response::new(Del {
            deleted_count: dropped as u64,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn list_clients(
        &self,
        _request: tonic::Request<ListClients>,
    ) -> Result<tonic::Response<ClientList>, tonic::Status> {
        Ok(tonic::Response::new(server::ClientList {
            clients: self.client_handler.list().into_iter().collect(),
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn list_stores(
        &self,
        _request: tonic::Request<ListStores>,
    ) -> Result<tonic::Response<StoreList>, tonic::Status> {
        Ok(tonic::Response::new(server::StoreList {
            stores: self.store_handler.list_stores().into_iter().collect(),
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn purge_stores(
        &self,
        _request: tonic::Request<PurgeStores>,
    ) -> Result<tonic::Response<Del>, tonic::Status> {
        let deleted_count = self.store_handler.purge_stores();
        Ok(tonic::Response::new(Del {
            deleted_count: deleted_count as u64,
        }))
    }

    #[tracing::instrument(skip_all)]
    async fn ping(
        &self,
        _request: tonic::Request<Ping>,
    ) -> Result<tonic::Response<Pong>, tonic::Status> {
        Ok(tonic::Response::new(Pong {}))
    }

    #[tracing::instrument(skip_all)]
    async fn pipeline(
        &self,
        request: tonic::Request<AiRequestPipeline>,
    ) -> Result<tonic::Response<AiResponsePipeline>, tonic::Status> {
        let params = request.into_inner();
        let mut response_vec = Vec::with_capacity(params.queries.len());

        for pipeline_query in params.queries {
            let pipeline_query =
                grpc_types::unwrap_or_invalid!(pipeline_query.query, "query is required");

            match pipeline_query {
                Query::Ping(params) => match self.ping(tonic::Request::new(params)).await {
                    Ok(res) => {
                        response_vec.push(ai_server_response::Response::Pong(res.into_inner()))
                    }
                    Err(err) => {
                        response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                            message: err.message().to_string(),
                            code: err.code().into(),
                        }));
                    }
                },

                Query::CreateStore(params) => {
                    match self.create_store(tonic::Request::new(params)).await {
                        Ok(res) => {
                            response_vec.push(ai_server_response::Response::Unit(res.into_inner()))
                        }
                        Err(err) => {
                            response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            }));
                        }
                    }
                }

                Query::CreatePredIndex(params) => {
                    match self.create_pred_index(tonic::Request::new(params)).await {
                        Ok(res) => response_vec
                            .push(ai_server_response::Response::CreateIndex(res.into_inner())),
                        Err(err) => {
                            response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            }));
                        }
                    }
                }

                Query::CreateNonLinearAlgorithmIndex(params) => {
                    match self
                        .create_non_linear_algorithm_index(tonic::Request::new(params))
                        .await
                    {
                        Ok(res) => response_vec
                            .push(ai_server_response::Response::CreateIndex(res.into_inner())),
                        Err(err) => {
                            response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            }));
                        }
                    }
                }

                Query::GetSimN(params) => match self.get_sim_n(tonic::Request::new(params)).await {
                    Ok(res) => {
                        response_vec.push(ai_server_response::Response::GetSimN(res.into_inner()))
                    }
                    Err(err) => {
                        response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                            message: err.message().to_string(),
                            code: err.code().into(),
                        }));
                    }
                },

                Query::GetPred(params) => match self.get_pred(tonic::Request::new(params)).await {
                    Ok(res) => {
                        response_vec.push(ai_server_response::Response::Get(res.into_inner()))
                    }
                    Err(err) => {
                        response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                            message: err.message().to_string(),
                            code: err.code().into(),
                        }));
                    }
                },

                Query::ListClients(params) => {
                    match self.list_clients(tonic::Request::new(params)).await {
                        Ok(res) => response_vec
                            .push(ai_server_response::Response::ClientList(res.into_inner())),
                        Err(err) => {
                            response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            }));
                        }
                    }
                }

                Query::ListStores(params) => {
                    match self.list_stores(tonic::Request::new(params)).await {
                        Ok(res) => response_vec
                            .push(ai_server_response::Response::StoreList(res.into_inner())),
                        Err(err) => {
                            response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            }));
                        }
                    }
                }

                Query::InfoServer(params) => {
                    match self.info_server(tonic::Request::new(params)).await {
                        Ok(res) => response_vec
                            .push(ai_server_response::Response::InfoServer(res.into_inner())),
                        Err(err) => {
                            response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            }));
                        }
                    }
                }

                Query::GetKey(params) => match self.get_key(tonic::Request::new(params)).await {
                    Ok(res) => {
                        response_vec.push(ai_server_response::Response::Get(res.into_inner()))
                    }
                    Err(err) => {
                        response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                            message: err.message().to_string(),
                            code: err.code().into(),
                        }));
                    }
                },

                Query::DelKey(params) => match self.del_key(tonic::Request::new(params)).await {
                    Ok(res) => {
                        response_vec.push(ai_server_response::Response::Del(res.into_inner()))
                    }
                    Err(err) => {
                        response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                            message: err.message().to_string(),
                            code: err.code().into(),
                        }));
                    }
                },

                Query::DropPredIndex(params) => {
                    match self.drop_pred_index(tonic::Request::new(params)).await {
                        Ok(res) => {
                            response_vec.push(ai_server_response::Response::Del(res.into_inner()))
                        }
                        Err(err) => {
                            response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            }));
                        }
                    }
                }

                Query::DropNonLinearAlgorithmIndex(params) => match self
                    .drop_non_linear_algorithm_index(tonic::Request::new(params))
                    .await
                {
                    Ok(res) => {
                        response_vec.push(ai_server_response::Response::Del(res.into_inner()))
                    }
                    Err(err) => {
                        response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                            message: err.message().to_string(),
                            code: err.code().into(),
                        }));
                    }
                },

                Query::PurgeStores(params) => {
                    match self.purge_stores(tonic::Request::new(params)).await {
                        Ok(res) => {
                            response_vec.push(ai_server_response::Response::Del(res.into_inner()))
                        }
                        Err(err) => {
                            response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            }));
                        }
                    }
                }

                Query::Set(params) => match self.set(tonic::Request::new(params)).await {
                    Ok(res) => {
                        response_vec.push(ai_server_response::Response::Set(res.into_inner()))
                    }
                    Err(err) => {
                        response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                            message: err.message().to_string(),
                            code: err.code().into(),
                        }));
                    }
                },

                Query::DropStore(params) => {
                    match self.drop_store(tonic::Request::new(params)).await {
                        Ok(res) => {
                            response_vec.push(ai_server_response::Response::Del(res.into_inner()))
                        }
                        Err(err) => {
                            response_vec.push(ai_server_response::Response::Error(ErrorResponse {
                                message: err.message().to_string(),
                                code: err.code().into(),
                            }));
                        }
                    }
                }
            }
        }
        let response_vec = response_vec
            .into_iter()
            .map(|res| AiServerResponse {
                response: Some(res),
            })
            .collect();
        Ok(tonic::Response::new(AiResponsePipeline {
            responses: response_vec,
        }))
    }
}

impl AhnlichServerUtils for AIProxyServer {
    type PersistenceTask = AIStoreHandler;

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

impl AIProxyServer {
    pub async fn new(config: AIProxyConfig) -> Result<Self, Box<dyn Error>> {
        Self::build(config).await
    }

    pub async fn build(config: AIProxyConfig) -> Result<Self, Box<dyn Error>> {
        // Enable log and tracing
        tracer::init_log_or_trace(
            config.common.enable_tracing,
            SERVICE_NAME,
            &config.common.otel_endpoint,
            &config.common.log_level,
        );
        let listener =
            ListenerStreamOrAddress::new(format!("{}:{}", &config.common.host, &config.port))
                .await?;
        let write_flag = Arc::new(AtomicBool::new(false));
        let db_client = Self::build_db_client(&config).await;
        let mut store_handler =
            AIStoreHandler::new(write_flag.clone(), config.supported_models.clone());
        if let Some(ref persist_location) = config.common.persist_location {
            match Persistence::load_snapshot(persist_location) {
                Err(e) => {
                    log::error!("Failed to load snapshot from persist location {e}");
                    if config.common.fail_on_startup_if_persist_load_fails {
                        return Err(Box::new(e));
                    }
                }
                Ok(snapshot) => {
                    store_handler.use_snapshot(snapshot);
                }
            }
        };
        let client_handler = Arc::new(ClientHandler::new(config.common.maximum_clients));
        let task_manager = Arc::new(TaskManager::new());
        let mut models: Vec<Model> = Vec::with_capacity(config.supported_models.len());
        for supported_model in &config.supported_models {
            let model = supported_model
                .to_concrete_model(config.model_cache_location.to_path_buf())
                .await?;
            // TODO (HAKSOAT): Handle if download fails or verification check fails
            model.get().await?;
            models.push(model);
        }

        let model_config = ModelConfig::from(&config);
        let model_manager = ModelManager::new(model_config, task_manager.clone()).await?;

        Ok(Self {
            listener,
            client_handler,
            store_handler: Arc::new(store_handler),
            config,
            db_client: Arc::new(db_client),
            task_manager,
            model_manager: Arc::new(model_manager),
        })
    }

    async fn build_db_client(config: &AIProxyConfig) -> DbClient {
        let scheme = if config.db_https { "https" } else { "http" };
        let addr = format!("{scheme}://{}:{}", config.db_host, config.db_port);
        DbClient::new(addr).await.expect("Cannot start DB client")
    }

    pub fn local_addr(&self) -> IoResult<SocketAddr> {
        self.listener.local_addr()
    }
}
