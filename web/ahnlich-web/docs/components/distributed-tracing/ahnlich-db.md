---
title: Tracing in Ahnlich DB
---

# Tracing in Ahnlich DB

### Why would you want to trace?
With DB tracing, you can:

- See **latency** of operations like `SET`, `GETPRED`, `DROPSTORE`.

- Debug issues in **index updates** and **store lifecycle events**.

- Verify persistence operations and background tasks.

- Observe how the DB behaves when called by AI for predicate filtering.


### How we support distributed tracing

- Enable tracing: `--enable-tracing`.

- Configure endpoint: `--otel-endpoint http://jaeger:4317.`

- Each DB action generates **spans** with metadata (execution time, error info, input sizes).

### Example docker-compose.yaml
```
ahnlich_db:
  image: ghcr.io/deven96/ahnlich-db:latest
  command: >
    "ahnlich-db run --host 0.0.0.0 \
    --enable-tracing \
    --otel-endpoint http://jaeger:4317"
  ports:
    - "1369:1369"
```

## Viewing DB Traces in Jaeger

1. Open http://localhost:16686 in your browser.

2. Select `service = ahnlich_db`.

3. Click **Find Traces**.

4. You’ll see entries for DB actions (e.g., `CREATESTORE`, `GETPRED`).

5. Click a trace → expand spans to view details such as:

    - Query time

    - Predicate filtering logic

    - Errors or warnings
