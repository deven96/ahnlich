---
title: Go
sidebar_position: 10
---

<!-- import GoIcon from '@site/static/img/icons/lang/go.svg' -->

# ⚙️ Ahnlich Go SDK

Official Go client for Ahnlich similarity‑search engine, providing idiomatic access to both **DB** (exact vector search) and **AI** (semantic, embedding‑based search) stores. Requires a running Ahnlich backend (`ahnlich-db` at port 1369 and/or `ahnlich-ai` at port 1370).

Visit the source and reference: [GitHub/ahnlich-client-go](https://github.com/deven96/ahnlich/tree/main/sdk/ahnlich-client-go)

---

## ⚙️ Installation

Ensure you have Go ≥ 1.20 and a running Ahnlich backend:

```bash
go get github.com/deven96/ahnlich/sdk/ahnlich-client-go@latest
```
Import modules in your code:

```
go
import (
    "context"
    ahn "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
)
```

🧠 Connecting to DB / AI Store
You need to establish separate clients:

```go
// DB client
dbClient, err := ahn.NewDbClient("127.0.0.1", 1369)
if err != nil { log.Fatalf(...) }
defer dbClient.Close()

// AI proxy client
aiClient, err := ahn.NewAiClient("127.0.0.1", 1370)
if err != nil { log.Fatalf(...) }
defer aiClient.Close()
```

This uses gRPC to connect to the Ahnlich backend. Always check .Close() on exit to clean up resources.

🧱 Creating a DB Store
**What's a DB Store?**
A DB Store is a fixed-dimension embedding container for exact nearest‑neighbor search using Cosine, Euclidean (L2), or DotProduct metrics. You choose the vector dimension upfront, and can optionally configure metadata predicate indexes to accelerate filtering.

To create:

```go

```

This will register the store with your chosen embedding size and default similarity algorithm.

🧠 Creating an AI Store
**What's an AI Store?**
An AI Store simplifies semantic search by accepting raw input (text or images), converting it to embeddings during ingestion and querying. It requires two models:

– IndexModel: used when ingesting raw data.
– QueryModel: used when computing embeddings for search queries.

These models can be identical (e.g. AIModel_ALL_MINI_LM_L6_V2) but must produce the same embedding dimension, such as 768. This flexibility allows selecting different pipelines for indexing vs querying without breaking compatibility.

All original inputs and metadata are preserved for retrieval alongside results.

To create:

```go

```

💾 Storing Entries
In a DB Store:
```go

```

In an AI Store (raw ingestion):
```go
```

If storing images, replace RawText with RawImage (of type []byte).

🔍 Searching for Closest Matches
Both DB and AI stores expose GetSimN() for similarity search. Only linear search algorithms (Cosine, Euclidean (L2), DotProduct) are supported. Approximate indexing (e.g. HNSW, locality‑sensitive hashing) is on the roadmap but not yet available 
github.com
.

DB Store search:
```go

```

AI Store search by query text:
```go

```

🧩 Using Metadata Filtering
You can narrow search results using predicates on metadata—for both DB and AI stores:

```go

```

Filter is applied after similarity ranking, so you still retrieve top‑N relevant items that meet the predicate.

🧹 Dropping a Store
Remove a namespace when you're done:

```go
if err := dbClient.DropStore(ctx, "my_util_store"); err != nil {
log.Fatalf("drop DB store: %v", err)
}

if err := aiClient.DropStore(ctx, "my_semantic_store"); err != nil {
log.Fatalf("drop AI store: %v", err)
}
```

🧷 Quick Comparison
Operation	DB Store	AI Store
Connect	NewDbClient(...)	NewAiClient(...)
Create Store	CreateStore(…, Dimension)	CreateStore(…, Index/Query models)
Ingestion	Set(Store, Key, Vector, Metadata)	Set(Store, Key, RawText/RawImage, Metadata)
Search	GetSimN(Input, Algorithm)	GetSimN(QueryText, Algorithm)
Metadata Filtering	Supported via predicate	Supported via predicate
Store Deletion	DropStore(...)	DropStore(...)

🚦 Tips & Best Practices
Consistent dimensions: All vectors and models must use the same dimension (e.g. 128, 768).

Index predicates: You can build predicate indexes using the CreatePredicateIndex() method for better filtering performance.


Raw data retrieval: Results include Metadata and (for AI stores) the original RawText or RawImage so you can surface or log full context.

AI store only: auto-embeddings via your configured models—no need to compute embeddings manually.

DB store only: good when you already have vectors, or use your own embedding pipeline.

