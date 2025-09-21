---
title: Get Sim N
---

# Get Sim N

The `GetSimN` request retrieves the **top-N most similar vectors** to a given query vector from a specified store. This is the core similarity search operation in Ahnlich DB, allowing you to find nearest neighbors by vector distance.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::{
      algorithm::algorithms::Algorithm,
      db::query::GetSimN,
      keyval::StoreKey,
  };
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // connect to server
      let addr = "http://127.0.0.1:1369"; // adjust to your server address
      let db_client = DbClient::new(addr.to_string()).await?;


      // prepare parameters
      let params = GetSimN {
          store: "Main".to_string(),
          search_input: Some(StoreKey { key: vec![1.0, 2.0, 3.0] }),
          closest_n: 2,
          algorithm: Algorithm::EuclideanDistance as i32,
          condition: None,
      };


      // call get_sim_n
      let response = db_client.get_sim_n(params, None).await?;
      println!("Response: {:?}", response);


      Ok(())
  }

  ```
</details>

## Parameters
* `params: GetSimN` – Defines the similarity search query. This includes:

  * `store` – The name of the target store.

  * `vector` – The query vector (`Vec<f32>`) used to compute similarity.

  * `n` – The number of nearest neighbors to return.

  * `predicate` (optional) – A filter expression applied to metadata before similarity is calculated.


* `tracing_id: Option<String>` – Optional identifier for distributed tracing.


## Returns
* `Ok(GetSimNResult)` – A structured result containing:

  * `matches` – A ranked list of vectors with associated similarity scores, keys, and metadata.

  * `count` – The number of results returned (up to `n`).


* `Err(AhnlichError)` – Possible error cases:

  * Store not found.

  * Query vector does not match store dimensionality.

  * Invalid or malformed predicate filter.

  * Transport or server-side errors.


## Behavior
* The search is **approximate or exact** depending on the index configuration of the store.

* Returned results are sorted in descending order of similarity (most similar first).

* If a predicate filter is provided, only vectors satisfying the filter are considered for similarity ranking.

* Results include both the vector key and any metadata if available.


