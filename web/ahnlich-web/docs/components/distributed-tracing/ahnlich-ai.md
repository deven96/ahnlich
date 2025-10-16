---
title: Tracing in Ahnlich AI
---

# Tracing in Ahnlich AI

### Why would you want to trace?
With AI tracing, you can:

- Measure **embedding computation** times for different models (e.g., `resnet-50` vs `all-MiniLM`).

- Track full **similarity searches** (`GETSIMN`) including preprocessing and DB lookups.

- Debug performance issues with **execution providers** or **index algorithms**.

- Correlate AI spans with DB spans for full pipeline visibility.

### How we support distributed tracing

- Enable tracing: `--enable-tracing`.

- Forward spans: `--otel-endpoint http://jaeger:4317`.

- Each AI action generates spans for:

  - Preprocessing

  - Embedding generation

  - Similarity algorithms

  - Downstream DB calls

### Example docker-compose.yaml
```
ahnlich_ai:
  image: ghcr.io/deven96/ahnlich-ai:latest
  command: >
    "ahnlich-ai run --db-host ahnlich_db --host 0.0.0.0 \
    --supported-models all-minilm-l6-v2,resnet-50 \
    --enable-tracing \
    --otel-endpoint http://jaeger:4317"
  ports:
    - "1370:1370"
```

## Viewing AI Traces in Jaeger

1. Open http://localhost:16686.

2. Select `service = ahnlich_ai`.

3. Click **Find Traces**.

4. Example trace breakdown for `GETSIMN`:

    - Span 1: Request received by AI.

    - Span 2: Preprocessing (`RawString → embedding input`).

    - Span 3: Embedding model execution (`resnet-50`).

    - Span 4: Similarity algorithm execution (`cosinesimilarity`).

    - Span 5: Predicate filtering in **Ahnlich DB** (cross-service span).

This gives you a **timeline view** of the query, where each bar represents time spent in different stages.

