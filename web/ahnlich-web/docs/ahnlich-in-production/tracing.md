---
title: Tracing
sidebar_position: 20
---

# Distributed Tracing

Ahnlich supports distributed tracing using OpenTelemetry for production observability.

## Quick Enable

Add these flags to enable tracing:

```bash
# For Ahnlich DB
ahnlich-db run \
  --enable-tracing \
  --otel-endpoint http://jaeger:4317

# For Ahnlich AI  
ahnlich-ai run \
  --enable-tracing \
  --otel-endpoint http://jaeger:4317
```

## Docker Compose Example

```yaml
version: "3.8"

services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger UI
      - "4317:4317"    # OTLP gRPC receiver

  ahnlich_db:
    image: ghcr.io/deven96/ahnlich-db:latest
    command: >
      ahnlich-db run --host 0.0.0.0
      --enable-tracing
      --otel-endpoint http://jaeger:4317
    ports:
      - "1369:1369"

  ahnlich_ai:
    image: ghcr.io/deven96/ahnlich-ai:latest
    command: >
      ahnlich-ai run --host 0.0.0.0
      --db-host ahnlich_db
      --enable-tracing
      --otel-endpoint http://jaeger:4317
    ports:
      - "1370:1370"
```

Access Jaeger UI at `http://localhost:16686`

## What Gets Traced

- Client requests and responses
- DB operations (similarity search, indexing)
- AI model inference (embedding generation)
- Inter-service communication (AI → DB)

## Detailed Documentation

For comprehensive tracing setup and configuration:

- **[Distributed Tracing Overview](../components/distributed-tracing/distributed-tracing)** - Architecture and concepts
- **[Using Jaeger](../components/distributed-tracing/using-jaeger)** - Jaeger setup guide
- **[Tracing in Ahnlich DB](../components/distributed-tracing/ahnlich-db)** - DB-specific configuration
- **[Tracing in Ahnlich AI](../components/distributed-tracing/ahnlich-ai)** - AI-specific configuration

## Production Tips

1. **Use a dedicated collector** - Don't send traces directly to Jaeger in production
2. **Sample traces** - Configure sampling to reduce overhead (not yet configurable, coming soon)
3. **Aggregate traces** - Use centralized tracing backends (Jaeger, Tempo, Honeycomb)
4. **Monitor overhead** - Tracing adds ~5-10% latency overhead
