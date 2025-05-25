//! Ahnlich Rust Client
//!
//! A Rust client for communicating with Ahnlich's DB and AI services over gRPC.
//!
//! ## Overview
//! This crate provides clients for interacting with both the vector database (DB) and AI services in Ahnlich. It supports building pipelines for batching multiple requests in order and receiving their responses in sequence.
//!
//! Internally, connections are managed via gRPC multiplexing—no external pooling is needed.
//!
//! ## Clients
//!
//! This crate exposes two primary client modules:
//!
//! - [`db`] — for interacting with the vector database.
//! - [`ai`] — for interacting with the AI service.
//!
//! Both clients provide direct methods and pipeline builders for batching commands.
//!
//! ---
//!
//! ## Example: DB Client
//! ```rust
//! use ahnlich_client_rs::db::DbClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;
//!     let tracing_id: Option<String> = None;
//!
//!     db_client.ping(tracing_id).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Example: AI Client
//! ```rust
//! use ahnlich_client_rs::ai::AIClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let ai_client = AIClient::new("127.0.0.1:1369".to_string()).await?;
//!     let tracing_id: Option<String> = None;
//!
//!     ai_client.ping(tracing_id).await?;
//!     Ok(())
//! }
//! ```
//!
//! ---
//!
//! ## Pipelines
//!
//! Pipelines enable multiple ordered operations to be issued in a batch. This ensures the operations are executed in sequence and allows for consistent read-after-write semantics.
//!
//! ### Example: DB Pipeline
//! ```rust
//! use ahnlich_client_rs::db::DbClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;
//!     let tracing_id: Option<String> = None;
//!     let mut pipeline = db_client.pipeline(tracing_id);
//!
//!     pipeline.list_clients();
//!     pipeline.list_stores();
//!     let responses = pipeline.exec().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ---
//!
//! ## Types & Utilities
//!
//! This crate re-exports common parameter and result types via the `prelude` module. You can use it to simplify imports:
//!
//! ```rust
//! use ahnlich_client_rs::prelude::*;
//! ```
//!
//! For full type definitions and request builders, refer to the `ahnlich_types` crate.
//!
//! ---
//!
//! ## Testing
//!
//! Example tests can be found in the repository to demonstrate usage, including:
//!
//! - Creating stores
//! - Setting and getting values
//! - Listing clients and stores
//! - Predicate-based querying
//!
//! These tests validate the behavior of both DB and AI pipelines in real scenarios, including with image inputs and structured metadata.
//!
//! ---
//!
//! ## Distributed Tracing
//!
//! The client supports [W3C Trace Context](https://www.w3.org/TR/trace-context/#traceparent-header) via optional `traceparent` headers. You can pass an `Option<String>` as `tracing_id` to propagate context across services.
//!
//! ---
//!
pub mod ai;
pub mod db;
pub mod error;
pub mod prelude;
