---
title: Rust Specific Resources
---

# Quickstart - Setup Guide

Before installing ahnlich_client_rs, you need Rust and Cargo set up on your system.

## Windows

Install Visual C++ Build Tools (needed by many crates)

```
winget install --id Microsoft.VisualStudio.2022.BuildTools -e --source winget
```

In the installer UI, check "Desktop development with C++" and finish

Install Rust (rustup + stable toolchain)

```
winget install --id Rustlang.Rustup -e
```

#Open a NEW terminal so PATH updates, then verify

```
rustc --version     # check Rust version
```

```
cargo --version     # check Cargo version
```

## macOS

Install compiler & build tools

```
xcode-select --install   # one-time setup
```

Install Rust (rustup + stable toolchain)

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Load Cargo into the shell (new terminals do this automatically)

```
source "$HOME/.cargo/env"
```

Verify installation

```
rustc --version
```

```
cargo --version
```

## Linux (Debian/Ubuntu example)

Install prerequisites

```
sudo apt update
```

```
sudo apt install -y build-essential pkg-config libssl-dev curl
```

Install Rust (rustup + stable toolchain)

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Load Cargo into the shell

```
source "$HOME/.cargo/env"
```

Verify installation

```
rustc --version
```

```
cargo --version
```

## Quick Sanity Check (All Platforms)

Create and run a starter app

```
cargo new hello-rust
```

```
cd hello-rust
```

```
cargo run   # should print "Hello, world!"
```

## Install the SDK

```
cargo add ahnlich_client_rs
```

## Whats Included in the SDK

The ahnlich_client_rs crate provides an idiomatic Rust client for Ahnlich’s Vector Database (DB) and AI services. It enables developers to integrate similarity search and AI-powered embedding workflows directly into Rust applications.

### Capabilities

* **DB Client**

  * Store vectors (`StoreKey: Vec<f32>`) and metadata (`StoreValue: HashMap<String, String>`).

  * Query for nearest neighbors with filtering (`Predicates`).

  * Manage stores (create, list, delete).

* **AI Client**

  * Generate embeddings from raw inputs (text, JSON, or other supported formats).

  * Interpret embeddings for similarity, clustering, or semantic tasks.

  * Designed to complement the DB client by producing vectors ready for indexing and search.

### Clients

This crate exposes two primary client modules for interacting with Ahnlich services:

* `db` — for interacting with the **Vector Database (DB)**.

* `ai` — for interacting with the **AI Service**.

Both clients support:

* **Direct method calls** for simple operations.

* **Pipeline builders** for batching multiple commands and receiving results in sequence.

### DB Client

The DB Client is used to manage vector stores and perform similarity queries. 
It provides methods for:

* Creating and deleting stores.

* Storing vectors and associated metadata.

* Querying nearest neighbors with filters and predicates.

* Running operations in pipelines for higher throughput.


#### Source Code Source Code Example: DB Client

use ahnlich_client_rs::db::DbClient;`**

```rust
#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;

    let tracing_id: Option<String> = None;

    db_client.ping(tracing_id).await?;

    Ok(())`**

}
```

### AI Client

The **AI Client** is used to generate and interpret embeddings.
It provides methods for:

* Creating embeddings from raw input (e.g., text, JSON).

* Sending embedding requests in pipelines for batch workloads.

* Integrating directly with the DB Client by producing vectors ready for storage.

#### Source Code Source Code Example: AI Client

``` rust
use ahnlich_client_rs::ai::AIClient;

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let ai_client = AIClient::new("127.0.0.1:1369".to_string()).await?;

    let tracing_id: Option<String> = None;

    ai_client.ping(tracing_id).await?;

    Ok(())

}
```

## Pipelines

Pipelines enable multiple ordered operations to be issued in a batch.
This ensures:

* **Sequential execution** — operations are applied in the order they are added.

* **Consistent read-after-write semantics** — queries can safely depend on previous writes.

* **Reduced overhead** — fewer gRPC round-trips compared to issuing requests individually.

Both the **DB Client** and **AI Client** provide pipeline builders, making it possible to group multiple commands into a single request stream.

#### Source Code Source Code Example: DB Pipeline

```rust
use ahnlich_client_rs::db::DbClient;

#[tokio::main]

async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;

    let tracing_id: Option<String> = None;

    let mut pipeline = db_client.pipeline(tracing_id);

    pipeline.list_clients();

    pipeline.list_stores();

    let responses = pipeline.exec().await?;

    Ok(())

}
```

In this Source Code Source Code Example:

* A new pipeline is created from the `DbClient`.

* Two operations are enqueued:

  * `list_clients()`

  * `list_stores()`

* `exec()` executes the pipeline, returning the responses in the same order the operations were added.