---
title: Choosing & performance
sidebar_label: Choosing & performance
---

# Choosing an algorithm & performance

How the [similarity algorithms](/docs/components/ahnlich-db/advanced/similarity-algorithms)
compare, and how they behave under real workloads.

## Choosing the right algorithm

| Algorithm | Best for | Pros | Cons |
| --- | --- | --- | --- |
| Cosine similarity | NLP, semantic search | Ignores magnitude, fast | Not for magnitude-based data |
| Euclidean distance | Images, structured numeric features | Intuitive, uses magnitude | Slower in very high dims |
| HNSW | High-dim, large-scale datasets | Fast ANN, tunable recall/speed | Approximate, higher memory |

## Performance & trade-offs

Ahnlich DB is optimized for real-time similarity search, but algorithms behave
differently by **data size, dimensionality, and query type**.

### Cosine similarity

- **Speed:** very fast (linear scan). **Accuracy:** high for semantic embeddings.
  **Memory:** moderate (normalization).
- *Benchmark (example):* 1M text embeddings (768-dim BERT) → ~15 ms avg latency
  (16-core CPU, in-memory), 95% recall@10 vs brute-force.

### Euclidean distance

- **Speed:** similar to cosine, slightly heavier math. **Accuracy:** high when
  magnitude matters. **Memory:** higher if embeddings aren't normalized.
- *Benchmark:* 5M product images (512-dim CLIP) → ~25 ms, 93% recall@10.

### HNSW

- **Speed:** sub-millisecond even on large datasets. **Accuracy:** approximate but
  tunable. **Memory:** higher (graph structure).
- *Benchmark:* 10K SIFT vectors (128-dim) → &lt;1 ms, 90%+ recall@50 (default),
  higher when tuned.
- **Limitation:** approximate; quality depends on configuration.

## Summary

| Algorithm | Speed | Accuracy | Best use case | Weakness |
| --- | --- | --- | --- | --- |
| Cosine similarity | Fast | High (95%) | Semantic search (NLP, docs) | Ignores magnitude |
| Euclidean distance | Moderate | High (93%) | Image search, recommendations | Slower in high dims |
| HNSW | Ultra-fast (any dim) | Tunable (80–99%) | Large-scale high-dim search | Approximate, more memory |

## Related

- [Similarity algorithms](/docs/components/ahnlich-db/advanced/similarity-algorithms)
- [Command deep dive](/docs/components/ahnlich-db/advanced/command-deep-dive)
