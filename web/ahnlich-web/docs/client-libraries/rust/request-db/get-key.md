---
title: Get Key
---

# Get Key

Retrieve a single stored vector (and its associated metadata) by key from a specified store. Use this request to fetch the exact item you previously inserted with `Set` or to validate the contents of a given key.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::{
      db::query::GetKey,
      keyval::StoreKey,
  };
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // point to your DB server (default is 1369, adjust if needed)
      let addr = "http://127.0.0.1:1369";
      let client = DbClient::new(addr.to_string()).await?;


      // example: look up a key from store "Main"
      let get_key_params = GetKey {
          store: "Main".to_string(),
          keys: vec![
              StoreKey {
                  key: vec![1.2, 1.3, 1.4], // must match a previously Set key
              },
          ],
      };


      match client.get_key(get_key_params, None).await {
          Ok(result) => {
              println!("Fetched: {:#?}", result);
          }
          Err(e) => {
              eprintln!("Error fetching key: {:?}", e);
          }
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `params: GetKey` — Request payload identifying the target store and key. Depending on your server proto, this typically includes:

  * `store` (required) — The store name to query.

  * `key` (required) — The unique identifier for the vector to retrieve.

  * Optional flags (server-dependent) to control returned fields (for example, whether the raw vector or metadata should be included).

* `tracing_id: Option<String>` — Optional trace parent to propagate observability context. When provided, `add_trace_parent` attaches tracing metadata to the outgoing gRPC request.

## Returns
* `Ok(Get)` — A `Get` response containing the requested payload. This commonly includes the vector data, any stored metadata, and the key. The exact fields are defined by the crate’s Get type and server proto.


* `Err(AhnlichError)` — Failure to fetch the key. Typical error cases:

  * Target store not found.

  * Key not found (missing).

  * Permission or authentication failures.

  * Transport-level errors (connection, timeout).

  * Any server-side validation or internal error.

## Behavior
* Read-only RPC with no side effects on server state.

* The request is synchronous in the sense that it returns the current stored value at the time of the call.

* Tracing metadata (if provided) is propagated for observability and debugging.

* The code returns the inner response (`into_inner()`), delivering the `Get` payload directly to the caller.
