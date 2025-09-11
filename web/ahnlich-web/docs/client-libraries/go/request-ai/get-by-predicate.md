---
title: Get by Predicate
---

# Get by Predicate

## Description

The **GetByPredicate** request-ai retrieves entries from an AI-managed store by applying **metadata-based filtering**. Unlike similarity search (`GetSimN`), which operates on vector embeddings, predicate queries operate on **stored metadata attributes**. This enables developers to fetch records that satisfy specific metadata conditions, regardless of their embedding similarity.

## Source Code Example

<details>
  <summary>Click to expand source code</summary>

```go
package main

```

</details>

## What the code does

Constructs a **predicate condition** that checks if the metadata field `"f"` equals the raw string `"v"`.

Issues a `GetByPredicate` request against "`ai_store`".

On success, prints the retrieved entries (`resp.Entries`); otherwise, returns the error.

## Behavior

- The AI proxy evaluates the condition against stored entries’ metadata.

- Only records with metadata key `"f"` equal to `"v"` are returned.

- Does not perform embedding similarity—this is purely metadata-based retrieval.

- If no matching entries exist, the response is valid but empty.

- Useful for filtering results in combination with embedding-based searches.
