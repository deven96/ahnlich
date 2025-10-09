---
title: Installation
sidebar_position: 20
---

# Installation

## 1. Download Binaries
Prebuilt binaries are available from GitHub Releases.

#### Download with `wget`:
```go
wget 
https://github.com/deven96/ahnlich/releases/download/bin%2Fdb%2F0.0.0/aarch64-darwin-ahnlich-db.tar.gz
```

#### Extract the archive:
```go
tar -xvzf aarch64-darwin-ahnlich-db.tar.gz
```

#### Run the binary:
```go
./ahnlich-db
```

Replace `aarch64-darwin-ahnlich-db.tar.gz` with the correct file for your platform.

## 2. Using Docker
Ahnlich DB also ships as a Docker image:
```go
docker pull ghcr.io/deven96/ahnlich-db:latest
```

#### Run with:
```go
docker run --rm -p 1369:1369 \
  --name ahnlich-db \
  ghcr.io/deven96/ahnlich-db:latest \
  ahnlich-db run --port 1369
```

## 3. Example Docker Compose

You can orchestrate Ahnlich DB with **docker-compose**.

#### Basic (with tracing enabled):
```go
services:
  ahnlich_db:
    image: ghcr.io/deven96/ahnlich-db:latest
    command: >
      "ahnlich-db run --host 0.0.0.0
      --enable-tracing
      --otel-endpoint http://jaeger:4317"
    ports:
      - "1369:1369"

  # Optional Jaeger service for tracing
  jaeger:
    image: jaegertracing/all-in-one:${JAEGER_VERSION:-latest}
    ports:
      - "16686:16686"
      - "4317:4317"
      - "4318:4318"
``` 

#### With Persistence:
```go
services:
  ahnlich_db:
    image: ghcr.io/deven96/ahnlich-db:latest
    command: >
      "ahnlich-db run --host 0.0.0.0
      --enable-persistence --persist-location /root/.ahnlich/data/db.dat
      --persistence-interval 300"
    ports:
      - "1369:1369"
    volumes:
      - "./data/:/root/.ahnlich/data" # Persistence Location
```

#### Configuration Options

Ahnlich DB can be customized using runtime flags:

- `--host <ip>` - Specify listening host (default: `0.0.0.0`).


- `--port <port>` - Specify server port (default: `1369`).


- `--enable-tracing` - Enable telemetry tracing with OpenTelemetry.


- `--otel-endpoint <url>` - OpenTelemetry endpoint (e.g., Jaeger).


- `--enable-persistence` - Enable snapshot persistence to disk.


- `--persist-location <path>` - File location for persistence (default: ~/.ahnlich/data/db.dat).


- `--persistence-interval <seconds>` - Interval in seconds between snapshots.


### Quick Start Example

#### Start a simple database:
```go
./ahnlich-db run --host 0.0.0.0 --port 1369
```

#### Create a store:
```go
CREATE STORE my_store DIMENSION 2 ALGORITHM cosine
```

#### Insert a vector with metadata:
```go
INSERT [0.2, 0.1] WITH { "page": "home" } IN my_store
```

#### Run a similarity search:
```go
GETSIMN 2 WITH [0.2, 0.1] USING cosinesimilarity IN my_store WHERE (page != hidden)
```
