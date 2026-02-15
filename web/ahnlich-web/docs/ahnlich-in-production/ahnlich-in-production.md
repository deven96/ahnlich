---
title: ⚡Ahnlich in production
sidebar_position: 50
---

# Ahnlich in Production

Ahnlich is production-ready and designed for deployment at scale. This section covers everything you need to run Ahnlich in production environments.

## What You'll Learn

- **[Deployment](./deployment)** - Docker-based deployment strategies for cloud and on-premise
- **[Tracing](./tracing)** - Distributed tracing setup for observability

## Architecture Overview

A typical production setup consists of:

```
┌─────────────┐
│   Clients   │
└──────┬──────┘
       │
       ├──────────────────────┐
       │                      │
       ▼                      ▼
┌─────────────┐        ┌─────────────┐
│ Ahnlich AI  │───────>│ Ahnlich DB  │
│  (Port 1370)│        │ (Port 1369) │
└─────────────┘        └─────────────┘
```

- **Ahnlich DB** handles vector storage and similarity search
- **Ahnlich AI** transforms inputs (text/images) into embeddings
- Both services communicate over gRPC

## Key Features for Production

### Persistence
Both services support disk persistence to survive restarts:
- Configurable intervals for snapshots
- Automatic recovery on startup

### Performance
- In-memory operations for low latency
- Batch processing support
- Configurable model batch sizes

### Observability
- Distributed tracing with OpenTelemetry
- Integration with Jaeger and other collectors
- Request/response logging

### Scalability
- Horizontal scaling via multiple instances
- Load balancing support
- Model caching to reduce startup time

## Quick Start

Get started with Docker Compose:

```bash
curl -O https://raw.githubusercontent.com/deven96/ahnlich/main/docker-compose.yml
docker-compose up -d
```

This starts both services with persistence enabled.

## Next Steps

1. **[Deploy to Production](./deployment)** - Choose your deployment platform
2. **[Enable Tracing](./tracing)** - Set up observability
3. Review the [CLI reference](../components/ahnlich-cli/ahnlich-cli) for configuration options
