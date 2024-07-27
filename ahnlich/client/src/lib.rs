//! A rust client for talking to ahnlich DB and AI
//!
//! Ships primarily the db, ai, error submodules
//!
//! ## Pooling
//!
//! DbConnManager and AIConnManager both implement deadpool::managed::Manager and so can be used to
//! create a pool of connections for reuse across multiple threads or within applications.
//!
//! ```rust
//! use ahnlich_client_rs::db::DbConnManager;
//! use deadpool::managed::Pool;
//!
//! let manager = DbConnManager::new("127.0.0.1".into(), 1369);
//! let pool = Pool::builder(manager).max_size(10).build().unwrap();
//! let db_client = DbClient::new_with_pool(pool);
//! db_client.ping().await.unwrap();
//! ```
//!
//! ## Pipelining
//!
//! When using a client to issue commands, there is no guarantee of reading your own writes, even
//! when the commands are sent sequentially in client code, this can be remedied by using a
//! pipeline which then couples all the commands in an ordered list and gets an ordered list of
//! responses in return
//!
//! ```rust
//! use ahnlich_client_rs::db::DbClient;
//!
//! let db_client = DbClient::new("127.0.0.1".into(), 1369).await;
//! let mut pipeline = db_client.pipeline(3).unwrap();
//! pipeline.info_server();
//! pipeline.list_clients();
//! pipeline.list_stores();
//! let results = pipeline.exec().await.unwrap();
//! ```
//!
//! ## Lib Types
//!
//! Necessary library types to pass into client methods can be found from prelude
//!
//! ```rust
//! use ahnlich_client_rs::db::DbClient;
//! use ahnlich_client_rs::prelude::*;
//! use std::num::NonZeroUsize;
//! use std::collections::HashSet;
//!
//! let db_client = DbClient::new("127.0.0.1".into(), 1369).await;
//! let mut pipeline = db_client.pipeline(1).unwrap();
//! pipeline.create_store(
//!     // StoreName found in prelude
//!     StoreName("Main".to_string()),
//!     NonZeroUsize::new(3).unwap(),
//!     HashSet::new(),
//!     true,
//! );
//! let results = pipeline.exec().await.unwrap();
//! ```
pub mod ai;
pub mod conn;
pub mod db;
pub mod error;
pub mod prelude;
