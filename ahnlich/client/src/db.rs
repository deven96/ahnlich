use crate::conn::Conn;
use crate::error::AhnlichError;
use r2d2::Builder;
use r2d2::ManageConnection;
use r2d2::Pool;
use r2d2::PooledConnection;
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

impl ManageConnection for DbConnManager {
    type Connection = Conn;
    type Error = AhnlichError;

    fn connect(&self) -> Result<Conn, AhnlichError> {
        Conn::new(&self.host, self.port)
    }

    fn is_valid(&self, conn: &mut Conn) -> Result<(), AhnlichError> {
        conn.is_db_conn_valid()
    }

    fn has_broken(&self, conn: &mut Conn) -> bool {
        conn.is_db_conn_broken()
    }
}

/// Allow executing multiple queries at once
#[derive(Debug)]
pub struct DbPipeline {
    queries: ServerQuery,
    conn: PooledConnection<DbConnManager>,
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
    pub fn exec(mut self) -> Result<ServerResult, AhnlichError> {
        self.conn.send_db_query(self.queries)
    }
}

/// Client for ahnlich db using an instantiated r2d2 pool
#[derive(Debug)]
pub struct DbClient {
    pool: Pool<DbConnManager>,
}

impl DbClient {
    /// create new DB client with default r2d2 config
    pub fn new(host: String, port: u16) -> Result<Self, AhnlichError> {
        let manager = DbConnManager::new(host, port);
        let pool = Builder::new().build(manager)?;
        Ok(Self { pool })
    }

    /// create new DB client with custom r2d2 pool
    pub fn new_with_pool(pool: Pool<DbConnManager>) -> Self {
        Self { pool }
    }

    pub fn pipeline(&self, capacity: usize) -> Result<DbPipeline, AhnlichError> {
        Ok(DbPipeline {
            queries: ServerQuery::with_capacity(capacity),
            conn: self.pool.clone().get()?,
        })
    }

    pub fn ping(&self) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::Ping)
    }

    pub fn info_server(&self) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::InfoServer)
    }

    pub fn list_stores(&self) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::ListStores)
    }

    pub fn list_clients(&self) -> Result<ServerResponse, AhnlichError> {
        self.exec(Query::ListClients)
    }

    fn exec(&self, query: Query) -> Result<ServerResponse, AhnlichError> {
        let mut conn = self.pool.clone().get()?;
        let mut queries = ServerQuery::with_capacity(1);
        queries.push(query);
        let res = conn
            .send_db_query(queries)?
            .pop()
            .transpose()
            .map_err(AhnlichError::DbError)?;
        res.ok_or(AhnlichError::EmptyResponse)
    }
}
