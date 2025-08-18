---
title: Rust
sidebar_position: 30
---

# Rust

The Ahnlich Rust client library is a Rust crate that allows you to interact with [Ahnlich DB](/docs/components/ahnlich-db/ahnlich-db.md) and [Ahnlich AI](/docs/components/ahnlich-ai/ahnlich-ai.md).

---
id: client-rust

title: Rust SDK (🦀)

sidebar_label: Rust

description: Official Rust client library to integrate with Ahnlich DB (exact vector search) and AI (semantic embeddings) services.

---

<!-- import RustIcon from '@site/static/img/icons/lang/rust.svg' -->

## 🦀 Ahnlich Rust SDK

The official Rust client to interface with **ahnlich‑db** (exact similarity search) and **ahnlich‑ai** (semantic similarity) over gRPC.

See full API docs and examples at [docs.rs – `ahnlich_client_rs`](https://docs.rs/ahnlich_client_rs/0.1.0/ahnlich_client_rs/) :contentReference[oaicite:0]{index=0}



## 🚀 Connecting to DB / AI Services

Both services expect:

- `ahnlich-db` should be accessible (default: `127.0.0.1:1369`)
- `ahnlich-ai` should be separately reachable (default: `127.0.0.1:1370` or as configured)

The SDK supports optional W3C trace context via an `Option<String>` `trace_id` in all calls. 

```rust,no_run
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let tracing_id: Option<String> = None;

    // ✔️ DB client:
    let db_client = ahnlich_client_rs::db::DbClient::new("127.0.0.1:1369".to_string()).await?;
    db_client.ping(tracing_id.clone()).await?;

    // ✔️ AI client:
    let ai_client = ahnlich_client_rs::ai::AIClient::new("127.0.0.1:1369".to_string()).await?;
    ai_client.ping(tracing_id.clone()).await?;

    Ok(())
}
```
<!-- < Can grab example rust snippets from > -->

🧠 Best Practices
Always match vector/query dimension to the store’s declared dimension (e.g. 128 or 768).

Use DbClient::pipeline() or AIClient::pipeline() if you require ordered batched operations with predictable response order.

Metadata predicates are fast and flexible filtering tools—even if predicates aren't pre-indexed.

AI Stores automatically handle embedding; no need to compute embeddings manually for raw input.


## Installation
[Installation and Usage](installation-and-usage.md)


## Reference
[Reference](reference.md)
