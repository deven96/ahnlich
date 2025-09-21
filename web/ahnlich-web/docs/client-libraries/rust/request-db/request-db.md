---
title: request-db
---
# Request DB

The **Request DB** provides a set of operations for managing vector stores and interacting with their contents. These APIs wrap low-level gRPC calls in a convenient Rust interface, handling request construction, tracing propagation, and response parsing. Together, they enable you to create, query, update, and delete records within the database in a structured and reliable way.

Each operation follows the same execution pattern:

* **Request preparation** — Input parameters are wrapped in a `tonic::Request` object.

* **Execution** — The client forwards the request to the DB service.

* **Response handling** — The response is unwrapped and returned in a typed result.

## Capabilities

With Request DB, you can:

* **Manage stores** — Create, list, and drop vector stores.

* **Insert and update data** — Use `set` to add or modify records.

* **Query data** — Fetch by key, by predicate, or by similarity (`get_key`, `get_pred`, `get_sim_n`).

* **Delete data** — Remove records by key or predicate (`del_key`, `del_pred`).

* **Index management** — Create and drop indexes for predicate and algorithm-based queries.

* **Server & client metadata** — Retrieve cluster information (`info_server`, `list_clients`).

## Behavior

All Request DB operations are designed to:

* **Ensure consistency** — Calls execute atomically on the server side.

* **Support idempotency** — Repeated calls with the same parameters will yield consistent results.

* **Handle concurrency** — Multiple clients can safely read and write without corrupting data.

* **Propagate observability** — Optional tracing IDs allow for full request tracing in distributed environments.

Below are the operations for managing vector stores, storing and retrieving vectors, performing similarity queries, and handling indexes in the Ahnlich database.

* [Ping](/docs/client-libraries/rust/request-db/ping)
* [Info Server](/docs/client-libraries/rust/request-db/info-server)
* [List Stores](/docs/client-libraries/rust/request-db/list-stores)
* [Create Store](/docs/client-libraries/rust/request-db/create-store)
* [Set](/docs/client-libraries/rust/request-db/set)
* [Get Sim N](/docs/client-libraries/rust/request-db/get-simn)
* [Get Key](/docs/client-libraries/rust/request-db/get-key)
* [Get by Predicate](/docs/client-libraries/rust/request-db/get-by-predicate)
* [Create Predicate Index](/docs/client-libraries/rust/request-db/create-predicate-index)
* [Drop Predicate Index](/docs/client-libraries/rust/request-db/drop-predicate-index)
* [Delete Key](/docs/client-libraries/rust/request-db/delete-key)
* [Drop Store](/docs/client-libraries/rust/request-db/drop-store)
* [List Connected Clients](/docs/client-libraries/rust/request-db/list-connected-clients)
* [Create Non-Linear Algorithm Index](/docs/client-libraries/rust/request-db/create-non-linear-algx)
* [Drop Non-Linear Algorithm Index](/docs/client-libraries/rust/request-db/drop-non-linear-algx)
* [Delete By Predicate](/docs/client-libraries/rust/request-db/delete-by-predicate)