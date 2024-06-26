use crate::conn::Conn;
use crate::error::AhnlichError;
use deadpool::managed::Manager;
use deadpool::managed::Metrics;
use deadpool::managed::Object;
use deadpool::managed::Pool;
use deadpool::managed::RecycleError;
use deadpool::managed::RecycleResult;
pub use types::query::*;
pub use types::server::*;

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

    pub async fn pipeline(&self, capacity: usize) -> Result<DbPipeline, AhnlichError> {
        Ok(DbPipeline {
            queries: ServerQuery::with_capacity(capacity),
            conn: self.pool.get().await?,
        })
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
    use once_cell::sync::Lazy;
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;
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
        let res = pipeline.exec().await.expect("Could not execute pipeline");
        let mut expected = ServerResult::with_capacity(2);
        expected.push(Ok(ServerResponse::StoreList(HashSet::new())));
        expected.push(Ok(ServerResponse::Pong));
        assert_eq!(res, expected);
    }

    #[tokio::test]
    async fn test_pool_fails_if_server_not_exist() {
        let host = "127.0.0.1";
        let port = 1234;
        let db_client = DbClient::new(host.to_string(), port)
            .await
            .expect("Could not initialize client");
        assert!(db_client.ping().await.is_err());
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
        let res = db_client.ping().await.expect("timeout");
        assert_eq!(res, ServerResponse::Pong);
    }
}
