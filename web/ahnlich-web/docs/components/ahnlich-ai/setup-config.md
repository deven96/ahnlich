---
title: Setup & Configuration
---

# Setup & Configuration

This section explains how to install, configure, and run **Ahnlich AI**. Ahnlich AI acts as a proxy layer to Ahnlich DB, handling raw input, embedding generation, and semantic querying. It can run standalone or alongside Ahnlich DB, and is available as prebuilt binaries and Docker images.

## Installation

### 1. Download Binaries
Prebuilt binaries are available from GitHub Releases.

#### Download with wget:
```
wget https://github.com/deven96/ahnlich/releases/download/bin%2Fai%2F0.0.0/aarch64-darwin-ahnlich-ai.tar.gz
```

#### Extract the archive:
```
tar -xvzf aarch64-darwin-ahnlich-ai.tar.gz
```

#### Run the binary:
```
./ahnlich-ai
```

Replace `aarch64-darwin-ahnlich-ai.tar.gz` with the correct file for your platform.

### 2. Using Docker
Ahnlich AI also ships as a Docker image:
```
docker pull ghcr.io/deven96/ahnlich-ai:latest
```

Run with:
```
docker run --rm -p 1370:1370 \
  --network ahnlich-net \
  --name ahnlich-ai \
  ghcr.io/deven96/ahnlich-ai:latest \
  ahnlich-ai run --port 1370 --db-url http://ahnlich-db:1369
```

### 3. Example Docker Compose
Ahnlich AI can be orchestrated with docker-compose, typically alongside Ahnlich DB.

  ```
  services:
  ahnlich_ai:
    image: ghcr.io/deven96/ahnlich-ai:latest
    command: >
      "ahnlich-ai run --host 0.0.0.0
      --db-url http://ahnlich_db:1369
      --enable-tracing
      --otel-endpoint http://jaeger:4317"
    ports:
      - "1370:1370"

  ahnlich_db:
    image: ghcr.io/deven96/ahnlich-db:latest
    command: >
      "ahnlich-db run --host 0.0.0.0 --enable-tracing"
    ports:
      - "1369:1369"
  ```

#### Optional Jaeger service for tracing
```
  jaeger:
    image: jaegertracing/all-in-one:${JAEGER_VERSION:-latest}
    ports:
      - "16686:16686"
      - "4317:4317"
      - "4318:4318"
```

## Configuration Options
Ahnlich AI can be customized using runtime flags:

- `--host <ip>` – Specify listening host (default: 0.0.0.0).

- `--port <port>` – Specify server port (default: 1370).

- `--enable-tracing` – Enable telemetry tracing with OpenTelemetry.

- `--otel-endpoint <url>` – OpenTelemetry endpoint (e.g., Jaeger).

- `CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2`

## Quick Start Example

Start Ahnlich AI (with a linked DB):

```
./ahnlich-ai run --host 0.0.0.0 --port 1370 --db-url http://localhost:1369
```

Create a model-aware store:

```
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
```

Insert raw input (text/image):

```
INSERT "The rise of renewable energy storage solutions" INTO my_store
```

Run a semantic query:

```
SEARCH "climate change effects on agriculture" IN my_store
```
