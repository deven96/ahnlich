use crate::conn::{Connection, DBConn};
use crate::error::AhnlichError;
use crate::prelude::*;
use ahnlich_types::query_builders::db as db_params;
use deadpool::managed::Manager;
use deadpool::managed::Metrics;
use deadpool::managed::Object;
use deadpool::managed::Pool;
use deadpool::managed::RecycleError;
use deadpool::managed::RecycleResult;

/// TCP Connection manager to ahnlich db
#[derive(Debug)]
pub struct DbConnManager {
    host: String,
    port: u16,
}

impl DbConnManager {
    pub fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

#[async_trait::async_trait]
impl Manager for DbConnManager {
    type Type = DBConn;
    type Error = AhnlichError;

    async fn create(&self) -> Result<DBConn, AhnlichError> {
        DBConn::new(&self.host, self.port).await
    }

    async fn recycle(&self, conn: &mut DBConn, _metrics: &Metrics) -> RecycleResult<AhnlichError> {
        conn.is_conn_valid().await.map_err(RecycleError::Backend)
    }
}

/// Allow executing multiple queries at once
#[derive(Debug)]
pub struct DbPipeline {
    queries: ServerDBQuery,
    conn: Object<DbConnManager>,
}

impl DbPipeline {
    pub fn new_from_queries_and_conn(queries: ServerDBQuery, conn: Object<DbConnManager>) -> Self {
        Self { queries, conn }
    }

    /// push create store command to pipeline
    pub fn create_store(&mut self, params: db_params::CreateStoreParams) {
        self.queries.push(DBQuery::CreateStore {
            store: params.store,
            dimension: params.dimension,
            create_predicates: params.create_predicates,
            non_linear_indices: params.non_linear_indices,
            error_if_exists: params.error_if_exists,
        })
    }

    /// push get key command to pipeline
    pub fn get_key(&mut self, params: db_params::GetKeyParams) {
        self.queries.push(DBQuery::GetKey {
            store: params.store,
            keys: params.keys,
        })
    }

    /// push get pred command to pipeline
    pub fn get_pred(&mut self, params: db_params::GetPredParams) {
        self.queries.push(DBQuery::GetPred {
            store: params.store,
            condition: params.condition,
        })
    }

    /// push get sim n command to pipeline
    pub fn get_sim_n(&mut self, params: db_params::GetSimNParams) {
        self.queries.push(DBQuery::GetSimN {
            store: params.store,
            search_input: params.search_input,
            closest_n: params.closest_n,
            algorithm: params.algorithm,
            condition: params.condition,
        })
    }

    /// push create predicate index command to pipeline
    pub fn create_pred_index(&mut self, params: db_params::CreatePredIndexParams) {
        self.queries.push(DBQuery::CreatePredIndex {
            store: params.store,
            predicates: params.predicates,
        })
    }

    /// push create non linear index command to pipeline
    pub fn create_non_linear_algorithm_index(
        &mut self,
        params: db_params::CreateNonLinearAlgorithmIndexParams,
    ) {
        self.queries.push(DBQuery::CreateNonLinearAlgorithmIndex {
            store: params.store,
            non_linear_indices: params.non_linear_indices,
        })
    }

    /// push drop pred index command to pipeline
    pub fn drop_pred_index(&mut self, params: db_params::DropPredIndexParams) {
        self.queries.push(DBQuery::DropPredIndex {
            store: params.store,
            predicates: params.predicates,
            error_if_not_exists: params.error_if_not_exists,
        })
    }

    /// push drop non linear index command to pipeline
    pub fn drop_non_linear_algorithm_index(
        &mut self,
        params: db_params::DropNonLinearAlgorithmIndexParams,
    ) {
        self.queries.push(DBQuery::DropNonLinearAlgorithmIndex {
            store: params.store,
            non_linear_indices: params.non_linear_indices,
            error_if_not_exists: params.error_if_not_exists,
        })
    }

    /// push set command to pipeline
    pub fn set(&mut self, params: db_params::SetParams) {
        self.queries.push(DBQuery::Set {
            store: params.store,
            inputs: params.inputs,
        })
    }

    /// push del key command to pipeline
    pub fn del_key(&mut self, params: db_params::DelKeyParams) {
        self.queries.push(DBQuery::DelKey {
            store: params.store,
            keys: params.keys,
        })
    }

    /// push del pred command to pipeline
    pub fn del_pred(&mut self, params: db_params::DelPredParams) {
        self.queries.push(DBQuery::DelPred {
            store: params.store,
            condition: params.condition,
        })
    }

    /// push drop store command to pipeline
    pub fn drop_store(&mut self, params: db_params::DropStoreParams) {
        self.queries.push(DBQuery::DropStore {
            store: params.store,
            error_if_not_exists: params.error_if_not_exists,
        })
    }
    /// push ping command to pipeline
    pub fn ping(&mut self) {
        self.queries.push(DBQuery::Ping)
    }

    /// push info server command to pipeline
    pub fn info_server(&mut self) {
        self.queries.push(DBQuery::InfoServer)
    }

    /// push list stores command to pipeline
    pub fn list_stores(&mut self) {
        self.queries.push(DBQuery::ListStores)
    }

    /// push list clients command to pipeline
    pub fn list_clients(&mut self) {
        self.queries.push(DBQuery::ListClients)
    }

    /// execute queries all at once and return ordered list of results matching the order in which
    /// queries were pushed
    pub async fn exec(mut self) -> Result<ServerResult, AhnlichError> {
        self.conn.send_query(self.queries).await
    }
}

/// Client for ahnlich db using an instantiated deadpool pool
#[derive(Debug)]
pub struct DbClient {
    pool: Pool<DbConnManager>,
}

impl DbClient {
    /// create new DB client with default deadpool config
    /// only made async because Pool::builder(...).build() can throw an error if not run within a
    /// runtime context like tokio
    pub async fn new(host: String, port: u16) -> Result<Self, AhnlichError> {
        let manager = DbConnManager::new(host, port);
        let pool = Pool::builder(manager).build()?;
        Ok(Self { pool })
    }

    /// create new DB client with custom deadpool pool
    pub fn new_with_pool(pool: Pool<DbConnManager>) -> Self {
        Self { pool }
    }

    /// Instantiate a new pipeline of a given capacity for which commands would be run sequentially
    /// on `pipeline.exec`
    pub async fn pipeline(
        &self,
        capacity: usize,
        tracing_id: Option<String>,
    ) -> Result<DbPipeline, AhnlichError> {
        Ok(DbPipeline::new_from_queries_and_conn(
            ServerDBQuery::with_capacity_and_tracing_id(capacity, tracing_id)?,
            self.pool.get().await?,
        ))
    }

    pub async fn create_store(
        &self,
        params: db_params::CreateStoreParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::CreateStore {
                store: params.store,
                dimension: params.dimension,
                create_predicates: params.create_predicates,
                non_linear_indices: params.non_linear_indices,
                error_if_exists: params.error_if_exists,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn get_key(
        &self,
        params: db_params::GetKeyParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::GetKey {
                store: params.store,
                keys: params.keys,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn get_pred(
        &self,
        params: db_params::GetPredParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::GetPred {
                store: params.store,
                condition: params.condition,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn get_sim_n(
        &self,
        params: db_params::GetSimNParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::GetSimN {
                store: params.store,
                search_input: params.search_input,
                closest_n: params.closest_n,
                algorithm: params.algorithm,
                condition: params.condition,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn create_pred_index(
        &self,
        params: db_params::CreatePredIndexParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::CreatePredIndex {
                store: params.store,
                predicates: params.predicates,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn create_non_linear_algorithm_index(
        &self,
        params: db_params::CreateNonLinearAlgorithmIndexParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::CreateNonLinearAlgorithmIndex {
                store: params.store,
                non_linear_indices: params.non_linear_indices,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn drop_pred_index(
        &self,
        params: db_params::DropPredIndexParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::DropPredIndex {
                store: params.store,
                predicates: params.predicates,
                error_if_not_exists: params.error_if_not_exists,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn drop_non_linear_algorithm_index(
        &self,
        params: db_params::DropNonLinearAlgorithmIndexParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::DropNonLinearAlgorithmIndex {
                store: params.store,
                non_linear_indices: params.non_linear_indices,
                error_if_not_exists: params.error_if_not_exists,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn set(&self, params: db_params::SetParams) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::Set {
                store: params.store,
                inputs: params.inputs,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn del_key(
        &self,
        params: db_params::DelKeyParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::DelKey {
                store: params.store,
                keys: params.keys,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn del_pred(
        &self,
        params: db_params::DelPredParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::DelPred {
                store: params.store,
                condition: params.condition,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn drop_store(
        &self,
        params: db_params::DropStoreParams,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(
            DBQuery::DropStore {
                store: params.store,
                error_if_not_exists: params.error_if_not_exists,
            },
            params.tracing_id,
        )
        .await
    }

    pub async fn ping(&self, tracing_id: Option<String>) -> Result<ServerResponse, AhnlichError> {
        self.exec(DBQuery::Ping, tracing_id).await
    }

    pub async fn info_server(
        &self,
        tracing_id: Option<String>,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(DBQuery::InfoServer, tracing_id).await
    }

    pub async fn list_stores(
        &self,
        tracing_id: Option<String>,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(DBQuery::ListStores, tracing_id).await
    }

    pub async fn list_clients(
        &self,
        tracing_id: Option<String>,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(DBQuery::ListClients, tracing_id).await
    }

    async fn exec(
        &self,
        query: DBQuery,
        tracing_id: Option<String>,
    ) -> Result<ServerResponse, AhnlichError> {
        let mut conn = self.pool.get().await?;
        let mut queries = ServerDBQuery::with_capacity_and_tracing_id(1, tracing_id)?;
        queries.push(query);
        let res = conn
            .send_query(queries)
            .await?
            .pop()
            .transpose()
            .map_err(AhnlichError::DbError)?;
        res.ok_or(AhnlichError::EmptyResponse)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ahnlich_db::cli::ServerConfig;
    use ahnlich_db::server::handler::Server;
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use std::collections::HashSet;
    use tokio::time::Duration;
    use utils::server::AhnlichServerUtils;

    static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());

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
        let db_client = DbClient::new(host.to_string(), port).await;
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
                size_in_bytes: 1056,
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
                size_in_bytes: 1352,
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
                size_in_bytes: 1240,
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
