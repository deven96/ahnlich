---
title: Request AI
---

# Request AI

The **Request AI client** provides a set of operations for interacting with the **Ahnlich AI service**, which complements the DB client by handling the generation, transformation, and interpretation of embeddings. Instead of managing storage or retrieval directly, the AI client focuses on **creating meaningful vector representations** from raw data and enabling higher-level reasoning tasks.

Just like the DB client, each operation follows a consistent execution pattern:
* **Request preparation** — Input parameters are wrapped in a `tonic::Request` object.

* **Tracing propagation** — If a `tracing_id` is provided, it is attached for observability.

* **Execution** — The client forwards the request to the AI service.

* **Response handling** — The response is unwrapped and returned in a typed result.

## Capabilities
With Request AI, you can:

* **Generate embeddings** — Convert text, documents, or structured input into dense vector representations.

* **Interpret embeddings** — Extract semantic meaning or similarity insights from stored vectors.

* **Support hybrid workflows** — Combine AI-generated embeddings with DB operations for efficient similarity search.

* **Metadata augmentation** — Enrich vectors with contextual or domain-specific annotations before persistence.

* **Batch processing** — Process multiple inputs in a single request for efficiency.

## Behavior
All Request AI operations are designed to:

* **Ensure consistency** — The same input will always yield the same embedding for reproducibility.

* **Support idempotency** — Repeated requests with identical input and parameters return identical results.

* **Handle concurrency** — Multiple requests can be executed in parallel, ensuring scalability under load.

* **Propagate observability** — Optional tracing IDs allow for debugging and performance monitoring in distributed systems.

Below are the operations for generating embeddings, interpreting inputs, and integrating AI-driven vectors into the Ahnlich ecosystem.

* [Ping](/docs/client-libraries/rust/request-ai/ping)
* [Info Server](/docs/client-libraries/rust/request-ai/info-server)
* [List Stores](/docs/client-libraries/rust/request-ai/list-stores)
* [Create Store](/docs/client-libraries/rust/request-ai/create-store)
* [Set](/docs/client-libraries/rust/request-ai/set)
* [Get Sim N](/docs/client-libraries/rust/request-ai/get-simn)
* [Get Key](/docs/client-libraries/rust/request-ai/get-key)
* [Get by Predicate](/docs/client-libraries/rust/request-ai/get-by-predicate)
* [Create Predicate Index](/docs/client-libraries/rust/request-ai/create-predicate-index)
* [Drop Predicate Index](/docs/client-libraries/rust/request-ai/drop-predicate-index)
* [Delete Key](/docs/client-libraries/rust/request-ai/delete-key)
* [Drop Store](/docs/client-libraries/rust/request-ai/drop-store)
* [List Connected Clients](/docs/client-libraries/rust/request-ai/list-connected-clients)
* [Create Non-Linear Algorithm Index](/docs/client-libraries/rust/request-ai/create-non-linear-algx)
* [Drop Non-Linear Algorithm Index](/docs/client-libraries/rust/request-ai/drop-non-linear-algx)
* [New](/docs/client-libraries/rust/request-ai/new)
* [Purge Stores](/docs/client-libraries/rust/request-ai/purge-stores)