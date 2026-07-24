---
title: Similarity algorithms
sidebar_label: Similarity algorithms
---

# Similarity algorithms

Ahnlich DB compares vectors with several algorithms — two **linear** metrics and
a **non-linear** index (HNSW). The right one depends on your data and query needs.
For the plain-language concepts, see
[Similarity metrics](/docs/concepts/similarity-metrics) and
[Vector index](/docs/concepts/vector-index); this page is the DB-specific,
command-level detail.

## Cosine similarity (linear)

Measures the cosine of the **angle** between two vectors — orientation, not
magnitude.

**Use it for** text embeddings and other high-dimensional data where magnitude
doesn't matter.

```bash
GETSIMN 3 WITH [0.25, 0.88] USING cosinesimilarity IN my_store
```

*Example:* query "What's the capital of France?" against document embeddings —
cosine retrieves the doc most semantically aligned with "Paris".

## Euclidean distance (linear)

Measures the straight-line (L2) **distance** between two vectors — magnitude
matters.

**Use it for** image embeddings and recommendation engines where closeness in
feature space is meaningful.

```bash
GETSIMN 5 WITH [0.12, 0.45] USING euclidean IN image_store
```

*Example:* searching for visually similar product images — a handbag photo returns
similar handbags.

## HNSW (non-linear)

A graph-based **approximate** nearest-neighbour (ANN) search that narrows
candidates through hierarchical layers.

**Use it for** high-dimensional embeddings (100+ dims) and large-scale datasets
where a small recall trade-off buys big speed gains — semantic search,
recommendations, and image retrieval at scale.

### Configuration

| Parameter | Default | Description |
| --- | --- | --- |
| `ef_construction` | 100 | Search breadth while building. Higher = better recall, slower inserts. |
| `maximum_connections` (M) | 48 | Max connections per node above layer 0. Higher = more memory, better recall. |
| `maximum_connections_zero` | 96 | Max connections at layer 0 (typically 2×M). |
| `extend_candidates` | false | Expand the candidate pool with neighbours' neighbours. |
| `keep_pruned_connections` | false | Retain pruned connections for higher connectivity. |
| `distance` | Euclidean | `Euclidean`, `Cosine`, or `DotProduct`. |

```bash
CREATE NON LINEAR ALGORITHM INDEX hnsw IN semantic_store
```

```python
db_client.create_store(
    store="semantic_store",
    dimension=384,
    create_predicates=["category"],
    non_linear_indices=[
        NonLinearIndex(index=HnswConfig(
            distance=DistanceMetric.Cosine,
            ef_construction=200,
            maximum_connections=32,
            maximum_connections_zero=64,
        ))
    ],
    error_if_exists=True,
)
```

```bash
GETSIMN 10 WITH [0.12, 0.45, ...] USING hnsw IN semantic_store
```

### Tuning tips

- **Low recall?** Increase `ef_construction` and `maximum_connections` for a denser
  graph.
- **Slow inserts?** Decrease `ef_construction` for faster builds at the cost of
  recall.
- **Memory constrained?** Lower `maximum_connections`.
- **Bad config?** Drop the index and recreate it — existing data is re-indexed
  automatically.

## Related

- [Choosing & performance](/docs/components/ahnlich-db/advanced/choosing-and-performance)
- [Vector index](/docs/concepts/vector-index)
- [Similarity metrics](/docs/concepts/similarity-metrics)
