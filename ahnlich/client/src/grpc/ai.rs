use grpc_types::{
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
    pub async fn create_store(&mut self, params: CreateStore) {
        self.queries.push(Query::CreateStore(params));
    }

    pub async fn create_pred_index(&mut self, params: CreatePredIndex) {
        self.queries.push(Query::CreatePredIndex(params));
    }

    pub async fn create_non_linear_algorithm_index(
        &mut self,
        params: CreateNonLinearAlgorithmIndex,
    ) {
        self.queries
            .push(Query::CreateNonLinearAlgorithmIndex(params));
    }

    pub async fn get_key(&mut self, params: GetKey) {
        self.queries.push(Query::GetKey(params));
    }

    pub async fn get_pred(&mut self, params: GetPred) {
        self.queries.push(Query::GetPred(params));
    }

    pub async fn get_sim_n(&mut self, params: GetSimN) {
        self.queries.push(Query::GetSimN(params));
    }

    pub async fn set(&mut self, params: Set) {
        self.queries.push(Query::Set(params));
    }

    pub async fn drop_pred_index(&mut self, params: DropPredIndex) {
        self.queries.push(Query::DropPredIndex(params));
    }

    pub async fn drop_non_linear_algorithm_index(&mut self, params: DropNonLinearAlgorithmIndex) {
        self.queries
            .push(Query::DropNonLinearAlgorithmIndex(params));
    }

    pub async fn del_key(&mut self, params: DelKey) {
        self.queries.push(Query::DelKey(params));
    }

    pub async fn drop_store(&mut self, params: DropStore) {
        self.queries.push(Query::DropStore(params));
    }

    pub async fn info_server(&mut self) {
        self.queries.push(Query::InfoServer(InfoServer {}));
    }

    pub async fn purge_stores(&mut self) {
        self.queries.push(Query::PurgeStores(PurgeStores {}));
    }

    pub async fn list_stores(&mut self) {
        self.queries.push(Query::ListStores(ListStores {}));
    }

    pub async fn list_clients(&mut self) {
        self.queries.push(Query::ListClients(ListClients {}));
    }

    pub async fn ping(&mut self) {
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
    use std::{
        collections::{HashMap, HashSet},
        time::Duration,
    };

    use super::*;
    use ahnlich_db::{cli::ServerConfig, server::handler::Server};
    use ahnlich_types::{
        db::ServerResponse,
        keyval::StoreKey,
        metadata::MetadataKey,
        predicate::{Predicate, PredicateCondition},
        similarity::Algorithm,
    };
    use grpc_types::{
        db::{
            pipeline::{db_server_response::Response, DbServerResponse},
            query::CreateStore,
            server::{GetSimNEntry, StoreInfo},
        },
        metadata::metadata_value::Value,
        shared::info::ErrorResponse,
        similarity::Similarity,
    };
    use once_cell::sync::Lazy;
    use utils::server::AhnlichServerUtils;

    static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());

    #[tokio::test]
    async fn test_grpc_create_store_with_pipeline() {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let address = server.local_addr().expect("Could not get local addr");
        tokio::spawn(async move {
            server.start().await;
        });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let db_client = DbClient::new(address.to_string())
            .await
            .expect("Could not initialize client");
        let mut pipeline = db_client.pipeline(None);
        pipeline.create_store(CreateStore {
            store: "Main".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
        });
        pipeline.create_store(CreateStore {
            store: "Main".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
        });
        pipeline.create_store(CreateStore {
            store: "Main".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: false,
        });
        pipeline.list_stores();

        let expected = DbResponsePipeline {
            responses: vec![
                DbServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                DbServerResponse {
                    response: Some(Response::Error(ErrorResponse {
                        message: "Store Main already exists".to_string(),
                        code: 20,
                    })),
                },
                DbServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                DbServerResponse {
                    response: Some(Response::StoreList(StoreList {
                        stores: vec![StoreInfo {
                            name: "Main".to_string(),
                            len: 0,
                            size_in_bytes: 1720,
                        }],
                    })),
                },
            ],
        };
        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_grpc_get_sim_n() {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let address = server.local_addr().expect("Could not get local addr");
        tokio::spawn(async move {
            server.start().await;
        });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let mut db_client = DbClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let create_store_params = db_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .dimension(3)
            .create_predicates(HashSet::from_iter([MetadataKey::new("medal".into())]))
            .build();

        assert!(db_client.create_store(create_store_params).await.is_ok());

        let set_key_params = db_params::SetParams::builder()
            .store("Main".to_string())
            .inputs(vec![
                (
                    StoreKey(vec![1.2, 1.3, 1.4]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::RawString("silver".into()),
                    )]),
                ),
                (
                    StoreKey(vec![2.0, 2.1, 2.2]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::RawString("gold".into()),
                    )]),
                ),
                (
                    StoreKey(vec![5.0, 5.1, 5.2]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::RawString("bronze".into()),
                    )]),
                ),
            ])
            .build();
        assert!(db_client.set(set_key_params).await.is_ok());
        // error due to dimension mismatch
        let get_sim_n_params = db_params::GetSimNParams::builder()
            .store("Main".to_string())
            .search_input(StoreKey(vec![1.1, 2.0]))
            .closest_n(2)
            .algorithm(Algorithm::EuclideanDistance)
            .build();
        assert!(db_client.get_sim_n(get_sim_n_params).await.is_err());

        let get_sim_n_params = db_params::GetSimNParams::builder()
            .store("Main".to_string())
            .search_input(StoreKey(vec![5.0, 2.1, 2.2]))
            .closest_n(2)
            .algorithm(Algorithm::CosineSimilarity)
            .condition(Some(PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("medal".into()),
                value: MetadataValue::RawString("gold".into()),
            })))
            .build();

        assert_eq!(
            db_client.get_sim_n(get_sim_n_params).await.unwrap(),
            GetSimNResult {
                entries: vec![GetSimNEntry {
                    key: Some(grpc_types::keyval::StoreKey {
                        key: vec![2.0, 2.1, 2.2]
                    }),
                    value: Some(grpc_types::keyval::StoreValue {
                        value: HashMap::from_iter([(
                            "medal".into(),
                            grpc_types::metadata::MetadataValue {
                                value: Some(Value::RawString("gold".into()))
                            },
                        )])
                    }),
                    similarity: Some(Similarity {
                        value: 0.9036338825194858
                    })
                }]
            }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahnlich_ai_proxy::cli::server::SupportedModels;
    use ahnlich_ai_proxy::cli::AIProxyConfig;
    use ahnlich_ai_proxy::engine::ai::models::ModelDetails;
    use ahnlich_ai_proxy::server::handler::AIProxyServer;
    use ahnlich_db::cli::ServerConfig;
    use ahnlich_db::server::handler::Server;
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::net::SocketAddr;
    use tokio::time::Duration;
    use utils::server::AhnlichServerUtils;

    static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default());
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
        let host = address.ip();
        let port = address.port();
        let ai_client = AIClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        assert!(ai_client.ping(None).await.is_ok());
    }

    #[tokio::test]
    async fn test_ai_client_simple_pipeline() {
        let address = provision_test_servers().await;
        let host = address.ip();
        let port = address.port();
        let ai_client = AIClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        let mut pipeline = ai_client
            .pipeline(3, None)
            .await
            .expect("Could not create pipeline");
        pipeline.list_stores();
        pipeline.ping();
        let mut expected = AIServerResult::with_capacity(2);
        expected.push(Ok(AIServerResponse::StoreList(HashSet::new())));
        expected.push(Ok(AIServerResponse::Pong));
        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_pool_commands_fail_if_server_not_exist() {
        let host = "127.0.0.1";
        let port = 1234;
        let ai_client = AIClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        assert!(ai_client.ping(None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_stores_with_pipeline() {
        let address = provision_test_servers().await;
        let host = address.ip();
        let port = address.port();
        let ai_client = AIClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");

        let mut pipeline = ai_client
            .pipeline(4, None)
            .await
            .expect("Could not create pipeline");
        let create_store_params = ai_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .index_model(AIModel::AllMiniLML6V2)
            .query_model(AIModel::AllMiniLML6V2)
            .build();

        let create_store_params_2 = ai_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .index_model(AIModel::AllMiniLML6V2)
            .query_model(AIModel::AllMiniLML6V2)
            .build();

        let create_store_params_no_error = ai_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .index_model(AIModel::AllMiniLML6V2)
            .query_model(AIModel::AllMiniLML6V2)
            .error_if_exists(false)
            .build();
        pipeline.create_store(create_store_params);
        pipeline.create_store(create_store_params_2);
        pipeline.create_store(create_store_params_no_error);

        let create_store_params = ai_params::CreateStoreParams::builder()
            .store("Less".to_string())
            .index_model(AIModel::AllMiniLML6V2)
            .query_model(AIModel::AllMiniLML6V2)
            .build();

        pipeline.create_store(create_store_params);
        pipeline.list_stores();
        let mut expected = AIServerResult::with_capacity(5);
        expected.push(Ok(AIServerResponse::Unit));
        expected.push(Err("Store Main already exists".to_string()));
        expected.push(Ok(AIServerResponse::Unit));
        expected.push(Ok(AIServerResponse::Unit));
        let ai_model: ModelDetails =
            SupportedModels::from(&AIModel::AllMiniLML6V2).to_model_details();
        expected.push(Ok(AIServerResponse::StoreList(HashSet::from_iter([
            AIStoreInfo {
                name: StoreName("Main".to_string()),
                embedding_size: ai_model.embedding_size.into(),
                query_model: AIModel::AllMiniLML6V2,
                index_model: AIModel::AllMiniLML6V2,
            },
            AIStoreInfo {
                name: StoreName("Less".to_string()),
                embedding_size: ai_model.embedding_size.into(),
                query_model: AIModel::AllMiniLML6V2,
                index_model: AIModel::AllMiniLML6V2,
            },
        ]))));
        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_del_key() {
        let address = provision_test_servers().await;
        let host = address.ip();
        let port = address.port();
        let ai_client = AIClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        let store_name = StoreName("Main".to_string());

        let create_store_params = ai_params::CreateStoreParams::builder()
            .store(store_name.clone().to_string())
            .index_model(AIModel::AllMiniLML6V2)
            .query_model(AIModel::AllMiniLML6V2)
            .build();

        assert!(ai_client.create_store(create_store_params).await.is_ok());

        let set_params = ai_params::SetParams::builder()
            .store(store_name.clone().to_string())
            .inputs(vec![
                (StoreInput::RawString("Adidas Yeezy".into()), HashMap::new()),
                (
                    StoreInput::RawString("Nike Air Jordans".into()),
                    HashMap::new(),
                ),
            ])
            .preprocess_action(PreprocessAction::NoPreprocessing)
            .build();

        assert!(ai_client.set(set_params).await.is_ok());

        let delete_key = ai_params::DelKeyParams::builder()
            .store(store_name.to_string())
            .key(StoreInput::RawString("Adidas Yeezy".into()))
            .build();
        assert_eq!(
            ai_client.del_key(delete_key).await.unwrap(),
            AIServerResponse::Del(1)
        )
    }

    #[tokio::test]
    async fn test_destroy_purge_stores_with_pipeline() {
        let address = provision_test_servers().await;
        let host = address.ip();
        let port = address.port();
        let ai_client = AIClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");

        let mut pipeline = ai_client
            .pipeline(4, None)
            .await
            .expect("Could not create pipeline");

        let create_store_params = ai_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .index_model(AIModel::AllMiniLML6V2)
            .query_model(AIModel::AllMiniLML6V2)
            .build();
        pipeline.create_store(create_store_params);
        let create_store_params_2 = ai_params::CreateStoreParams::builder()
            .store("Main2".to_string())
            .index_model(AIModel::AllMiniLML6V2)
            .query_model(AIModel::AllMiniLML6V2)
            .build();
        pipeline.create_store(create_store_params_2);

        let create_store_params_3 = ai_params::CreateStoreParams::builder()
            .store("Less".to_string())
            .index_model(AIModel::AllMiniLML6V2)
            .query_model(AIModel::AllMiniLML6V2)
            .build();

        pipeline.create_store(create_store_params_3);
        pipeline.list_stores();
        let drop_store_params = ai_params::DropStoreParams::builder()
            .store("Less".to_string())
            .build();
        pipeline.drop_store(drop_store_params);
        pipeline.purge_stores();
        let mut expected = AIServerResult::with_capacity(6);
        expected.push(Ok(AIServerResponse::Unit));
        expected.push(Ok(AIServerResponse::Unit));
        expected.push(Ok(AIServerResponse::Unit));

        let ai_model: ModelDetails =
            SupportedModels::from(&AIModel::AllMiniLML6V2).to_model_details();
        expected.push(Ok(AIServerResponse::StoreList(HashSet::from_iter([
            AIStoreInfo {
                name: StoreName("Main".to_string()),
                embedding_size: ai_model.embedding_size.into(),
                query_model: AIModel::AllMiniLML6V2,
                index_model: AIModel::AllMiniLML6V2,
            },
            AIStoreInfo {
                name: StoreName("Main2".to_string()),
                embedding_size: ai_model.embedding_size.into(),
                query_model: AIModel::AllMiniLML6V2,
                index_model: AIModel::AllMiniLML6V2,
            },
            AIStoreInfo {
                name: StoreName("Less".to_string()),
                embedding_size: ai_model.embedding_size.into(),
                query_model: AIModel::AllMiniLML6V2,
                index_model: AIModel::AllMiniLML6V2,
            },
        ]))));
        expected.push(Ok(AIServerResponse::Del(1)));
        expected.push(Ok(AIServerResponse::Del(2)));
        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_ai_client_get_pred() {
        let address = provision_test_servers().await;

        let host = address.ip();
        let port = address.port();
        let ai_client = AIClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");

        let store_name = StoreName(String::from("Deven Kicks"));
        let matching_metadatakey = MetadataKey::new("Brand".to_owned());
        let matching_metadatavalue = MetadataValue::RawString("Nike".to_owned());

        let nike_store_value =
            StoreValue::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]);
        let adidas_store_value = StoreValue::from_iter([(
            matching_metadatakey.clone(),
            MetadataValue::RawString("Adidas".to_owned()),
        )]);
        let store_data = vec![
            (
                StoreInput::RawString(String::from("Air Force 1 Retro Boost")),
                nike_store_value.clone(),
            ),
            (
                StoreInput::RawString(String::from("Jordan")),
                nike_store_value.clone(),
            ),
            (
                StoreInput::RawString(String::from("Yeezy")),
                adidas_store_value.clone(),
            ),
        ];

        let mut pipeline = ai_client
            .pipeline(6, None)
            .await
            .expect("Could not create pipeline");

        let create_store_params = ai_params::CreateStoreParams::builder()
            .store(store_name.clone().to_string())
            .index_model(AIModel::AllMiniLML6V2)
            .query_model(AIModel::AllMiniLML6V2)
            .build();

        pipeline.create_store(create_store_params);
        pipeline.list_stores();
        let create_pred_index_params = ai_params::CreatePredIndexParams::builder()
            .store(store_name.clone().to_string())
            .predicates(HashSet::from_iter([
                MetadataKey::new("Brand".to_string()),
                MetadataKey::new("Vintage".to_string()),
            ]))
            .build();
        pipeline.create_pred_index(create_pred_index_params);

        let set_params = ai_params::SetParams::builder()
            .store(store_name.clone().to_string())
            .inputs(store_data)
            .preprocess_action(PreprocessAction::NoPreprocessing)
            .build();
        pipeline.set(set_params);

        let drop_pred_params = ai_params::DropPredIndexParams::builder()
            .store(store_name.clone().to_string())
            .predicates(HashSet::from_iter([MetadataKey::new(
                "Vintage".to_string(),
            )]))
            .build();

        pipeline.drop_pred_index(drop_pred_params);
        let res = pipeline.exec().await.expect("Could not execute pipeline");

        let mut expected = AIServerResult::with_capacity(6);

        expected.push(Ok(AIServerResponse::Unit));
        let ai_model: ModelDetails =
            SupportedModels::from(&AIModel::AllMiniLML6V2).to_model_details();
        expected.push(Ok(AIServerResponse::StoreList(HashSet::from_iter([
            AIStoreInfo {
                name: store_name.clone(),
                query_model: AIModel::AllMiniLML6V2,
                index_model: AIModel::AllMiniLML6V2,

                embedding_size: ai_model.embedding_size.into(),
            },
        ]))));
        expected.push(Ok(AIServerResponse::CreateIndex(2)));
        expected.push(Ok(AIServerResponse::Set(StoreUpsert {
            inserted: 3,
            updated: 0,
        })));
        expected.push(Ok(AIServerResponse::Del(1)));

        assert_eq!(res, expected);

        let get_pred_params = ai_params::GetPredParams::builder()
            .store(store_name.to_string())
            .condition(PredicateCondition::Value(Predicate::Equals {
                key: matching_metadatakey,
                value: matching_metadatavalue,
            }))
            .build();

        let response = ai_client.get_pred(get_pred_params).await.unwrap();

        let expected = vec![
            (
                Some(StoreInput::RawString(String::from(
                    "Air Force 1 Retro Boost",
                ))),
                nike_store_value.clone(),
            ),
            (
                Some(StoreInput::RawString(String::from("Jordan"))),
                nike_store_value.clone(),
            ),
        ];
        if let AIServerResponse::Get(get_pred_result) = response {
            for item in get_pred_result {
                assert!(expected.contains(&item) == true);
            }
        }
    }

    #[tokio::test]
    async fn test_ai_client_actions_on_binary_store() {
        let address = provision_test_servers().await;

        let host = address.ip();
        let port = address.port();
        let ai_client = AIClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");

        let store_name = StoreName(String::from("Deven Image Store"));
        let matching_metadatakey = MetadataKey::new("Name".to_owned());
        let matching_metadatavalue = MetadataValue::RawString("Daniel".to_owned());

        let store_value_1 =
            StoreValue::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]);
        let store_value_2 = StoreValue::from_iter([(
            matching_metadatakey.clone(),
            MetadataValue::RawString("Deven".to_owned()),
        )]);
        let store_data = vec![
            (
                StoreInput::Image(include_bytes!("../../ai/src/tests/images/dog.jpg").to_vec()),
                StoreValue::from_iter([(
                    matching_metadatakey.clone(),
                    MetadataValue::RawString("Greatness".to_owned()),
                )]),
            ),
            (
                StoreInput::Image(include_bytes!("../../ai/src/tests/images/test.webp").to_vec()),
                store_value_2.clone(),
            ),
            (
                StoreInput::Image(include_bytes!("../../ai/src/tests/images/cat.png").to_vec()),
                store_value_1.clone(),
            ),
        ];

        let mut pipeline = ai_client
            .pipeline(7, None)
            .await
            .expect("Could not create pipeline");

        let create_store_params = ai_params::CreateStoreParams::builder()
            .store(store_name.clone().to_string())
            .index_model(AIModel::Resnet50)
            .query_model(AIModel::Resnet50)
            .build();

        pipeline.create_store(create_store_params);
        pipeline.list_stores();
        let create_pred_index_params = ai_params::CreatePredIndexParams::builder()
            .store(store_name.clone().to_string())
            .predicates(HashSet::from_iter([
                MetadataKey::new("Name".to_string()),
                MetadataKey::new("Age".to_string()),
            ]))
            .build();
        pipeline.create_pred_index(create_pred_index_params);
        let set_params = ai_params::SetParams::builder()
            .store(store_name.clone().to_string())
            .inputs(store_data)
            .preprocess_action(PreprocessAction::NoPreprocessing)
            .build();
        pipeline.set(set_params);

        let drop_pred_index_params = ai_params::DropPredIndexParams::builder()
            .store(store_name.clone().to_string())
            .predicates(HashSet::from_iter([MetadataKey::new("Age".to_string())]))
            .build();
        pipeline.drop_pred_index(drop_pred_index_params);

        let get_pred_params = ai_params::GetPredParams::builder()
            .store(store_name.clone().to_string())
            .condition(PredicateCondition::Value(Predicate::Equals {
                key: matching_metadatakey.clone(),
                value: matching_metadatavalue,
            }))
            .build();
        pipeline.get_pred(get_pred_params);

        pipeline.purge_stores();

        let mut expected = AIServerResult::with_capacity(7);

        expected.push(Ok(AIServerResponse::Unit));
        let resnet_model: ModelDetails =
            SupportedModels::from(&AIModel::Resnet50).to_model_details();
        expected.push(Ok(AIServerResponse::StoreList(HashSet::from_iter([
            AIStoreInfo {
                name: store_name,
                query_model: AIModel::Resnet50,
                index_model: AIModel::Resnet50,
                embedding_size: resnet_model.embedding_size.into(),
            },
        ]))));
        expected.push(Ok(AIServerResponse::CreateIndex(2)));
        expected.push(Ok(AIServerResponse::Set(StoreUpsert {
            inserted: 3,
            updated: 0,
        })));
        expected.push(Ok(AIServerResponse::Del(1)));
        expected.push(Ok(AIServerResponse::Get(vec![(
            Some(StoreInput::Image(
                include_bytes!("../../ai/src/tests/images/cat.png").to_vec(),
            )),
            store_value_1.clone(),
        )])));
        expected.push(Ok(AIServerResponse::Del(1)));

        let res = pipeline.exec().await.expect("Could not execute pipeline");

        assert_eq!(res, expected);
    }
}
