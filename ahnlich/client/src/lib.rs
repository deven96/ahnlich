//! A rust client for talking to ahnlich DB and AI
//!
//! Ships primarily the db, ai, error submodules
//!
//! ## Pooling
//!
//! DbConnManager and AIConnManager both implement r2d2::ManageConnection and so can be used to
//! create a pool of connections for reuse across multiple threads or within applications.
//!
//! ```rust
//! use ahnlich_client_rs::db::DbConnManager;
//! use r2d2::{Builder, Pool};
//!
//! let manager = DbConnManager::new("127.0.0.1".into(), 1369);
//! let pool = Builder::new().pool_size(5).build(manager).unwrap();
//! let db_client = DbClient::new_with_pool(pool);
//! db_client.ping().unwrap();
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
//! let db_client = DbClient::new("127.0.0.1".into(), 1369);
//! let mut pipeline = db_client.pipeline(3).unwrap();
//! pipeline.info_server();
//! pipeline.list_clients();
//! pipeline.list_stores();
//! let results = pipeline.exec().unwrap();
//! ```
pub mod conn;
pub mod db;
pub mod error;
