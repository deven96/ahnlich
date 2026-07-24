---
title: What is Ahnlich?
sidebar_label: What is Ahnlich?
sidebar_position: 10
---

# What is Ahnlich?

**Ahnlich is an in-memory vector database with a built-in AI proxy** that embeds
your text, images, and audio for you. Point it at raw content, search by meaning, and
get ranked results — all from a single binary with no external services.

```bash
docker run -d -p 1370:1370 ghcr.io/deven96/ahnlich-ai:latest
```

Ready to try it? Jump straight to the [**Quickstart**](./getting-started/quickstart).

## Why Ahnlich?

- **No embedding pipeline to build.** Send raw text, images, or audio; the AI proxy
  embeds and stores them automatically. No separate model server to run.
- **Runs anywhere, instantly.** A self-contained binary — no cluster, no managed
  service, no cloud dependency. Great for local dev, prototypes, and the edge.
- **Fast semantic search.** RAM-resident vectors with Cosine, Euclidean (L2), or
  Dot Product similarity.
- **Filter while you search.** Attach metadata (author, genre, timestamps…) and
  combine similarity with metadata conditions in one query.
- **Update in place.** Add, change, or delete vectors on the fly — no full index
  rebuilds.
- **Scales when you need it.** Approximate search via HNSW indexes for
  large datasets.
- **Use your language.** Native clients for **Python, Rust, Node, and Go**, plus
  an interactive CLI.

## What can I build with it?

- **Semantic document & FAQ search** — find content by meaning, not keywords.
- **RAG chat memory** — fetch the most relevant context to enrich LLM prompts.
- **Recommendations** — turn users, products, or docs into vectors and rank by
  similarity plus metadata.
- **Code & log search** — surface meaningfully similar snippets, not exact matches.

## How it fits together

Ahnlich ships two services and a CLI:

| Component | What it does |
| --- | --- |
| **`ahnlich-db`** | The in-memory vector store — holds vectors and metadata, runs similarity search. |
| **`ahnlich-ai`** | The AI proxy — turns raw text, images, or audio into embeddings, then talks to the DB for you. |
| **`ahnlich-cli`** | An interactive shell for creating stores, inserting data, and querying. |

Use `ahnlich-ai` when you want automatic embeddings, or talk to `ahnlich-db`
directly if you already have your own vectors.

## Next steps

- [**Quickstart**](./getting-started/quickstart) — first store and search in minutes.
- [**Installation**](./getting-started/installation) — Docker, binaries, or source.
- [**Client Libraries**](./client-libraries) — SDK docs for each language.
