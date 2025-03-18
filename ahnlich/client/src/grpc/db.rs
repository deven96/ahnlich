use grpc_types::{
    db::{
        pipeline::{db_query::Query, DbQuery, DbRequestPipeline, DbResponsePipeline},
        query::{
            CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey, DelPred,
            DropNonLinearAlgorithmIndex, DropPredIndex, DropStore, GetKey, GetPred, GetSimN,
            InfoServer, ListClients, ListStores, Ping, Set, StoreEntry,
        },
        server::{
            ClientList, CreateIndex, Del, Get, GetSimN as GetSimNResult, Pong, Set as SetResult,
            StoreList, Unit,
        },
    },
    services::db_service::db_service_client::DbServiceClient,
    shared::info::ServerInfo,
    utils::add_trace_parent,
};
use tonic::transport::Channel;

use crate::error::AhnlichError;

#[derive(Debug, Clone)]
pub struct DbPipeline {
    queries: Vec<Query>,
    tracing_id: Option<String>,
    client: DbServiceClient<Channel>,
}

impl DbPipeline {
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

    pub async fn del_pred(&mut self, params: DelPred) {
        self.queries.push(Query::DelPred(params));
    }

    pub async fn info_server(&mut self) {
        self.queries.push(Query::InfoServer(InfoServer {}));
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

    pub async fn exec(mut self) -> Result<DbResponsePipeline, AhnlichError> {
        let tracing_id = self.tracing_id.clone();
        let mut req = tonic::Request::new(DbRequestPipeline {
            queries: self
                .queries
                .into_iter()
                .map(|q| DbQuery { query: Some(q) })
                .collect(),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.pipeline(req).await?.into_inner())
    }
}

// GRPC Client for Ahnlich DB
//
// client needs &mut as it can only send one request in flight, hence is not thread safe to use,
// however `Channel` makes use of `tower_buffer::Buffer` underneath and hence DBClient is cheap
// to clone and is encouraged for use across multiple threads
// https://docs.rs/tonic/latest/tonic/transport/struct.Channel.html#multiplexing-requests
// So we clone client underneath with every call to create a threadsafe client
#[derive(Debug, Clone)]
pub struct DbClient {
    client: DbServiceClient<Channel>,
}

impl DbClient {
    pub async fn new(addr: String) -> Result<Self, AhnlichError> {
        let addr = if !(addr.starts_with("https://") || addr.starts_with("http://")) {
            format!("http://{addr}")
        } else {
            addr
        };
        let channel = Channel::from_shared(addr)?;
        let client = DbServiceClient::connect(channel).await?;
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

    pub async fn del_pred(
        &self,
        params: DelPred,
        tracing_id: Option<String>,
    ) -> Result<Del, AhnlichError> {
        let mut req = tonic::Request::new(params);
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().del_pred(req).await?.into_inner())
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

    pub async fn list_stores(&self, tracing_id: Option<String>) -> Result<StoreList, AhnlichError> {
        let mut req = tonic::Request::new(ListStores {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().list_stores(req).await?.into_inner())
    }

    pub async fn list_clients(
        &self,
        tracing_id: Option<String>,
    ) -> Result<ClientList, AhnlichError> {
        let mut req = tonic::Request::new(ListClients {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().list_clients(req).await?.into_inner())
    }

    pub async fn ping(&self, tracing_id: Option<String>) -> Result<Pong, AhnlichError> {
        let mut req = tonic::Request::new(Ping {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.clone().ping(req).await?.into_inner())
    }

    // Create list of instructions to execute in a pipeline loop
    // on the server end
    pub fn pipeline(&self, tracing_id: Option<String>) -> DbPipeline {
        DbPipeline {
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
                                r#type: MetadataType::RawString.into(),
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

    #[tokio::test]
    async fn test_simple_server_ping() {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let address = server.local_addr().expect("Could not get local addr");
        let host = address.ip();
        let port = address.port();
        let _ = tokio::spawn(async move { server.start().await });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let db_client = DbClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        assert!(db_client.ping(None).await.is_ok());
    }

    #[tokio::test]
    async fn test_simple_pipeline() {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let address = server.local_addr().expect("Could not get local addr");
        let host = address.ip();
        let port = address.port();
        tokio::spawn(async { server.start().await });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let db_client = DbClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        let mut pipeline = db_client
            .pipeline(3, None)
            .await
            .expect("Could not create pipeline");
        pipeline.list_stores();
        pipeline.ping();
        let mut expected = ServerResult::with_capacity(2);
        expected.push(Ok(ServerResponse::StoreList(HashSet::new())));
        expected.push(Ok(ServerResponse::Pong));
        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_pool_commands_fail_if_server_not_exist() {
        let host = "127.0.0.1";
        let port = 1234;
        let db_client = DbClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        assert!(db_client.ping(None).await.is_err());
    }

    #[tokio::test]
    async fn test_create_stores_with_pipeline() {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let address = server.local_addr().expect("Could not get local addr");
        let _ = tokio::spawn(async move { server.start().await });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let host = address.ip();
        let port = address.port();
        let db_client = DbClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        let mut pipeline = db_client
            .pipeline(4, None)
            .await
            .expect("Could not create pipeline");

        let create_store_params = db_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .dimension(3)
            .build();
        let create_store_params_2 = db_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .dimension(3)
            .build();

        let create_store_params_no_error = db_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .dimension(3)
            .error_if_exists(false)
            .build();
        pipeline.create_store(create_store_params);
        pipeline.create_store(create_store_params_2);
        pipeline.create_store(create_store_params_no_error);
        pipeline.list_stores();
        let mut expected = ServerResult::with_capacity(4);
        expected.push(Ok(ServerResponse::Unit));
        expected.push(Err("Store Main already exists".to_string()));
        expected.push(Ok(ServerResponse::Unit));
        expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
            StoreInfo {
                name: StoreName("Main".to_string()),
                len: 0,
                size_in_bytes: 1720,
            },
        ]))));
        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_del_key() {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let address = server.local_addr().expect("Could not get local addr");
        let _ = tokio::spawn(async move { server.start().await });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let host = address.ip();
        let port = address.port();
        let db_client = DbClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        let del_key_params = db_params::DelKeyParams::builder()
            .store("Main".to_string())
            .keys(vec![])
            .build();
        assert!(db_client.del_key(del_key_params).await.is_err());

        let create_store_params = db_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .dimension(4)
            .create_predicates(HashSet::from_iter([MetadataKey::new("role".into())]))
            .non_linear_indices(HashSet::from_iter([NonLinearAlgorithm::KDTree]))
            .build();

        assert!(db_client.create_store(create_store_params).await.is_ok());
        let del_key_params = db_params::DelKeyParams::builder()
            .store("Main".to_string())
            .keys(vec![StoreKey(vec![1.0, 1.1, 1.2, 1.3])])
            .build();
        assert_eq!(
            db_client.del_key(del_key_params).await.unwrap(),
            ServerResponse::Del(0)
        );
        let set_key_params = db_params::SetParams::builder()
            .store("Main".to_string())
            .inputs(vec![
                (StoreKey(vec![1.0, 1.1, 1.2, 1.3]), HashMap::new()),
                (StoreKey(vec![1.1, 1.2, 1.3, 1.4]), HashMap::new()),
            ])
            .build();
        assert!(db_client.set(set_key_params).await.is_ok());
        assert_eq!(
            db_client.list_stores(None).await.unwrap(),
            ServerResponse::StoreList(HashSet::from_iter([StoreInfo {
                name: StoreName("Main".to_string()),
                len: 2,
                size_in_bytes: 2016,
            },]))
        );
        // error as different dimensions

        let del_key_params = db_params::DelKeyParams::builder()
            .store("Main".to_string())
            .keys(vec![StoreKey(vec![1.0, 1.2])])
            .build();
        assert!(db_client.del_key(del_key_params).await.is_err());

        let del_key_params = db_params::DelKeyParams::builder()
            .store("Main".to_string())
            .keys(vec![StoreKey(vec![1.0, 1.1, 1.2, 1.3])])
            .build();

        assert_eq!(
            db_client.del_key(del_key_params).await.unwrap(),
            ServerResponse::Del(1)
        );
        assert_eq!(
            db_client.list_stores(None).await.unwrap(),
            ServerResponse::StoreList(HashSet::from_iter([StoreInfo {
                name: StoreName("Main".to_string()),
                len: 1,
                size_in_bytes: 1904,
            },]))
        );
    }

    #[tokio::test]
    async fn test_get_sim_n() {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let address = server.local_addr().expect("Could not get local addr");
        let _ = tokio::spawn(async move { server.start().await });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let host = address.ip();
        let port = address.port();
        let db_client = DbClient::new(host.to_string(), port)
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
            ServerResponse::GetSimN(vec![(
                StoreKey(vec![2.0, 2.1, 2.2]),
                HashMap::from_iter([(
                    MetadataKey::new("medal".into()),
                    MetadataValue::RawString("gold".into()),
                )]),
                Similarity(0.9036338825194858),
            )])
        );
    }
}
