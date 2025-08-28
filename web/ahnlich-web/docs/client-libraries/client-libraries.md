---
title: Client libraries
sidebar_position: 40
---

import DocCardList from '@theme/DocCardList';

# 🧩 Ahnlich Client Libraries

Ahnlich offers SDKs for **Go**, **Python**, and **Rust**—each allowing you to manage vector stores, run similarity searches, and interact with the AI proxy programmatically.  

---

## Supported SDKs

| Language | SDK Name | Highlights |
|-------------|----------|------------|
| **Golang**🐹 | `ahnlich-client-go` | Minimal, idiomatic Go client supporting DB and optional AI-mode |
| **Python**🐍 | `ahnlich-client-py` | Async client (via `grpclib`) with built-in intent to support gRPC streaming and RAG workflows |
| **Rust**⚙️ | `ahnlich-client-rs` | Uses `tokio` + `tonic`, compatible with `deadpool` pools and full pipeline support |

Each SDK connects over **gRPC** to Ahnlich’s services—make sure at least `ahnlich-db` is running (default port **1369**), and if using AI (`ahnlich-ai`), that’s reachable too (default port **1370**)

Choose your preferred SDK below for installation steps, quickstarts, and links to full docs:

- Go SDK [developer guide](/docs/client-libraries/go) and [API reference](/docs/client-libraries/go/reference)
- Python SDK [developer guide](/docs/client-libraries/python) and [API reference](/docs/client-libraries/python/reference)
- Rust SDK [developer guide](/docs/client-libraries/rust) and [API reference](/docs/client-libraries/rust/reference)

