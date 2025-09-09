---
title: Bulk Requests
sidebar_position: 4
---

# Bulk Requests

The AI client supports **bulk requests**, allowing you to send multiple operations at once. Instead of sending each request individually, you can batch them using a **pipeline builder**.

Bulk requests are executed **sequentially** in the order they were added. The client automatically collects all responses and returns them as a single aggregated result.

## Source Code

<details>
  <summary>Click to expand</summary>

  ```py
  from ahnlich_client_py import AhnlichAIClient

  client = AhnlichAIClient(address="127.0.0.1", port=1370)

  # Create a pipeline builder
  request_builder = client.pipeline()

  # Queue multiple requests
  request_builder.ping()
  request_builder.info_server()
  request_builder.list_clients()
  request_builder.list_stores()

  # Execute the pipeline
  response = client.exec()
  ```
</details>

## Explanation

* **Pipeline builder**:
 Collects multiple requests before sending them to the AI service.

* **Sequential execution**:
 Requests are executed one after the other, preserving the order in which they were added.

* **Aggregated response**:
 The result is a list of individual responses, one for each request in the pipeline.

* **Use case**:

  * Reduce network overhead by batching requests.

  * Efficiently run related queries in one execution cycle.

  * Simplify client code when multiple calls are always needed together.

Example responses might include:

* A `Ping` acknowledgment.

* Server info metadata.

* Connected clients.

* Available stores.


