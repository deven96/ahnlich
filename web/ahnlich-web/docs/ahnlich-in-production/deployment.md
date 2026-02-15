---
title: Deployment
sidebar_position: 10
---

# Deployment

Ahnlich consists of two services that work together:

- **ahnlich-db**: In-memory vector store with exact similarity search
- **ahnlich-ai**: AI proxy that transforms raw inputs (text/image) into embeddings

The recommended production setup runs both services using Docker.

## Official Docker Images

Ahnlich provides prebuilt images on [GitHub Container Registry](https://github.com/deven96/ahnlich/pkgs/container/ahnlich-db):

- **DB**: `ghcr.io/deven96/ahnlich-db:latest`
- **AI**: `ghcr.io/deven96/ahnlich-ai:latest`

## Docker Compose Setup

The easiest deployment for local or cloud use:

```yaml
version: "3.8"

services:
  ahnlich_db:
    image: ghcr.io/deven96/ahnlich-db:latest
    command: >
      ahnlich-db run --host 0.0.0.0
      --enable-persistence
      --persist-location /root/.ahnlich/data/db.dat
      --persistence-interval 300
    ports:
      - "1369:1369"
    volumes:
      - ./data:/root/.ahnlich/data

  ahnlich_ai:
    image: ghcr.io/deven96/ahnlich-ai:latest
    command: >
      ahnlich-ai run --host 0.0.0.0
      --db-host ahnlich_db
      --enable-persistence
      --persist-location /root/.ahnlich/data/ai.dat
      --persistence-interval 300
    ports:
      - "1370:1370"
    volumes:
      - ./data:/root/.ahnlich/data
      - ./ahnlich_ai_model_cache:/root/.ahnlich/models
```

This configuration:
- Enables disk persistence (data survives restarts)
- Maps ports 1369 (DB) and 1370 (AI)
- Caches AI models across restarts

## Persistence

Without persistence, all data is in-memory and lost on restart. To enable:

```bash
--enable-persistence
--persist-location /root/.ahnlich/data/db.dat
--persistence-interval 300  # seconds
```

Mount the persist location to a host volume:

```yaml
volumes:
  - ./data:/root/.ahnlich/data
```

## Cloud Deployments

### AWS EC2

1. Launch EC2 instance
2. Install Docker
3. Run DB:
   ```bash
   docker run -d \
     --name ahnlich_db \
     -p 1369:1369 \
     -v /data/ahnlich:/root/.ahnlich/data \
     ghcr.io/deven96/ahnlich-db:latest \
     ahnlich-db run --host 0.0.0.0 \
       --enable-persistence \
       --persist-location /root/.ahnlich/data/db.dat
   ```
4. Run AI:
   ```bash
   docker run -d \
     --name ahnlich_ai \
     -p 1370:1370 \
     --link ahnlich_db \
     -v /data/ahnlich:/root/.ahnlich/data \
     -v /data/models:/root/.ahnlich/models \
     ghcr.io/deven96/ahnlich-ai:latest \
     ahnlich-ai run --host 0.0.0.0 \
       --db-host ahnlich_db \
       --enable-persistence \
       --persist-location /root/.ahnlich/data/ai.dat
   ```

Open ports 1369 and 1370 in your security group.

### GCP Compute Engine

1. Create VM instance
2. Install Docker
3. Follow same Docker commands as AWS EC2
4. Create firewall rules for TCP ports 1369 and 1370
5. Mount a persistent disk to `/data` for persistence

### Coolify

[Coolify](https://coolify.io/) is a self-hosted PaaS supporting Docker images.

**Steps:**

1. Create new app → Docker Image
2. Set images:
   - DB: `ghcr.io/deven96/ahnlich-db:latest`
   - AI: `ghcr.io/deven96/ahnlich-ai:latest`
3. Configure run commands:
   - DB: `ahnlich-db run --host 0.0.0.0 --enable-persistence --persist-location /root/.ahnlich/data/db.dat`
   - AI: `ahnlich-ai run --host 0.0.0.0 --db-host ahnlich_db --enable-persistence --persist-location /root/.ahnlich/data/ai.dat`
4. Mount volumes:
   - `/root/.ahnlich/data` (persistence)
   - `/root/.ahnlich/models` (AI model cache)
5. Expose ports 1369 and 1370

### Google Cloud Run

Cloud Run supports gRPC containers with these requirements:

- Containers listen on `$PORT` (use `--port $PORT`)
- Expose endpoints over HTTPS (port 443)
- Configure `ahnlich-ai` with `--db-host <Cloud Run URL>`

See [Cloud Run gRPC Guide](https://cloud.google.com/run/docs/triggering/grpc)

## Production Checklist

| Item | Recommendation |
|------|----------------|
| Ports | Expose 1369 (DB) and 1370 (AI) |
| DB Connection | `ahnlich-ai` must use `--db-host` with reachable address |
| Persistence | Enable with `--enable-persistence` and bind volumes |
| Model Caching | Mount `/root/.ahnlich/models` for AI |
| Tracing | Optional: `--enable-tracing --otel-endpoint <collector>` |
| Security | Use TLS via proxy/load balancer for external exposure |

## References

- [Ahnlich GitHub](https://github.com/deven96/ahnlich)
- [Docker Images](https://github.com/deven96/ahnlich/pkgs/container/ahnlich-db)
- [Docker Compose Example](https://github.com/deven96/ahnlich/blob/main/docker-compose.yml)
- [Coolify Docs](https://docs.coolify.io/)
- [AWS EC2 Docker](https://docs.aws.amazon.com/AmazonECS/latest/developerguide/docker-basics.html)
- [Cloud Run gRPC](https://cloud.google.com/run/docs/triggering/grpc)
