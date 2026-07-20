#![allow(clippy::size_of_ref)]
mod algorithm;

#[cfg(feature = "server")]
pub mod cli;

pub mod engine;
pub mod errors;

#[cfg(feature = "server")]
pub mod replication;

#[cfg(feature = "server")]
pub mod server;

#[cfg(test)]
mod tests;
