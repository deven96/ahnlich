use crate::conn::{AIConn, Connection};
use crate::error::AhnlichError;
use crate::prelude::*;
use deadpool::managed::Manager;
use deadpool::managed::Metrics;
use deadpool::managed::Object;
use deadpool::managed::Pool;
use deadpool::managed::RecycleError;
use deadpool::managed::RecycleResult;
use std::collections::HashSet;
use std::num::NonZeroUsize;

/// TCP Connection manager to ahnlich db
#[derive(Debug)]
pub struct AIConnManager {
    host: String,
    port: u16,
}

impl AIConnManager {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

#[async_trait::async_trait]
impl Manager for AIConnManager {
    type Type = AIConn;
    type Error = AhnlichError;

    async fn create(&self) -> Result<AIConn, AhnlichError> {
        AIConn::new(&self.host, self.port).await
    }

    async fn recycle(&self, conn: &mut AIConn, _metrics: &Metrics) -> RecycleResult<AhnlichError> {
        conn.is_conn_valid().await.map_err(RecycleError::Backend)
    }
}

/// Allow executing multiple queries at once
#[derive(Debug)]
pub struct AIPipeline {
    queries: AIServerQuery,
    conn: Object<AIConnManager>,
}

impl AIPipeline {
    pub fn new_from_queries_and_conn(queries: AIServerQuery, conn: Object<AIConnManager>) -> Self {
        Self { queries, conn }
    }
    /// push create store command to pipeline
    pub fn create_store(
        &mut self,
        store: StoreName,
        query_model: AIModel,
        index_model: AIModel,
        predicates: HashSet<MetadataKey>,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
    ) {
        self.queries.push(AIQuery::CreateStore {
            store,
            query_model,
            index_model,
            predicates,
            non_linear_indices,
        })
    }

    /// Push get pred command to pipeline
    pub fn get_pred(&mut self, store: StoreName, condition: PredicateCondition) {
        self.queries.push(AIQuery::GetPred { store, condition })
    }

    /// Push get sim n command to pipeline
    pub fn get_sim_n(
        &mut self,
        store: StoreName,
        search_input: StoreInput,
        condition: Option<PredicateCondition>,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
    ) {
        self.queries.push(AIQuery::GetSimN {
            store,
            search_input,
            condition,
            closest_n,
            algorithm,
        })
    }

    /// push create pred index command to pipeline
    pub fn create_pred_index(&mut self, store: StoreName, predicates: HashSet<MetadataKey>) {
        self.queries
            .push(AIQuery::CreatePredIndex { store, predicates })
    }

    /// push create non linear index command to pipeline
    pub fn create_non_linear_algorithm_index(
        &mut self,
        store: StoreName,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
    ) {
        self.queries.push(AIQuery::CreateNonLinearAlgorithmIndex {
            store,
            non_linear_indices,
        })
    }

    /// push drop pred index command to pipeline
    pub fn drop_pred_index(
        &mut self,
        store: StoreName,
        predicates: HashSet<MetadataKey>,
        error_if_not_exists: bool,
    ) {
        self.queries.push(AIQuery::DropPredIndex {
            store,
            predicates,
            error_if_not_exists,
        })
    }

    /// push set command to pipeline
    pub fn set(
        &mut self,
        store: StoreName,
        inputs: Vec<(StoreInput, StoreValue)>,
        preprocess_action: PreprocessAction,
    ) {
        self.queries.push(AIQuery::Set {
            store,
            inputs,
            preprocess_action,
        })
    }

    /// push del key command to pipeline
    pub fn del_key(&mut self, store: StoreName, key: StoreInput) {
        self.queries.push(AIQuery::DelKey { store, key })
    }

    /// Push drop store command to pipeline
    pub fn drop_store(&mut self, store: StoreName, error_if_not_exists: bool) {
        self.queries.push(AIQuery::DropStore {
            store,
            error_if_not_exists,
        })
    }

    /// Push info server command to pipeline
    pub fn info_server(&mut self) {
        self.queries.push(AIQuery::InfoServer)
    }

    /// Push list stores command to pipeline
    pub fn list_stores(&mut self) {
        self.queries.push(AIQuery::ListStores)
    }

    /// Push purge stores command to pipeline
    pub fn purge_stores(&mut self) {
        self.queries.push(AIQuery::PurgeStores)
    }

    /// Push ping command to pipeline
    pub fn ping(&mut self) {
        self.queries.push(AIQuery::Ping)
    }

    /// execute queries all at once and return ordered list of results matching the order in which
    /// queries were pushed
    pub async fn exec(mut self) -> Result<AIServerResult, AhnlichError> {
        self.conn.send_query(self.queries).await
    }
}

/// Client for Ahnlich AI using an instantiated deadpool pool
#[derive(Debug)]
pub struct AIClient {
    pool: Pool<AIConnManager>,
}

impl AIClient {
    pub async fn new(host: String, port: u16) -> Result<Self, AhnlichError> {
        let manager = AIConnManager::new(host, port);
        let pool = Pool::builder(manager).build()?;
        Ok(Self { pool })
    }

    /// Create new ai client with custom deadpool pool
    pub fn new_with_pool(pool: Pool<AIConnManager>) -> Self {
        Self { pool }
    }

    /// Instantiate a new pipeline with a given capacity. Runs commands sequentially on
    /// `pipeline.exec`
    pub async fn pipeline(
        &self,
        capacity: usize,
        tracing_id: Option<String>,
    ) -> Result<AIPipeline, AhnlichError> {
        Ok(AIPipeline::new_from_queries_and_conn(
            AIServerQuery::with_capacity_and_tracing_id(capacity, tracing_id),
            self.pool.get().await?,
        ))
    }

    pub async fn create_store(
        &self,
        store: StoreName,
        query_model: AIModel,
        index_model: AIModel,
        predicates: HashSet<MetadataKey>,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(
            AIQuery::CreateStore {
                store,
                query_model,
                index_model,
                predicates,
                non_linear_indices,
            },
            tracing_id,
        )
        .await
    }

    pub async fn get_pred(
        &self,
        store: StoreName,
        condition: PredicateCondition,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::GetPred { store, condition }, tracing_id)
            .await
    }

    pub async fn get_sim_n(
        &self,
        store: StoreName,
        search_input: StoreInput,
        condition: Option<PredicateCondition>,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(
            AIQuery::GetSimN {
                store,
                search_input,
                condition,
                closest_n,
                algorithm,
            },
            tracing_id,
        )
        .await
    }

    pub async fn create_pred_index(
        &self,
        store: StoreName,
        predicates: HashSet<MetadataKey>,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::CreatePredIndex { store, predicates }, tracing_id)
            .await
    }

    pub async fn create_non_linear_algorithm_index(
        &self,
        store: StoreName,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(
            AIQuery::CreateNonLinearAlgorithmIndex {
                store,
                non_linear_indices,
            },
            tracing_id,
        )
        .await
    }

    pub async fn drop_pred_index(
        &self,
        store: StoreName,
        predicates: HashSet<MetadataKey>,
        error_if_not_exists: bool,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(
            AIQuery::DropPredIndex {
                store,
                predicates,
                error_if_not_exists,
            },
            tracing_id,
        )
        .await
    }

    pub async fn set(
        &self,
        store: StoreName,
        inputs: Vec<(StoreInput, StoreValue)>,
        preprocess_action: PreprocessAction,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(
            AIQuery::Set {
                store,
                inputs,
                preprocess_action,
            },
            tracing_id,
        )
        .await
    }

    pub async fn del_key(
        &self,
        store: StoreName,
        key: StoreInput,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::DelKey { store, key }, tracing_id).await
    }

    pub async fn drop_store(
        &self,
        store: StoreName,
        error_if_not_exists: bool,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(
            AIQuery::DropStore {
                store,
                error_if_not_exists,
            },
            tracing_id,
        )
        .await
    }

    pub async fn info_server(
        &self,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::InfoServer, tracing_id).await
    }

    pub async fn list_stores(
        &self,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::ListStores, tracing_id).await
    }

    pub async fn purge_stores(
        &self,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::PurgeStores, tracing_id).await
    }

    pub async fn ping(&self, tracing_id: Option<String>) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::Ping, tracing_id).await
    }

    async fn exec(
        &self,
        query: AIQuery,
        tracing_id: Option<String>,
    ) -> Result<AIServerResponse, AhnlichError> {
        let mut conn = self.pool.get().await?;

        let mut queries = AIServerQuery::with_capacity_and_tracing_id(1, tracing_id);
        queries.push(query);

        let res = conn
            .send_query(queries)
            .await?
            .pop()
            .transpose()
            .map_err(AhnlichError::AIProxyError)?;
        res.ok_or(AhnlichError::EmptyResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahnlich_ai_proxy::cli::AIProxyConfig;
    use ahnlich_ai_proxy::{engine::ai::models::Model, server::handler::AIProxyServer};
    use ahnlich_db::cli::ServerConfig;
    use ahnlich_db::server::handler::Server;
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
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
        pipeline.create_store(
            StoreName("Main".to_string()),
            AIModel::AllMiniLML6V2,
            AIModel::AllMiniLML6V2,
            HashSet::new(),
            HashSet::new(),
        );
        pipeline.create_store(
            StoreName("Main".to_string()),
            AIModel::AllMiniLML6V2,
            AIModel::AllMiniLML6V2,
            HashSet::new(),
            HashSet::new(),
        );
        pipeline.create_store(
            StoreName("Less".to_string()),
            AIModel::AllMiniLML6V2,
            AIModel::AllMiniLML6V2,
            HashSet::new(),
            HashSet::new(),
        );
        pipeline.list_stores();
        let mut expected = AIServerResult::with_capacity(4);
        expected.push(Ok(AIServerResponse::Unit));
        expected.push(Err("Store Main already exists".to_string()));
        expected.push(Ok(AIServerResponse::Unit));
        let ai_model: Model = (&AIModel::AllMiniLML6V2).into();
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

        assert!(ai_client
            .create_store(
                store_name.clone(),
                AIModel::AllMiniLML6V2,
                AIModel::AllMiniLML6V2,
                HashSet::new(),
                HashSet::new(),
                None
            )
            .await
            .is_ok());

        assert!(ai_client
            .set(
                store_name.clone(),
                vec![
                    (StoreInput::RawString("Adidas Yeezy".into()), HashMap::new()),
                    (
                        StoreInput::RawString("Nike Air Jordans".into()),
                        HashMap::new()
                    ),
                ],
                PreprocessAction::RawString(StringAction::ErrorIfTokensExceed),
                None
            )
            .await
            .is_ok());

        assert_eq!(
            ai_client
                .del_key(
                    store_name,
                    StoreInput::RawString("Adidas Yeezy".into()),
                    None
                )
                .await
                .unwrap(),
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
        pipeline.create_store(
            StoreName("Main".to_string()),
            AIModel::AllMiniLML6V2,
            AIModel::AllMiniLML6V2,
            HashSet::new(),
            HashSet::new(),
        );
        pipeline.create_store(
            StoreName("Main2".to_string()),
            AIModel::AllMiniLML6V2,
            AIModel::AllMiniLML6V2,
            HashSet::new(),
            HashSet::new(),
        );
        pipeline.create_store(
            StoreName("Less".to_string()),
            AIModel::AllMiniLML6V2,
            AIModel::AllMiniLML6V2,
            HashSet::new(),
            HashSet::new(),
        );
        pipeline.list_stores();
        pipeline.drop_store(StoreName("Less".to_string()), true);
        pipeline.purge_stores();
        let mut expected = AIServerResult::with_capacity(6);
        expected.push(Ok(AIServerResponse::Unit));
        expected.push(Ok(AIServerResponse::Unit));
        expected.push(Ok(AIServerResponse::Unit));

        let ai_model: Model = (&AIModel::AllMiniLML6V2).into();
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

        pipeline.create_store(
            store_name.clone(),
            AIModel::AllMiniLML6V2,
            AIModel::AllMiniLML6V2,
            HashSet::new(),
            HashSet::new(),
        );
        pipeline.list_stores();
        pipeline.create_pred_index(
            store_name.clone(),
            HashSet::from_iter([
                MetadataKey::new("Brand".to_string()),
                MetadataKey::new("Vintage".to_string()),
            ]),
        );
        pipeline.set(
            store_name.clone(),
            store_data,
            PreprocessAction::RawString(StringAction::ErrorIfTokensExceed),
        );

        pipeline.drop_pred_index(
            store_name.clone(),
            HashSet::from_iter([MetadataKey::new("Vintage".to_string())]),
            true,
        );
        let res = pipeline.exec().await.expect("Could not execute pipeline");

        let mut expected = AIServerResult::with_capacity(6);

        expected.push(Ok(AIServerResponse::Unit));
        let ai_model: Model = (&AIModel::AllMiniLML6V2).into();
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

        let response = ai_client
            .get_pred(
                store_name,
                PredicateCondition::Value(Predicate::Equals {
                    key: matching_metadatakey,
                    value: matching_metadatavalue,
                }),
                None,
            )
            .await
            .unwrap();

        let expected = vec![
            (
                StoreInput::RawString(String::from("Air Force 1 Retro Boost")),
                nike_store_value.clone(),
            ),
            (
                StoreInput::RawString(String::from("Jordan")),
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

        pipeline.create_store(
            store_name.clone(),
            AIModel::Resnet50,
            AIModel::Resnet50,
            HashSet::new(),
            HashSet::new(),
        );
        pipeline.list_stores();
        pipeline.create_pred_index(
            store_name.clone(),
            HashSet::from_iter([
                MetadataKey::new("Name".to_string()),
                MetadataKey::new("Age".to_string()),
            ]),
        );
        pipeline.set(
            store_name.clone(),
            store_data,
            PreprocessAction::Image(ImageAction::ErrorIfDimensionsMismatch),
        );

        pipeline.drop_pred_index(
            store_name.clone(),
            HashSet::from_iter([MetadataKey::new("Age".to_string())]),
            true,
        );
        pipeline.get_pred(
            store_name.clone(),
            PredicateCondition::Value(Predicate::Equals {
                key: matching_metadatakey.clone(),
                value: matching_metadatavalue,
            }),
        );

        pipeline.purge_stores();

        let mut expected = AIServerResult::with_capacity(7);

        expected.push(Ok(AIServerResponse::Unit));
        let resnet_model: Model = (&AIModel::Resnet50).into();
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
            StoreInput::Image(include_bytes!("../../ai/src/tests/images/cat.png").to_vec()),
            store_value_1.clone(),
        )])));
        expected.push(Ok(AIServerResponse::Del(1)));

        let res = pipeline.exec().await.expect("Could not execute pipeline");

        assert_eq!(res, expected);
    }
}
