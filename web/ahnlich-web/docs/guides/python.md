---
title: 📚 Python Book Search
---

**Source**: [examples/python/book‑search](https://github.com/deven96/ahnlich/tree/main/examples/python/book-search)

This walkthrough guides building a **textual book similarity search** tool using Python:

- Creating a DB or AI Store for embedding book descriptions
- Ingesting book titles and summaries as raw text
- Querying similar books based on semantic relevance
- Filtering results by metadata like genre or author
- Displaying matched books with scores and metadata

## 🔧 What you’ll learn

1. Async setup using `grpclib` and `ahnlich-client-py`.
2. Creating a store and ingesting raw book data.
3. Running `get_sim_n()` on text queries like "space opera classics".
4. Filtering responses by metadata predicates (e.g. `author == "Asimov"`).
5. Retrieving stored metadata and original summaries for output.

## 💡 Highlighted snippet

```python
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai.models import AiModel

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    await client.create_store(ai_query.CreateStore(
        store="books",
        index_model=AiModel.ALL_MINI_LM_L6_V2,
        query_model=AiModel.ALL_MINI_LM_L6_V2,
        predicates=["author", "genre"],
        store_original=True,
        error_if_exists=True
    ))

    # ingest books
    await client.set(ai_query.Set(
        store="books",
        inputs=[
            keyval.AiStoreEntry(
                key=keyval.StoreInput(raw_string="Dune summary..."),
                value=keyval.StoreValue(value={"genre": metadata.MetadataValue(raw_string="SciFi")})
            )
        ],
        preprocess_action=preprocess.PreprocessAction.NoPreprocessing
    ))
```

## ➕ Try it yourself

- Clone the example project  
- Run both Ahnlich services locally  
- Adjust the book dataset (titles / summaries / metadata)  
- Use the Python script to test querying with new text prompts



## 🧠 Why these guides matter

- Show **real usage** of the Go/Python/Rust SDKs in fully working apps  
- Let you **adapt the patterns** for your domain (images, documents, etc.)  
- Emphasize **semantic similarity + metadata filtering** capabilities  
- Provide production-style, real-world code to build upon  

Check out the source on GitHub and follow along as each guide walks you step-by-step through store creation, ingestion, search, and result processing.



## 📌 Next Steps

- Browse the [Rust Image Search](https://github.com/deven96/ahnlich/tree/main/examples/rust/image-search) repo  
- Browse the [Python Book Search](https://github.com/deven96/ahnlich/tree/main/examples/python/book-search) repo  
- Start customizing: swap embeddings, apply new metadata, or change query logic  
- Use these patterns as templates for your own semantic search pipelines