use crate::conn::{AIConnect, Connection};
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
    type Type = AIConnect;
    type Error = AhnlichError;

    async fn create(&self) -> Result<AIConnect, AhnlichError> {
        AIConnect::new(&self.host, self.port).await
    }

    async fn recycle(
        &self,
        conn: &mut AIConnect,
        _metrics: &Metrics,
    ) -> RecycleResult<AhnlichError> {
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
    /// push create store command to pipeline
    pub fn create_store(
        &mut self,
        store: StoreName,
        store_type: AIStoreType,
        model: AIModel,
        predicates: HashSet<MetadataKey>,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
    ) {
        self.queries.push(AIQuery::CreateStore {
            store,
            r#type: store_type,
            predicates,
            non_linear_indices,
            model,
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
    pub fn set(&mut self, store: StoreName, inputs: Vec<(StoreInput, StoreValue)>) {
        self.queries.push(AIQuery::Set { store, inputs })
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
    pub async fn pipeline(&self, capacity: usize) -> Result<AIPipeline, AhnlichError> {
        Ok(AIPipeline {
            queries: AIServerQuery::with_capacity(capacity),
            conn: self.pool.get().await?,
        })
    }

    pub async fn create_store(
        &self,
        store: StoreName,
        store_type: AIStoreType,
        model: AIModel,
        predicates: HashSet<MetadataKey>,
        non_linear_indices: HashSet<NonLinearAlgorithm>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::CreateStore {
            r#type: store_type,
            store,
            model,
            predicates,
            non_linear_indices,
        })
        .await
    }

    pub async fn get_pred(
        &self,
        store: StoreName,
        condition: PredicateCondition,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::GetPred { store, condition }).await
    }

    pub async fn get_sim_n(
        &self,
        store: StoreName,
        search_input: StoreInput,
        condition: Option<PredicateCondition>,
        closest_n: NonZeroUsize,
        algorithm: Algorithm,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::GetSimN {
            store,
            search_input,
            condition,
            closest_n,
            algorithm,
        })
        .await
    }

    pub async fn create_pred_index(
        &self,
        store: StoreName,
        predicates: HashSet<MetadataKey>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::CreatePredIndex { store, predicates })
            .await
    }

    pub async fn drop_pred_index(
        &self,
        store: StoreName,
        predicates: HashSet<MetadataKey>,
        error_if_not_exists: bool,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::DropPredIndex {
            store,
            predicates,
            error_if_not_exists,
        })
        .await
    }

    pub async fn set(
        &self,
        store: StoreName,
        inputs: Vec<(StoreInput, StoreValue)>,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::Set { store, inputs }).await
    }

    pub async fn del_key(
        &self,
        store: StoreName,
        key: StoreInput,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::DelKey { store, key }).await
    }

    pub async fn drop_store(
        &self,
        store: StoreName,
        error_if_not_exists: bool,
    ) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::DropStore {
            store,
            error_if_not_exists,
        })
        .await
    }

    pub async fn info_server(&self) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::InfoServer).await
    }

    pub async fn list_stores(&self) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::ListStores).await
    }

    pub async fn purge_stores(&self) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::PurgeStores).await
    }

    pub async fn ping(&self) -> Result<AIServerResponse, AhnlichError> {
        self.exec(AIQuery::Ping).await
    }

    async fn exec(&self, query: AIQuery) -> Result<AIServerResponse, AhnlichError> {
        let mut conn = self.pool.get().await?;

        let mut queries = AIServerQuery::with_capacity(1);
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
