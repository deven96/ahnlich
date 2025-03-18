//! FIXME: Fix documentation for rust client
//! A rust client for talking to ahnlich DB and AI
//!
//! Ships primarily the db, ai, error submodules
//!
//! ## Pooling
//!
//! DbConnManager and AIConnManager both implement deadpool::managed::Manager and so can be used to
//! create a pool of connections for reuse across multiple threads or within applications.
//!
//! ### DB Client
//! ```rust
//! use ahnlich_client_rs::db::DbConnManager;
//! use deadpool::managed::Pool;
//! use ahnlich_client_rs::db::DbClient;
//!
//! let manager = DbConnManager::new("127.0.0.1".into(), 1369);
//! let pool = Pool::builder(manager).max_size(10).build().unwrap();
//! let db_client = DbClient::new_with_pool(pool);
//!
//! // Library has support for distributed tracing. https://www.w3.org/TR/trace-context/#traceparent-header
//! let tracing_id: Option<String> = None;
//! db_client.ping(tracing_id).await.unwrap();
//! ```
//!
//! ### AI Client
//! ```rust
//! use ahnlich_client_rs::ai::AIConnManager;
//! use deadpool::managed::Pool;
//! use ahnlich_client_rs::ai::AIClient;
//!
//! let manager = AIConnManager::new("127.0.0.1".into(), 1369);
//! let pool = Pool::builder(manager).max_size(10).build().unwrap();
//! // Library has support for distributed tracing - https://www.w3.org/TR/trace-context/#traceparent-header
//! let tracing_id: Option<String> = None;
//! let ai_client = AIClient::new_with_pool(pool);
//! ai_client.ping(tracing_id).await.unwrap();
//! ```
//!
//! ## Pipelining
//!
//! When using a client(db or aiproxy) to issue commands, there is no guarantee of reading your own writes, even
//! when the commands are sent sequentially in client code, this can be remedied by using a
//! pipeline which then couples all the commands in an ordered list and gets an ordered list of
//! responses in return
//!
//! ```rust
//! use ahnlich_client_rs::db::DbClient;
//!
//! let db_client = DbClient::new("127.0.0.1".into(), 1369).await.unwrap();
//! let tracing_id: Option<String> = None;
//! let mut pipeline = db_client.pipeline(3, tracing_id).unwrap();
//! pipeline.info_server();
//! pipeline.list_clients();
//! pipeline.list_stores();
//! let results = pipeline.exec().await.unwrap();
//! ```
//!
//! ## Lib Types
//!
//! Necessary library types to pass into the clients methods can be found from prelude
//!
//! ### DB Client
//! ```rust
//! use ahnlich_client_rs::db::DbClient;
//! use ahnlich_types::query_builders::db as db_params;
//! use ahnlich_client_rs::prelude::*;
//! use std::num::NonZeroUsize;
//! use std::collections::HashSet;
//!
//! let db_client = DbClient::new("127.0.0.1".into(), 1369).await.unwrap();
//! let tracing_id: Option<String> = None;
//! let mut pipeline = db_client.pipeline(1, tracing_id).unwrap();
//! let create_store_params = db_params::CreateStoreParams::builder()
//!         .store("Main".to_string())
//!         .dimension(3)
//!         .build();
//!
//! pipeline.create_store(
//!     create_store_params
//! );
//! let results = pipeline.exec().await.unwrap();
//! ```
//!
//! ### AI Client
//! ```rust
//! use ahnlich_client_rs::ai::AIClient;
//! use ahnlich_client_rs::prelude::*;
//! use ahnlich_types::query_builders::ai as ai_params;
//! use std::collections::HashSet;
//!
//! let ai_client = AIClient::new("127.0.0.1".into(), 1369).await.unwrap();
//! let store_name = StoreName("Less".to_string());
//! // Model used to set to create embeddings for all forms of get command
//! let query_model = AIModel::AllMiniLML6V2;
//! // Model used to set to create embeddings for set command
//! let index_model = AIModel::AllMiniLML6V2;
//! let tracing_id: Option<String> = None;
//! let mut pipeline = ai_client.pipeline(2, tracing_id).unwrap();
//! let create_store_params = ai_params::CreateStoreParams::builder()
//!     .store("Main".to_string())
//!     .index_model(index_model)
//!     .query_model(query_model)
//!     .build();
//!   
//!   pipeline.create_store(
//!     create_store_params
//!   );
//!
//! let set_params = ai_params::SetParams::builder()
//!     .store(store_name.clone().to_string())
//!     .inputs(vec![
//!     (StoreInput::RawString("Adidas Yeezy".into()), HashMap::new()),
//!     (
//!         StoreInput::RawString("Nike Air Jordans".into()),
//!         HashMap::new(),
//!     ),
//!     ])
//!     .preprocess_action(PreprocessAction::NoPreprocessing)
//!     .build();
//!
//! let results = pipeline.exec().await.unwrap();
//! ```
pub mod error;
pub mod grpc;
pub mod prelude;
