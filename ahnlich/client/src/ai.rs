use ahnlich_types::{
    ai::{
        pipeline::{ai_query::Query, AiQuery, AiRequestPipeline, AiResponsePipeline},
        query::{
            CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey,
            DropNonLinearAlgorithmIndex, DropPredIndex, DropStore, GetKey, GetPred, GetSimN,
            InfoServer, ListClients, ListStores, Ping, PurgeStores, Set,
        },
        server::{
            ClientList, CreateIndex, Del, Get, GetSimN as GetSimNResult, Pong, Set as SetResult,
            StoreList, Unit,
        },
    },
    services::ai_service::ai_service_client::AiServiceClient,
    shared::info::ServerInfo,
    utils::add_trace_parent,
};
use tonic::transport::Channel;

use crate::error::AhnlichError;

#[derive(Debug, Clone)]
pub struct AiPipeline {
    queries: Vec<Query>,
    tracing_id: Option<String>,
    client: AiServiceClient<Channel>,
}

impl AiPipeline {
    pub fn create_store(&mut self, params: CreateStore) {
        self.queries.push(Query::CreateStore(params));
    }

    pub fn create_pred_index(&mut self, params: CreatePredIndex) {
        self.queries.push(Query::CreatePredIndex(params));
    }

    pub fn create_non_linear_algorithm_index(&mut self, params: CreateNonLinearAlgorithmIndex) {
        self.queries
            .push(Query::CreateNonLinearAlgorithmIndex(params));
    }

    pub fn get_key(&mut self, params: GetKey) {
        self.queries.push(Query::GetKey(params));
    }

    pub fn get_pred(&mut self, params: GetPred) {
        self.queries.push(Query::GetPred(params));
    }

    pub fn get_sim_n(&mut self, params: GetSimN) {
        self.queries.push(Query::GetSimN(params));
    }

    pub fn set(&mut self, params: Set) {
        self.queries.push(Query::Set(params));
    }

    pub fn drop_pred_index(&mut self, params: DropPredIndex) {
        self.queries.push(Query::DropPredIndex(params));
    }

    pub fn drop_non_linear_algorithm_index(&mut self, params: DropNonLinearAlgorithmIndex) {
        self.queries
            .push(Query::DropNonLinearAlgorithmIndex(params));
    }

    pub fn del_key(&mut self, params: DelKey) {
        self.queries.push(Query::DelKey(params));
    }

    pub fn drop_store(&mut self, params: DropStore) {
        self.queries.push(Query::DropStore(params));
    }

    pub fn info_server(&mut self) {
        self.queries.push(Query::InfoServer(InfoServer {}));
    }

    pub fn purge_stores(&mut self) {
        self.queries.push(Query::PurgeStores(PurgeStores {}));
    }

    pub fn list_stores(&mut self) {
        self.queries.push(Query::ListStores(ListStores {}));
    }

    pub fn list_clients(&mut self) {
        self.queries.push(Query::ListClients(ListClients {}));
    }

    pub fn ping(&mut self) {
        self.queries.push(Query::Ping(Ping {}));
    }

    pub async fn exec(mut self) -> Result<AiResponsePipeline, AhnlichError> {
        let tracing_id = self.tracing_id.clone();
        let mut req = tonic::Request::new(AiRequestPipeline {
            queries: self
                .queries
                .into_iter()
                .map(|q| AiQuery { query: Some(q) })
                .collect(),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.pipeline(req).await?.into_inner())
    }

    pub fn set_queries(&mut self, queries: Vec<Query>) {
        self.queries = queries
    }
}

// GRPC Client for Ahnlich AI
//
// client needs &mut as it can only send one request in flight, hence is not thread safe to use,
// however `Channel` makes use of `tower_buffer::Buffer` underneath and hence DBClient is cheap
// to clone and is encouraged for use across multiple threads
// https://docs.rs/tonic/latest/tonic/transport/struct.Channel.html#multiplexing-requests
// So we clone client underneath with every call to create a threadsafe client
#[derive(Debug, Clone)]
pub struct AiClient {
    client: AiServiceClient<Channel>,
}

impl AiClient {
    pub async fn new(addr: String) -> Result<Self, AhnlichError> {
        let addr = if !(addr.starts_with("https://") || addr.starts_with("http://")) {
            format!("http://{addr}")
        } else {
            addr
        };
        let channel = Channel::from_shared(addr)?;
        let client = AiServiceClient::connect(channel).await?;
        Ok(Self { client })
    }

    pub async fn create_store(
        &self,
        params: CreateStore,
        tracing_id: Option<String>,
    ) -> Result<Unit, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().create_store(req).await?.into_inner())
    }

    pub async fn create_pred_index(
        &self,
        params: CreatePredIndex,
        tracing_id: Option<String>,
    ) -> Result<CreateIndex, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self
            .client
            .clone()
            .create_pred_index(req)
            .await?
            .into_inner())
    }

    pub async fn create_non_linear_algorithm_index(
        &self,
        params: CreateNonLinearAlgorithmIndex,
        tracing_id: Option<String>,
    ) -> Result<CreateIndex, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self
            .client
            .clone()
            .create_non_linear_algorithm_index(req)
            .await?
            .into_inner())
    }

    pub async fn get_key(
        &self,
        params: GetKey,
        tracing_id: Option<String>,
    ) -> Result<Get, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().get_key(req).await?.into_inner())
    }

    pub async fn get_pred(
        &self,
        params: GetPred,
        tracing_id: Option<String>,
    ) -> Result<Get, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().get_pred(req).await?.into_inner())
    }

    pub async fn get_sim_n(
        &self,
        params: GetSimN,
        tracing_id: Option<String>,
    ) -> Result<GetSimNResult, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().get_sim_n(req).await?.into_inner())
    }

    pub async fn set(
        &self,
        params: Set,
        tracing_id: Option<String>,
    ) -> Result<SetResult, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().set(req).await?.into_inner())
    }

    pub async fn drop_pred_index(
        &self,
        params: DropPredIndex,
        tracing_id: Option<String>,
    ) -> Result<Del, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().drop_pred_index(req).await?.into_inner())
    }

    pub async fn drop_non_linear_algorithm_index(
        &self,
        params: DropNonLinearAlgorithmIndex,
        tracing_id: Option<String>,
    ) -> Result<Del, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self
            .client
            .clone()
            .drop_non_linear_algorithm_index(req)
            .await?
            .into_inner())
    }

    pub async fn del_key(
        &self,
        params: DelKey,
        tracing_id: Option<String>,
    ) -> Result<Del, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().del_key(req).await?.into_inner())
    }

    pub async fn drop_store(
        &self,
        params: DropStore,
        tracing_id: Option<String>,
    ) -> Result<Del, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().drop_store(req).await?.into_inner())
    }
    pub async fn list_clients(
        &self,
        tracing_id: Option<String>,
    ) -> Result<ClientList, AhnlichError> {
        let mut req = tonic::Request::new(ListClients {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().list_clients(req).await?.into_inner())
    }

    pub async fn list_stores(&self, tracing_id: Option<String>) -> Result<StoreList, AhnlichError> {
        let mut req = tonic::Request::new(ListStores {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().list_stores(req).await?.into_inner())
    }

    pub async fn info_server(
        &self,
        tracing_id: Option<String>,
    ) -> Result<ServerInfo, AhnlichError> {
        let mut req = tonic::Request::new(InfoServer {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self
            .client
            .clone()
            .info_server(req)
            .await?
            .into_inner()
            .info
            .expect("Server info should be Some"))
    }

    pub async fn purge_stores(&self, tracing_id: Option<String>) -> Result<Del, AhnlichError> {
        let mut req = tonic::Request::new(PurgeStores {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().purge_stores(req).await?.into_inner())
    }

    pub async fn ping(&self, tracing_id: Option<String>) -> Result<Pong, AhnlichError> {
        let mut req = tonic::Request::new(Ping {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().ping(req).await?.into_inner())
    }

    // Create list of instructions to execute in a pipeline loop
    // on the server end
    pub fn pipeline(&self, tracing_id: Option<String>) -> AiPipeline {
        AiPipeline {
            queries: vec![],
            client: self.client.clone(),
            tracing_id,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use ahnlich_ai_proxy::cli::server::SupportedModels;
    use ahnlich_ai_proxy::cli::AIProxyConfig;
    use ahnlich_ai_proxy::engine::ai::models::ModelDetails;
    use ahnlich_ai_proxy::error::AIProxyError;
    use ahnlich_ai_proxy::server::handler::AIProxyServer;

    use ahnlich_db::cli::ServerConfig;
    use ahnlich_db::server::handler::Server;
    use ahnlich_types::ai::models::AiModel;
    use ahnlich_types::ai::pipeline::AiServerResponse;
    use ahnlich_types::ai::preprocess::PreprocessAction;
    use ahnlich_types::ai::query::StoreEntry;
    use ahnlich_types::ai::server::{AiStoreInfo, GetEntry};
    use ahnlich_types::keyval::store_input::Value;
    use ahnlich_types::keyval::{StoreInput, StoreValue};
    use ahnlich_types::metadata::{metadata_value::Value as MValue, MetadataValue};
    use ahnlich_types::shared::info::{ErrorResponse, StoreUpsert};
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use tokio::time::Duration;
    use utils::server::AhnlichServerUtils;

    use ahnlich_types::{ai::pipeline::ai_server_response::Response, keyval::StoreName};

    use ahnlich_types::predicates::{
        self, predicate::Kind as PredicateKind,
        predicate_condition::Kind as PredicateConditionKind, Predicate, PredicateCondition,
    };

    static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());
    static AI_CONFIG: Lazy<AIProxyConfig> = Lazy::new(|| {
        let mut ai_proxy = AIProxyConfig::default().os_select_port();
        ai_proxy.db_port = CONFIG.port.clone();
        ai_proxy.db_host = CONFIG.common.host.clone();
        ai_proxy
    });

    async fn provision_test_servers() -> SocketAddr {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let db_port = server.local_addr().unwrap().port();
        let mut config = AI_CONFIG.clone();
        config.db_port = db_port;

        let ai_server = AIProxyServer::new(config)
            .await
            .expect("Could not initialize ai proxy");

        let ai_address = ai_server.local_addr().expect("Could not get local addr");
        let _ = tokio::spawn(async move { server.start().await });
        // start up ai proxy
        let _ = tokio::spawn(async move { ai_server.start().await });
        // Allow some time for the servers to start
        tokio::time::sleep(Duration::from_millis(200)).await;

        ai_address
    }

    #[tokio::test]
    async fn test_ai_client_ping() {
        let address = provision_test_servers().await;
        let ai_client = AiClient::new(address.to_string())
            .await
            .expect("Could not initialize client");
        assert!(ai_client.ping(None).await.is_ok());
    }

    #[tokio::test]
    async fn test_ai_client_simple_pipeline() {
        let address = provision_test_servers().await;
        let ai_client = AiClient::new(address.to_string())
            .await
            .expect("Could not initialize client");
        let mut pipeline = ai_client.pipeline(None);
        pipeline.list_stores();
        pipeline.ping();

        let expected = AiResponsePipeline {
            responses: vec![
                AiServerResponse {
                    response: Some(Response::StoreList(StoreList { stores: vec![] })),
                },
                AiServerResponse {
                    response: Some(Response::Pong(Pong {})),
                },
            ],
        };

        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_ai_client_init_fail_if_server_not_exist() {
        let host = "127.0.0.1";
        let port = 1234;
        let ai_client = AiClient::new(format!("{host}:{port}")).await;
        assert!(ai_client.is_err());
    }

    #[tokio::test]
    async fn test_create_stores_with_pipeline() {
        let address = provision_test_servers().await;
        let ai_client = AiClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let mut pipeline = ai_client.pipeline(None);

        let create_store_params = CreateStore {
            store: "Main".to_string(),
            index_model: AiModel::AllMiniLmL6V2 as i32,
            query_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };

        let create_store_params_2 = CreateStore {
            store: "Main".to_string(),
            index_model: AiModel::AllMiniLmL6V2 as i32,
            query_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };

        let create_store_params_no_error = CreateStore {
            store: "Main".to_string(),
            index_model: AiModel::AllMiniLmL6V2 as i32,
            query_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: false,
            store_original: true,
        };

        pipeline.create_store(create_store_params);
        pipeline.create_store(create_store_params_2);
        pipeline.create_store(create_store_params_no_error);

        let create_store_params = CreateStore {
            store: "Less".to_string(),
            index_model: AiModel::AllMiniLmL6V2 as i32,
            query_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };

        pipeline.create_store(create_store_params);
        pipeline.list_stores();

        let ai_model: ModelDetails =
            SupportedModels::from(&AiModel::AllMiniLmL6V2).to_model_details();
        let already_exists_error = AIProxyError::StoreAlreadyExists(StoreName {
            value: "Main".into(),
        });

        let expected = AiResponsePipeline {
            responses: vec![
                AiServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                AiServerResponse {
                    response: Some(Response::Error(ErrorResponse {
                        message: already_exists_error.to_string(),
                        code: 6,
                    })),
                },
                AiServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                AiServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                AiServerResponse {
                    response: Some(Response::StoreList(StoreList {
                        stores: vec![
                            AiStoreInfo {
                                name: "Less".to_string(),
                                embedding_size: ai_model.embedding_size.get() as u64,
                                query_model: AiModel::AllMiniLmL6V2 as i32,
                                index_model: AiModel::AllMiniLmL6V2 as i32,
                            },
                            AiStoreInfo {
                                name: "Main".to_string(),
                                embedding_size: ai_model.embedding_size.get() as u64,
                                query_model: AiModel::AllMiniLmL6V2 as i32,
                                index_model: AiModel::AllMiniLmL6V2 as i32,
                            },
                        ],
                    })),
                },
            ],
        };

        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res.responses.len(), expected.responses.len());
        assert_eq!(res, expected);

        for expected_entry in expected.responses {
            assert!(
                res.responses.contains(&expected_entry),
                "Missing entry: {:?}",
                expected_entry
            );
        }
    }

    #[tokio::test]
    async fn test_del_key() {
        let address = provision_test_servers().await;
        let ai_client = AiClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let create_store_params = CreateStore {
            store: "Main".to_string(),
            index_model: AiModel::AllMiniLmL6V2 as i32,
            query_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };

        assert!(ai_client
            .create_store(create_store_params, None)
            .await
            .is_ok());

        let set_params = Set {
            store: "Main".to_string(),
            execution_provider: None,
            preprocess_action: PreprocessAction::NoPreprocessing as i32,
            inputs: vec![
                StoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString("Adidas Yeezy".into())),
                    }),
                    value: HashMap::new(),
                },
                StoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString("Nike Air Jordans".into())),
                    }),
                    value: HashMap::new(),
                },
            ],
        };

        assert!(ai_client.set(set_params, None).await.is_ok());

        let delete_key = DelKey {
            store: "Main".to_string(),
            key: Some(StoreInput {
                value: Some(Value::RawString("Adidas Yeezy".into())),
            }),
        };

        assert_eq!(
            ai_client.del_key(delete_key, None).await.unwrap(),
            Del { deleted_count: 1 }
        );
    }

    #[tokio::test]
    async fn test_destroy_purge_stores_with_pipeline() {
        let address = provision_test_servers().await;
        let ai_client = AiClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let mut pipeline = ai_client.pipeline(None);

        let create_store_params = CreateStore {
            store: "Less".to_string(),
            index_model: AiModel::AllMiniLmL6V2 as i32,
            query_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };
        pipeline.create_store(create_store_params);

        let create_store_params_2 = CreateStore {
            store: "Main".to_string(),
            index_model: AiModel::AllMiniLmL6V2 as i32,
            query_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };
        pipeline.create_store(create_store_params_2);

        let create_store_params_3 = CreateStore {
            store: "Main2".to_string(),
            index_model: AiModel::AllMiniLmL6V2 as i32,
            query_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };

        pipeline.create_store(create_store_params_3);
        pipeline.list_stores();

        let drop_store_params = DropStore {
            store: "Less".into(),
            error_if_not_exists: true,
        };

        pipeline.drop_store(drop_store_params);
        pipeline.purge_stores();

        let ai_model: ModelDetails =
            SupportedModels::from(&AiModel::AllMiniLmL6V2).to_model_details();

        let expected = AiResponsePipeline {
            responses: vec![
                AiServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                AiServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                AiServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                AiServerResponse {
                    response: Some(Response::StoreList(StoreList {
                        stores: vec![
                            AiStoreInfo {
                                name: "Less".to_string(),
                                embedding_size: ai_model.embedding_size.get() as u64,
                                query_model: AiModel::AllMiniLmL6V2 as i32,
                                index_model: AiModel::AllMiniLmL6V2 as i32,
                            },
                            AiStoreInfo {
                                name: "Main".to_string(),
                                embedding_size: ai_model.embedding_size.get() as u64,
                                query_model: AiModel::AllMiniLmL6V2 as i32,
                                index_model: AiModel::AllMiniLmL6V2 as i32,
                            },
                            AiStoreInfo {
                                name: "Main2".to_string(),
                                embedding_size: ai_model.embedding_size.get() as u64,
                                query_model: AiModel::AllMiniLmL6V2 as i32,
                                index_model: AiModel::AllMiniLmL6V2 as i32,
                            },
                        ],
                    })),
                },
                AiServerResponse {
                    response: Some(Response::Del(Del { deleted_count: 1 })),
                },
                AiServerResponse {
                    response: Some(Response::Del(Del { deleted_count: 2 })),
                },
            ],
        };

        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_ai_client_get_pred() {
        let address = provision_test_servers().await;

        let ai_client = AiClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let store_name = StoreName {
            value: String::from("Deven Kicks"),
        };

        let matching_metadatakey = "Brand".to_owned();
        let matching_metadatavalue = MetadataValue {
            value: Some(MValue::RawString("Nike".into())),
        };

        let nike_store_value = StoreValue {
            value: HashMap::from_iter([(
                matching_metadatakey.clone(),
                matching_metadatavalue.clone(),
            )]),
        };

        let adidas_store_value = StoreValue {
            value: HashMap::from_iter([(
                matching_metadatakey.clone(),
                MetadataValue {
                    value: Some(MValue::RawString("Adidas".into())),
                },
            )]),
        };

        let store_data = vec![
            StoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::RawString("Air Force 1 Retro Boost".into())),
                }),
                value: nike_store_value.clone().value,
            },
            StoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::RawString("Jordan".into())),
                }),
                value: nike_store_value.clone().value,
            },
            StoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::RawString("Yeezy".into())),
                }),
                value: adidas_store_value.clone().value,
            },
        ];

        let mut pipeline = ai_client.pipeline(None);

        let create_store_params = CreateStore {
            store: store_name.value.clone(),
            index_model: AiModel::AllMiniLmL6V2 as i32,
            query_model: AiModel::AllMiniLmL6V2 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };

        pipeline.create_store(create_store_params);
        pipeline.list_stores();

        let create_pred_index_params = CreatePredIndex {
            store: store_name.value.clone(),
            predicates: vec!["Brand".into(), "Vintage".into()],
        };

        pipeline.create_pred_index(create_pred_index_params);

        let set_params = Set {
            store: store_name.value.clone(),
            inputs: store_data,
            execution_provider: None,
            preprocess_action: PreprocessAction::NoPreprocessing as i32,
        };
        pipeline.set(set_params);

        let drop_pred_params = DropPredIndex {
            store: store_name.value.clone(),
            predicates: vec!["Vintage".to_string()],
            error_if_not_exists: true,
        };

        pipeline.drop_pred_index(drop_pred_params);
        let res = pipeline.exec().await.expect("Could not execute pipeline");

        let ai_model: ModelDetails =
            SupportedModels::from(&AiModel::AllMiniLmL6V2).to_model_details();

        let expected = AiResponsePipeline {
            responses: vec![
                AiServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                AiServerResponse {
                    response: Some(Response::StoreList(StoreList {
                        stores: vec![AiStoreInfo {
                            name: store_name.value.clone(),
                            embedding_size: ai_model.embedding_size.get() as u64,
                            query_model: AiModel::AllMiniLmL6V2 as i32,
                            index_model: AiModel::AllMiniLmL6V2 as i32,
                        }],
                    })),
                },
                AiServerResponse {
                    response: Some(Response::CreateIndex(CreateIndex { created_indexes: 2 })),
                },
                AiServerResponse {
                    response: Some(Response::Set(SetResult {
                        upsert: Some(StoreUpsert {
                            inserted: 3,
                            updated: 0,
                        }),
                    })),
                },
                AiServerResponse {
                    response: Some(Response::Del(Del { deleted_count: 1 })),
                },
            ],
        };

        assert_eq!(res, expected);

        let condition = PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::Equals(predicates::Equals {
                    key: matching_metadatakey,
                    value: Some(matching_metadatavalue),
                })),
            })),
        };

        let get_pred_params = GetPred {
            store: store_name.value,
            condition: Some(condition),
        };

        let response = ai_client.get_pred(get_pred_params, None).await.unwrap();

        let expected = Get {
            entries: vec![
                GetEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString("Air Force 1 Retro Boost".into())),
                    }),
                    value: Some(nike_store_value.clone()),
                },
                GetEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString("Jordan".into())),
                    }),
                    value: Some(nike_store_value.clone()),
                },
            ],
        };

        for item in response.entries {
            assert!(expected.entries.contains(&item) == true)
        }
    }

    #[tokio::test]
    async fn test_ai_client_actions_on_binary_store() {
        let address = provision_test_servers().await;

        let ai_client = AiClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let store_name = StoreName {
            value: String::from("Deven Image Store"),
        };
        let matching_metadatakey = "Name".to_owned();

        let matching_metadatavalue = MetadataValue {
            value: Some(MValue::RawString("Daniel".into())),
        };

        let store_value_1 = StoreValue {
            value: HashMap::from_iter([(
                matching_metadatakey.clone(),
                matching_metadatavalue.clone(),
            )]),
        };
        let store_value_2 = StoreValue {
            value: HashMap::from_iter([(
                matching_metadatakey.clone(),
                MetadataValue {
                    value: Some(MValue::RawString("Deven".into())),
                },
            )]),
        };

        let store_data = vec![
            StoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Image(
                        include_bytes!("../../ai/src/tests/images/dog.jpg").to_vec(),
                    )),
                }),
                value: HashMap::from_iter([(
                    matching_metadatakey.clone(),
                    MetadataValue {
                        value: Some(MValue::RawString("Greatness".into())),
                    },
                )]),
            },
            StoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Image(
                        include_bytes!("../../ai/src/tests/images/test.webp").to_vec(),
                    )),
                }),
                value: store_value_2.clone().value,
            },
            StoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Image(
                        include_bytes!("../../ai/src/tests/images/cat.png").to_vec(),
                    )),
                }),
                value: store_value_1.clone().value,
            },
        ];

        let mut pipeline = ai_client.pipeline(None);

        let create_store_params = CreateStore {
            store: store_name.value.clone(),
            index_model: AiModel::Resnet50 as i32,
            query_model: AiModel::Resnet50 as i32,
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        };

        pipeline.create_store(create_store_params);
        pipeline.list_stores();

        let create_pred_index_params = CreatePredIndex {
            store: store_name.value.clone(),
            predicates: vec!["Name".into(), "Age".into()],
        };

        pipeline.create_pred_index(create_pred_index_params);

        let set_params = Set {
            store: store_name.value.clone(),
            inputs: store_data,
            execution_provider: None,
            preprocess_action: PreprocessAction::NoPreprocessing as i32,
        };

        pipeline.set(set_params);

        let drop_pred_index_params = DropPredIndex {
            store: store_name.value.clone(),
            predicates: vec!["Age".to_string()],
            error_if_not_exists: true,
        };

        pipeline.drop_pred_index(drop_pred_index_params);

        let condition = PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::Equals(predicates::Equals {
                    key: matching_metadatakey,
                    value: Some(matching_metadatavalue),
                })),
            })),
        };

        let get_pred_params = GetPred {
            store: store_name.value.clone(),
            condition: Some(condition),
        };

        pipeline.get_pred(get_pred_params);

        pipeline.purge_stores();

        let resnet_model: ModelDetails =
            SupportedModels::from(&AiModel::Resnet50).to_model_details();

        let expected = AiResponsePipeline {
            responses: vec![
                AiServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                AiServerResponse {
                    response: Some(Response::StoreList(StoreList {
                        stores: vec![AiStoreInfo {
                            name: store_name.value.clone(),
                            embedding_size: resnet_model.embedding_size.get() as u64,
                            query_model: AiModel::Resnet50 as i32,
                            index_model: AiModel::Resnet50 as i32,
                        }],
                    })),
                },
                AiServerResponse {
                    response: Some(Response::CreateIndex(CreateIndex { created_indexes: 2 })),
                },
                AiServerResponse {
                    response: Some(Response::Set(SetResult {
                        upsert: Some(StoreUpsert {
                            inserted: 3,
                            updated: 0,
                        }),
                    })),
                },
                AiServerResponse {
                    response: Some(Response::Del(Del { deleted_count: 1 })),
                },
                AiServerResponse {
                    response: Some(Response::Get(Get {
                        entries: vec![GetEntry {
                            key: Some(StoreInput {
                                value: Some(Value::Image(
                                    include_bytes!("../../ai/src/tests/images/cat.png").to_vec(),
                                )),
                            }),
                            value: Some(store_value_1.clone()),
                        }],
                    })),
                },
                AiServerResponse {
                    response: Some(Response::Del(Del { deleted_count: 1 })),
                },
            ],
        };

        let res = pipeline.exec().await.expect("Could not execute pipeline");

        assert_eq!(res, expected);
    }
}
