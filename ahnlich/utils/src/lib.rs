// Core modules (WASM-compatible)
pub mod fallible;
pub mod parallel;

// Server-only modules (need tokio/async)
#[cfg(feature = "server")]
pub mod size_calculation;

// Server-only modules
#[cfg(feature = "server")]
pub mod allocator;
#[cfg(feature = "server")]
pub mod auth;
#[cfg(feature = "server")]
pub mod cli;
#[cfg(feature = "server")]
pub mod client;
#[cfg(feature = "server")]
pub mod connection_layer;
#[cfg(feature = "server")]
pub mod persistence;
#[cfg(feature = "server")]
pub mod server;
#[cfg(feature = "server")]
pub mod snapshot;
