use grpc_types::{
    db::{
        pipeline::{db_query::Query, DbQuery, DbRequestPipeline, DbResponsePipeline},
        query::{
            CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey, DelPred,
            DropNonLinearAlgorithmIndex, DropPredIndex, DropStore, GetKey, GetPred, GetSimN,
            InfoServer, ListClients, ListStores, Ping, Set,
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

    pub fn del_pred(&mut self, params: DelPred) {
        self.queries.push(Query::DelPred(params));
    }

    pub fn info_server(&mut self) {
        self.queries.push(Query::InfoServer(InfoServer {}));
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
    use pretty_assertions::assert_eq;
    use std::{collections::HashMap, time::Duration};

    use super::*;
    use ahnlich_db::{cli::ServerConfig, errors::ServerError, server::handler::Server};
    use grpc_types::{
        algorithm::{algorithms::Algorithm, nonlinear::NonLinearAlgorithm},
        db::{
            pipeline::{db_server_response::Response, DbServerResponse},
            query::CreateStore,
            server::{GetSimNEntry, StoreInfo},
        },
        keyval::{StoreEntry, StoreKey, StoreValue},
        metadata::{metadata_value::Value, MetadataValue},
        shared::info::ErrorResponse,
        similarity::Similarity,
    };

    use grpc_types::predicates::{
        self, predicate::Kind as PredicateKind,
        predicate_condition::Kind as PredicateConditionKind, Predicate, PredicateCondition,
    };

    use grpc_types::{
        db::{
            pipeline::{self as db_pipeline},
            server as db_response_types,
        },
        keyval::StoreName,
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
            server.start().await.expect("Failed to start db server");
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

        let store_already_exists_err = ServerError::StoreAlreadyExists(StoreName {
            value: "Main".to_string(),
        });

        let expected = DbResponsePipeline {
            responses: vec![
                DbServerResponse {
                    response: Some(Response::Unit(Unit {})),
                },
                DbServerResponse {
                    response: Some(Response::Error(ErrorResponse {
                        message: store_already_exists_err.to_string(),
                        code: 6,
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
                            size_in_bytes: 1056,
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
            server.start().await.expect("Failed to start db server");
        });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let db_client = DbClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let create_store_params = CreateStore {
            store: "Main".to_string(),
            create_predicates: vec!["medal".to_string()],
            dimension: 3,
            non_linear_indices: vec![],
            error_if_exists: true,
        };

        assert!(db_client
            .create_store(create_store_params, None)
            .await
            .is_ok());

        let set_key_params = Set {
            store: "Main".to_string(),
            inputs: vec![
                StoreEntry {
                    key: Some(StoreKey {
                        key: vec![1.2, 1.3, 1.4],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from_iter([(
                            "medal".into(),
                            MetadataValue {
                                value: Some(Value::RawString("silver".into())),
                            },
                        )]),
                    }),
                },
                StoreEntry {
                    key: Some(StoreKey {
                        key: vec![2.0, 2.1, 2.2],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from_iter([(
                            "medal".into(),
                            MetadataValue {
                                value: Some(Value::RawString("gold".into())),
                            },
                        )]),
                    }),
                },
                StoreEntry {
                    key: Some(StoreKey {
                        key: vec![5.0, 5.1, 5.2],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from_iter([(
                            "medal".into(),
                            MetadataValue {
                                value: Some(Value::RawString("bronze".into())),
                            },
                        )]),
                    }),
                },
            ],
        };

        assert!(db_client.set(set_key_params, None).await.is_ok());
        // error due to dimension mismatch
        let get_sim_n_params = GetSimN {
            store: "Main".to_string(),
            search_input: Some(StoreKey {
                key: vec![1.1, 2.0],
            }),
            closest_n: 2,
            algorithm: Algorithm::EuclideanDistance as i32,
            condition: None,
        };
        assert!(db_client.get_sim_n(get_sim_n_params, None).await.is_err());

        let condition = PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::Equals(predicates::Equals {
                    key: "medal".into(),
                    value: Some(MetadataValue {
                        value: Some(grpc_types::metadata::metadata_value::Value::RawString(
                            "gold".to_string(),
                        )),
                    }),
                })),
            })),
        };

        let get_sim_n_params = GetSimN {
            store: "Main".to_string(),
            search_input: Some(StoreKey {
                key: vec![5.0, 2.1, 2.2],
            }),
            closest_n: 2,
            algorithm: Algorithm::CosineSimilarity as i32,
            condition: Some(condition),
        };

        assert_eq!(
            db_client.get_sim_n(get_sim_n_params, None).await.unwrap(),
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

    #[tokio::test]
    async fn test_simple_server_ping() {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let address = server.local_addr().expect("Could not get local addr");
        let _ = tokio::spawn(async move { server.start().await });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let db_client = DbClient::new(address.to_string())
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
        tokio::spawn(async { server.start().await });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let db_client = DbClient::new(address.to_string())
            .await
            .expect("Could not initialize client");
        let mut pipeline = db_client.pipeline(None);
        pipeline.list_stores();
        pipeline.ping();

        let expected = vec![
            db_pipeline::DbServerResponse {
                response: Some(db_pipeline::db_server_response::Response::StoreList(
                    db_response_types::StoreList { stores: vec![] },
                )),
            },
            db_pipeline::DbServerResponse {
                response: Some(db_pipeline::db_server_response::Response::Pong(
                    db_response_types::Pong {},
                )),
            },
        ];

        let expected = db_pipeline::DbResponsePipeline {
            responses: expected,
        };

        let res = pipeline.exec().await.expect("Could not execute pipeline");
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_pool_commands_fail_if_server_not_exist() {
        let host = "127.0.0.1";
        let port = 1234;
        let address = format!("{host}:{port}");
        let db_client = DbClient::new(address).await;
        assert!(db_client.is_err());
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

        let error_response = ahnlich_db::errors::ServerError::StoreAlreadyExists(StoreName {
            value: "Main".to_string(),
        });

        let expected = vec![
            db_pipeline::DbServerResponse {
                response: Some(db_pipeline::db_server_response::Response::Unit(
                    db_response_types::Unit {},
                )),
            },
            db_pipeline::DbServerResponse {
                response: Some(db_pipeline::db_server_response::Response::Error(
                    grpc_types::shared::info::ErrorResponse {
                        message: error_response.to_string(),
                        code: 6,
                    },
                )),
            },
            db_pipeline::DbServerResponse {
                response: Some(db_pipeline::db_server_response::Response::Unit(
                    db_response_types::Unit {},
                )),
            },
            db_pipeline::DbServerResponse {
                response: Some(db_pipeline::db_server_response::Response::StoreList(
                    db_response_types::StoreList {
                        stores: vec![db_response_types::StoreInfo {
                            name: "Main".to_string(),
                            len: 0,
                            size_in_bytes: 1056,
                        }],
                    },
                )),
            },
        ];

        let expected = db_pipeline::DbResponsePipeline {
            responses: expected,
        };

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
        let db_client = DbClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let del_key_params = DelKey {
            store: "Main".to_string(),
            keys: vec![],
        };
        assert!(db_client.del_key(del_key_params, None).await.is_err());

        let create_store_params = CreateStore {
            store: "Main".to_string(),
            create_predicates: vec!["role".to_string()],
            dimension: 4,
            non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
            error_if_exists: true,
        };

        assert!(db_client
            .create_store(create_store_params, None)
            .await
            .is_ok());

        let del_key_params = DelKey {
            store: "Main".to_string(),
            keys: vec![StoreKey {
                key: vec![1.0, 1.1, 1.2, 1.3],
            }],
        };

        assert_eq!(
            db_client.del_key(del_key_params, None).await.unwrap(),
            db_response_types::Del { deleted_count: 0 },
        );

        let set_key_params = Set {
            store: "Main".to_string(),
            inputs: vec![
                StoreEntry {
                    key: Some(StoreKey {
                        key: vec![1.0, 1.1, 1.2, 1.3],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                },
                StoreEntry {
                    key: Some(StoreKey {
                        key: vec![1.1, 1.2, 1.3, 1.4],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                },
            ],
        };

        assert!(db_client.set(set_key_params, None).await.is_ok());

        assert_eq!(
            db_client.list_stores(None).await.unwrap(),
            db_response_types::StoreList {
                stores: vec![StoreInfo {
                    name: "Main".to_string(),
                    len: 2,
                    size_in_bytes: 1356,
                }]
            }
        );

        // error as different dimensions

        let del_key_params = DelKey {
            store: "Main".to_string(),
            keys: vec![StoreKey {
                key: vec![1.0, 1.1],
            }],
        };

        assert!(db_client.del_key(del_key_params, None).await.is_err());

        let del_key_params = DelKey {
            store: "Main".to_string(),
            keys: vec![StoreKey {
                key: vec![1.0, 1.1, 1.2, 1.3],
            }],
        };

        assert_eq!(
            db_client.del_key(del_key_params, None).await.unwrap(),
            db_response_types::Del { deleted_count: 1 },
        );

        assert_eq!(
            db_client.list_stores(None).await.unwrap(),
            db_response_types::StoreList {
                stores: vec![StoreInfo {
                    name: "Main".to_string(),
                    len: 1,
                    size_in_bytes: 1244,
                }]
            }
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
        let db_client = DbClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let create_store_params = CreateStore {
            store: "Main".to_string(),
            create_predicates: vec!["medal".to_string()],
            dimension: 3,
            non_linear_indices: vec![],
            error_if_exists: true,
        };

        assert!(db_client
            .create_store(create_store_params, None)
            .await
            .is_ok());

        let set_key_params = Set {
            store: "Main".to_string(),
            inputs: vec![
                StoreEntry {
                    key: Some(StoreKey {
                        key: vec![1.2, 1.3, 1.4],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from_iter([(
                            "medal".into(),
                            MetadataValue {
                                value: Some(Value::RawString("silver".into())),
                            },
                        )]),
                    }),
                },
                StoreEntry {
                    key: Some(StoreKey {
                        key: vec![2.0, 2.1, 2.2],
                    }),

                    value: Some(StoreValue {
                        value: HashMap::from_iter([(
                            "medal".into(),
                            MetadataValue {
                                value: Some(Value::RawString("gold".into())),
                            },
                        )]),
                    }),
                },
                StoreEntry {
                    key: Some(StoreKey {
                        key: vec![5.0, 5.1, 5.2],
                    }),

                    value: Some(StoreValue {
                        value: HashMap::from_iter([(
                            "medal".into(),
                            MetadataValue {
                                value: Some(Value::RawString("bronze".into())),
                            },
                        )]),
                    }),
                },
            ],
        };

        assert!(db_client.set(set_key_params, None).await.is_ok());

        // error due to dimension mismatch

        let get_sim_n_params = GetSimN {
            store: "Main".to_string(),
            search_input: Some(StoreKey {
                key: vec![1.1, 2.0],
            }),
            closest_n: 2,
            algorithm: Algorithm::EuclideanDistance as i32,
            condition: None,
        };
        assert!(db_client.get_sim_n(get_sim_n_params, None).await.is_err());

        let condition = PredicateCondition {
            kind: Some(PredicateConditionKind::Value(Predicate {
                kind: Some(PredicateKind::Equals(predicates::Equals {
                    key: "medal".into(),
                    value: Some(MetadataValue {
                        value: Some(grpc_types::metadata::metadata_value::Value::RawString(
                            "gold".to_string(),
                        )),
                    }),
                })),
            })),
        };

        let get_sim_n_params = GetSimN {
            store: "Main".to_string(),
            search_input: Some(StoreKey {
                key: vec![5.0, 2.1, 2.2],
            }),
            closest_n: 2,
            algorithm: Algorithm::CosineSimilarity as i32,
            condition: Some(condition),
        };

        assert_eq!(
            db_client.get_sim_n(get_sim_n_params, None).await.unwrap(),
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
