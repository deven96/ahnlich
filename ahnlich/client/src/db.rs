use crate::conn::Conn;
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
pub struct DbConnManager {
    host: String,
    port: u16,
}

impl DbConnManager {
    fn new(host: String, port: u16) -> Self {
        Self { host, port }
    }
}

#[async_trait::async_trait]
impl Manager for DbConnManager {
    type Type = Conn;
    type Error = AhnlichError;

    async fn create(&self) -> Result<Conn, AhnlichError> {
        Conn::new(&self.host, self.port).await
    }

    async fn recycle(&self, conn: &mut Conn, _metrics: &Metrics) -> RecycleResult<AhnlichError> {
        conn.is_db_conn_valid().await.map_err(RecycleError::Backend)
    }
}

/// Allow executing multiple queries at once
#[derive(Debug)]
pub struct DbPipeline {
    queries: ServerQuery,
    conn: Object<DbConnManager>,
}

impl DbPipeline {
    /// push create store command to pipeline
    pub fn create_store(
        &mut self,
        store: StoreName,
        dimension: NonZeroUsize,
        create_predicates: HashSet<MetadataKey>,
        error_if_exists: bool,
    ) {
        self.queries.push(Query::CreateStore {
            store,
            dimension,
            create_predicates,
            error_if_exists,
        })
    }

    /// push get key command to pipeline
    pub fn get_key(&mut self, store: StoreName, keys: Vec<StoreKey>) {
        self.queries.push(Query::GetKey { store, keys })
    }

    /// push get pred command to pipeline
    pub fn get_pred(&mut self, store: StoreName, condition: PredicateCondition) {
        self.queries.push(Query::GetPred { store, condition })
    }

    /// push get sim n command to pipeline
    pub fn get_sim_n(
        &mut self,
        store: StoreName,
        search_input: StoreKey,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
        condition: Option<PredicateCondition>,
    ) {
        self.queries.push(Query::GetSimN {
            store,
            search_input,
            closest_n,
            algorithm,
            condition,
        })
    }

    /// push create index command to pipeline
    pub fn create_index(&mut self, store: StoreName, predicates: HashSet<MetadataKey>) {
        self.queries.push(Query::CreateIndex { store, predicates })
    }

    /// push drop index command to pipeline
    pub fn drop_index(
        &mut self,
        store: StoreName,
        predicates: HashSet<MetadataKey>,
        error_if_not_exists: bool,
    ) {
        self.queries.push(Query::DropIndex {
            store,
            predicates,
            error_if_not_exists,
        })
    }

    /// push set command to pipeline
    pub fn set(&mut self, store: StoreName, inputs: Vec<(StoreKey, StoreValue)>) {
        self.queries.push(Query::Set { store, inputs })
    }

    /// push del key command to pipeline
    pub fn del_key(&mut self, store: StoreName, keys: Vec<StoreKey>) {
        self.queries.push(Query::DelKey { store, keys })
    }

    /// push del pred command to pipeline
    pub fn del_pred(&mut self, store: StoreName, condition: PredicateCondition) {
        self.queries.push(Query::DelPred { store, condition })
    }

    /// push drop store command to pipeline
    pub fn drop_store(&mut self, store: StoreName, error_if_not_exists: bool) {
        self.queries.push(Query::DropStore {
            store,
            error_if_not_exists,
        })
    }
    /// push ping command to pipeline
    pub fn ping(&mut self) {
        self.queries.push(Query::Ping)
    }

    /// push info server command to pipeline
    pub fn info_server(&mut self) {
        self.queries.push(Query::InfoServer)
    }

    /// push list stores command to pipeline
    pub fn list_stores(&mut self) {
        self.queries.push(Query::ListStores)
    }

    /// push list clients command to pipeline
    pub fn list_clients(&mut self) {
        self.queries.push(Query::ListClients)
    }

    /// execute queries all at once and return ordered list of results matching the order in which
    /// queries were pushed
    pub async fn exec(mut self) -> Result<ServerResult, AhnlichError> {
        self.conn.send_db_query(self.queries).await
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
    pub async fn pipeline(&self, capacity: usize) -> Result<DbPipeline, AhnlichError> {
        Ok(DbPipeline {
            queries: ServerQuery::with_capacity(capacity),
            conn: self.pool.get().await?,
        })
    }

    pub async fn create_store(
        &self,
        store: StoreName,
        dimension: NonZeroUsize,
        create_predicates: HashSet<MetadataKey>,
        error_if_exists: bool,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::CreateStore {
            store,
            dimension,
            create_predicates,
            error_if_exists,
        })
        .await
    }

    pub async fn get_key(
        &self,
        store: StoreName,
        keys: Vec<StoreKey>,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::GetKey { store, keys }).await
    }

    pub async fn get_pred(
        &self,
        store: StoreName,
        condition: PredicateCondition,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::GetPred { store, condition }).await
    }

    pub async fn get_sim_n(
        &self,
        store: StoreName,
        search_input: StoreKey,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
        condition: Option<PredicateCondition>,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::GetSimN {
            store,
            search_input,
            closest_n,
            algorithm,
            condition,
        })
        .await
    }

    pub async fn create_index(
        &self,
        store: StoreName,
        predicates: HashSet<MetadataKey>,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::CreateIndex { store, predicates }).await
    }

    pub async fn drop_index(
        &self,
        store: StoreName,
        predicates: HashSet<MetadataKey>,
        error_if_not_exists: bool,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::DropIndex {
            store,
            predicates,
            error_if_not_exists,
        })
        .await
    }

    pub async fn set(
        &self,
        store: StoreName,
        inputs: Vec<(StoreKey, StoreValue)>,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::Set { store, inputs }).await
    }

    pub async fn del_key(
        &self,
        store: StoreName,
        keys: Vec<StoreKey>,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::DelKey { store, keys }).await
    }

    pub async fn del_pred(
        &self,
        store: StoreName,
        condition: PredicateCondition,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::DelPred { store, condition }).await
    }

    pub async fn drop_store(
        &self,
        store: StoreName,
        error_if_not_exists: bool,
    ) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::DropStore {
            store,
            error_if_not_exists,
        })
        .await
    }

    pub async fn ping(&self) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::Ping).await
    }

    pub async fn info_server(&self) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::InfoServer).await
    }

    pub async fn list_stores(&self) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::ListStores).await
    }

    pub async fn list_clients(&self) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::ListClients).await
    }

    async fn exec(&self, query: Query) -> Result<ServerResponse, AhnlichError> {
        let mut conn = self.pool.get().await?;
        let mut queries = ServerQuery::with_capacity(1);
        queries.push(query);
        let res = conn
            .send_db_query(queries)
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
    use ndarray::array;
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use tokio::time::Duration;

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
        assert!(db_client.ping().await.is_ok());
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
            .pipeline(3)
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
        assert!(db_client.ping().await.is_err());
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
            .pipeline(4)
            .await
            .expect("Could not create pipeline");
        pipeline.create_store(
            StoreName("Main".to_string()),
            NonZeroUsize::new(3).unwrap(),
            HashSet::new(),
            true,
        );
        pipeline.create_store(
            StoreName("Main".to_string()),
            NonZeroUsize::new(2).unwrap(),
            HashSet::new(),
            true,
        );
        pipeline.create_store(
            StoreName("Main".to_string()),
            NonZeroUsize::new(2).unwrap(),
            HashSet::new(),
            false,
        );
        pipeline.list_stores();
        let mut expected = ServerResult::with_capacity(4);
        expected.push(Ok(ServerResponse::Unit));
        expected.push(Err("Store Main already exists".to_string()));
        expected.push(Ok(ServerResponse::Unit));
        expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
            StoreInfo {
                name: StoreName("Main".to_string()),
                len: 0,
                size_in_bytes: 1712,
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
        assert!(db_client
            .del_key(StoreName("Main".to_string()), vec![],)
            .await
            .is_err());
        assert!(db_client
            .create_store(
                StoreName("Main".to_string()),
                NonZeroUsize::new(4).unwrap(),
                HashSet::from_iter([MetadataKey::new("role".into())]),
                true,
            )
            .await
            .is_ok());
        assert_eq!(
            db_client
                .del_key(
                    StoreName("Main".to_string()),
                    vec![StoreKey(array![1.0, 1.1, 1.2, 1.3])],
                )
                .await
                .unwrap(),
            ServerResponse::Del(0)
        );
        assert!(db_client
            .set(
                StoreName("Main".to_string()),
                vec![
                    (StoreKey(array![1.0, 1.1, 1.2, 1.3]), HashMap::new()),
                    (StoreKey(array![1.1, 1.2, 1.3, 1.4]), HashMap::new()),
                ],
            )
            .await
            .is_ok());
        assert_eq!(
            db_client.list_stores().await.unwrap(),
            ServerResponse::StoreList(HashSet::from_iter([StoreInfo {
                name: StoreName("Main".to_string()),
                len: 2,
                size_in_bytes: 1880,
            },]))
        );
        // error as different dimensions
        assert!(db_client
            .del_key(
                StoreName("Main".to_string()),
                vec![StoreKey(array![1.0, 1.2])],
            )
            .await
            .is_err());
        assert_eq!(
            db_client
                .del_key(
                    StoreName("Main".to_string()),
                    vec![StoreKey(array![1.0, 1.1, 1.2, 1.3])],
                )
                .await
                .unwrap(),
            ServerResponse::Del(1)
        );
        assert_eq!(
            db_client.list_stores().await.unwrap(),
            ServerResponse::StoreList(HashSet::from_iter([StoreInfo {
                name: StoreName("Main".to_string()),
                len: 1,
                size_in_bytes: 1808,
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
        assert!(db_client
            .create_store(
                StoreName("Main".to_string()),
                NonZeroUsize::new(3).unwrap(),
                HashSet::from_iter([MetadataKey::new("medal".into())]),
                true,
            )
            .await
            .is_ok());
        assert!(db_client
            .set(
                StoreName("Main".to_string()),
                vec![
                    (
                        StoreKey(array![1.2, 1.3, 1.4]),
                        HashMap::from_iter([(
                            MetadataKey::new("medal".into()),
                            MetadataValue::RawString("silver".into()),
                        )]),
                    ),
                    (
                        StoreKey(array![2.0, 2.1, 2.2]),
                        HashMap::from_iter([(
                            MetadataKey::new("medal".into()),
                            MetadataValue::RawString("gold".into()),
                        )]),
                    ),
                    (
                        StoreKey(array![5.0, 5.1, 5.2]),
                        HashMap::from_iter([(
                            MetadataKey::new("medal".into()),
                            MetadataValue::RawString("bronze".into()),
                        )]),
                    ),
                ],
            )
            .await
            .is_ok());
        // error due to dimension mismatch
        assert!(db_client
            .get_sim_n(
                StoreName("Main".to_string()),
                StoreKey(array![1.1, 2.0]),
                NonZeroUsize::new(2).unwrap(),
                Algorithm::EuclideanDistance,
                None,
            )
            .await
            .is_err());
        assert_eq!(
            db_client
                .get_sim_n(
                    StoreName("Main".to_string()),
                    StoreKey(array![5.0, 2.1, 2.2]),
                    NonZeroUsize::new(2).unwrap(),
                    Algorithm::CosineSimilarity,
                    Some(PredicateCondition::Value(Predicate::Equals {
                        key: MetadataKey::new("medal".into()),
                        value: MetadataValue::RawString("gold".into()),
                    })),
                )
                .await
                .unwrap(),
            ServerResponse::GetSimN(vec![(
                StoreKey(array![2.0, 2.1, 2.2]),
                HashMap::from_iter([(
                    MetadataKey::new("medal".into()),
                    MetadataValue::RawString("gold".into()),
                )]),
                Similarity(0.9036338825194858),
            )])
        );
    }
}
