---
title: Configuration Reference
sidebar_position: 20
---

# Configuration Reference

Complete reference for all configuration options in Ahnlich DB and AI.

## Environment Variables

**Note:** Ahnlich currently does not use environment variables for configuration. All settings must be provided via CLI flags.

## Configuration Files

**Note:** Ahnlich does not currently support configuration files. All settings are provided as command-line arguments.

---

## Database Server (ahnlich-db)

Start the database server with:
```bash
ahnlich-db run [OPTIONS]
```

### Server Options

#### `--host`
- **Type:** String
- **Default:** `"127.0.0.1"`
- **Description:** Server host address to bind to
- **Examples:**
  ```bash
  --host 127.0.0.1  # Localhost only
  --host 0.0.0.0    # All interfaces
  --host 192.168.1.10  # Specific IP
  ```

#### `--port`
- **Type:** u16 (unsigned 16-bit integer)
- **Default:** `1369`
- **Description:** Database server port
- **Examples:**
  ```bash
  --port 1369  # Default
  --port 8080  # Custom port
  ```

---

### Memory Management

#### `--allocator-size`
- **Type:** usize (bytes)
- **Default:** `10,737,418,240` (10 GiB)
- **Minimum:** `10,485,760` (10 MiB)
- **Description:** Global allocator size in bytes - maximum memory for vector storage
- **Validation:** 
  - Must be ≥ 10 MiB
  - Should be >2x persistence file size if persistence is enabled
- **Examples:**
  ```bash
  --allocator-size 10737418240      # 10 GiB (default)
  --allocator-size 21474836480      # 20 GiB
  --allocator-size 107374182400     # 100 GiB
  ```
- **Calculation Helper:**
  ```
  1 GiB = 1,073,741,824 bytes
  1 MiB = 1,048,576 bytes
  ```

#### `--message-size`
- **Type:** usize (bytes)
- **Default:** `10,485,760` (10 MiB)
- **Description:** Maximum gRPC message size
- **Examples:**
  ```bash
  --message-size 10485760     # 10 MiB (default)
  --message-size 104857600    # 100 MiB
  ```

---

### Persistence

#### `--enable-persistence`
- **Type:** bool (flag)
- **Default:** `false`
- **Description:** Enable data persistence to disk
- **Example:**
  ```bash
  --enable-persistence
  ```

#### `--persist-location`
- **Type:** PathBuf (file path)
- **Default:** None
- **Required:** Yes, if `--enable-persistence` is set
- **Description:** Path to persistence file
- **Examples:**
  ```bash
  --persist-location /var/lib/ahnlich/db.dat
  --persist-location ~/ahnlich-data/db.dat
  --persist-location ./data/persistence.dat
  ```

#### `--persistence-interval`
- **Type:** u64 (milliseconds)
- **Default:** `300000` (5 minutes)
- **Description:** How often to save data to disk (in milliseconds)
- **Examples:**
  ```bash
  --persistence-interval 300000   # 5 minutes (default)
  --persistence-interval 60000    # 1 minute
  --persistence-interval 600000   # 10 minutes
  ```
- **Calculation Helper:**
  ```
  1 second  = 1,000 ms
  1 minute  = 60,000 ms
  5 minutes = 300,000 ms
  ```

#### `--fail-on-startup-if-persist-load-fails`
- **Type:** bool (flag)
- **Default:** `false`
- **Description:** Whether to crash on startup if persistence load fails
- **Examples:**
  ```bash
  --fail-on-startup-if-persist-load-fails  # Fail loudly
  # Omit flag to continue without persistence on load failure
  ```

---

### Networking

#### `--maximum-clients`
- **Type:** usize
- **Default:** `1000`
- **Description:** Maximum concurrent client connections
- **Examples:**
  ```bash
  --maximum-clients 1000   # Default
  --maximum-clients 5000   # Higher limit
  --maximum-clients 100    # Lower limit
  ```

---

### Performance

#### `--threadpool-size`
- **Type:** usize
- **Default:** `16`
- **Description:** CPU thread pool size for request handling
- **Recommendation:** Set to number of CPU cores or slightly higher
- **Examples:**
  ```bash
  --threadpool-size 16  # Default
  --threadpool-size 32  # For 32-core system
  --threadpool-size 8   # For 8-core system
  ```

---

### Observability

#### `--enable-tracing`
- **Type:** bool (flag)
- **Default:** `false`
- **Description:** Enable OpenTelemetry distributed tracing
- **Example:**
  ```bash
  --enable-tracing
  ```

#### `--otel-endpoint`
- **Type:** String (URL)
- **Default:** None
- **Required:** Yes, if `--enable-tracing` is set
- **Description:** OpenTelemetry collector endpoint (gRPC)
- **Examples:**
  ```bash
  --otel-endpoint http://localhost:4317
  --otel-endpoint http://jaeger:4317
  --otel-endpoint http://192.168.1.10:4317
  ```

#### `--log-level`
- **Type:** String (log filter)
- **Default:** `"info,hf_hub=warn"`
- **Description:** Log level configuration using env_logger syntax
- **Valid Levels:** `error`, `warn`, `info`, `debug`, `trace`
- **Examples:**
  ```bash
  --log-level info                           # All modules: info
  --log-level debug                          # All modules: debug
  --log-level "info,ahnlich_db=debug"       # DB debug, others info
  --log-level "warn,ahnlich_db=trace"       # DB trace, others warn
  --log-level "error,hf_hub=off"            # Silence HuggingFace logs
  ```

---

### Complete DB Example

```bash
ahnlich-db run \
  --host 0.0.0.0 \
  --port 1369 \
  --allocator-size 21474836480 \
  --enable-persistence \
  --persist-location /var/lib/ahnlich/db.dat \
  --persistence-interval 300000 \
  --maximum-clients 2000 \
  --threadpool-size 32 \
  --enable-tracing \
  --otel-endpoint http://jaeger:4317 \
  --log-level "info,ahnlich_db=debug"
```

---

## AI Proxy Server (ahnlich-ai)

Start the AI proxy with:
```bash
ahnlich-ai run [OPTIONS]
```

### Server Options

#### `--host`
Same as DB server (default: `"127.0.0.1"`)

#### `--port`
- **Type:** u16
- **Default:** `1370`
- **Description:** AI proxy server port
- **Example:**
  ```bash
  --port 1370  # Default
  --port 8081  # Custom port
  ```

---

### Database Connection

#### `--without-db`
- **Type:** bool (flag)
- **Default:** `false`
- **Description:** Start AI proxy without connecting to database (standalone mode)
- **Conflicts With:** `--db-host`, `--db-port`, `--db-https`, `--db-client-pool-size`
- **Example:**
  ```bash
  ahnlich-ai run --without-db
  ```

#### `--db-host`
- **Type:** String
- **Default:** `"127.0.0.1"`
- **Description:** Ahnlich Database host to connect to
- **Conflicts With:** `--without-db`
- **Examples:**
  ```bash
  --db-host 127.0.0.1
  --db-host ahnlich-db  # Docker service name
  --db-host 192.168.1.10
  ```

#### `--db-port`
- **Type:** u16
- **Default:** `1369`
- **Description:** Ahnlich Database port
- **Conflicts With:** `--without-db`
- **Example:**
  ```bash
  --db-port 1369  # Default
  --db-port 1400  # Custom DB port
  ```

#### `--db-https`
- **Type:** bool (flag)
- **Default:** `false`
- **Description:** Use HTTPS for database connection
- **Conflicts With:** `--without-db`
- **Example:**
  ```bash
  --db-https  # Use https:// instead of http://
  ```

#### `--db-client-pool-size`
- **Type:** usize
- **Default:** `10`
- **Description:** Number of database client connections in the pool
- **Conflicts With:** `--without-db`
- **Recommendation:** Increase for high-concurrency scenarios
- **Examples:**
  ```bash
  --db-client-pool-size 10  # Default
  --db-client-pool-size 50  # Higher concurrency
  ```

---

### AI Models

#### `--supported-models`
- **Type:** Comma-separated list
- **Default:** All models (see table below)
- **Description:** Which AI models to load and support
- **Examples:**
  ```bash
  # Load only specific models
  --supported-models all-minilm-l6-v2,resnet-50
  
  # Load text models only
  --supported-models all-minilm-l6-v2,all-minilm-l12-v2,bge-base-en-v1.5
  
  # Load all models (default)
  # Omit flag or list all
  ```

**Supported Models:**

| Model Name | Type | Max Tokens | Image Size | Embedding Dim | Use Case |
|------------|------|-----------|------------|---------------|----------|
| `all-minilm-l6-v2` | Text | 256 | N/A | 384 | Fast sentence embeddings |
| `all-minilm-l12-v2` | Text | 256 | N/A | 384 | Better sentence embeddings |
| `bge-base-en-v1.5` | Text | 512 | N/A | 768 | General text embedding |
| `bge-large-en-v1.5` | Text | 512 | N/A | 1024 | High-quality text embedding |
| `resnet-50` | Image | N/A | 224x224 | 2048 | Image classification features |
| `clip-vit-b32-image` | Image | N/A | 224x224 | 512 | Visual embeddings |
| `clip-vit-b32-text` | Text | 77 | N/A | 512 | Text for image-text matching |

#### `--ai-model-idle-time`
- **Type:** u64 (seconds)
- **Default:** `300` (5 minutes)
- **Description:** How long to keep models in memory before unloading (when idle)
- **Examples:**
  ```bash
  --ai-model-idle-time 300   # 5 minutes (default)
  --ai-model-idle-time 600   # 10 minutes
  --ai-model-idle-time 60    # 1 minute
  --ai-model-idle-time 0     # Never unload
  ```

#### `--model-cache-location`
- **Type:** PathBuf (directory path)
- **Default:** `~/.ahnlich/models`
- **Description:** Directory where model artifacts are cached
- **Examples:**
  ```bash
  --model-cache-location ~/.ahnlich/models      # Default
  --model-cache-location /var/lib/ahnlich/models
  --model-cache-location ./models
  ```
- **Note:** Models are downloaded from HuggingFace Hub on first use

---

### Performance Options

#### `--session-profiling`
- **Type:** bool (flag)
- **Default:** `false`
- **Description:** Enable ONNX Runtime session profiling
- **Use:** For performance debugging and optimization
- **Example:**
  ```bash
  --session-profiling
  ```

#### `--enable-streaming`
- **Type:** bool (flag)
- **Default:** `false`
- **Description:** Decode images in chunks (reduces memory by 10x but 40% slower)
- **Use:** When processing many large images with limited memory
- **Example:**
  ```bash
  --enable-streaming
  ```

---

### Memory, Persistence, Networking, Observability

AI proxy supports all the same options as DB server:
- `--allocator-size`
- `--message-size`
- `--enable-persistence`
- `--persist-location`
- `--persistence-interval`
- `--fail-on-startup-if-persist-load-fails`
- `--maximum-clients`
- `--threadpool-size`
- `--enable-tracing`
- `--otel-endpoint`
- `--log-level`

See DB Server section above for details.

---

### Complete AI Example

```bash
ahnlich-ai run \
  --host 0.0.0.0 \
  --port 1370 \
  --db-host ahnlich-db \
  --db-port 1369 \
  --db-client-pool-size 20 \
  --supported-models all-minilm-l6-v2,bge-base-en-v1.5,resnet-50 \
  --ai-model-idle-time 600 \
  --model-cache-location /var/lib/ahnlich/models \
  --enable-streaming \
  --allocator-size 21474836480 \
  --enable-persistence \
  --persist-location /var/lib/ahnlich/ai.dat \
  --maximum-clients 2000 \
  --enable-tracing \
  --otel-endpoint http://jaeger:4317 \
  --log-level "info,ahnlich_ai=debug,hf_hub=warn"
```

---

## CLI Client (ahnlich)

Interactive CLI tool for querying Ahnlich:

```bash
ahnlich [OPTIONS]
```

### Options

#### `--agent`
- **Type:** Enum (DB or AI)
- **Required:** Yes
- **Description:** Which server type to connect to
- **Valid Values:** `DB`, `AI`
- **Examples:**
  ```bash
  ahnlich --agent DB
  ahnlich --agent AI
  ```

#### `--host`
- **Type:** String
- **Default:** `"127.0.0.1"`
- **Description:** Server host to connect to
- **Examples:**
  ```bash
  --host 127.0.0.1
  --host localhost
  --host ahnlich-db
  ```

#### `--port`
- **Type:** u16
- **Default:** Auto-selected based on agent (DB=1369, AI=1370)
- **Description:** Server port to connect to
- **Examples:**
  ```bash
  # Defaults
  ahnlich --agent DB              # Connects to :1369
  ahnlich --agent AI              # Connects to :1370
  
  # Custom ports
  ahnlich --agent DB --port 8080
  ahnlich --agent AI --port 8081
  ```

### CLI Examples

```bash
# Connect to DB locally
ahnlich --agent DB --host 127.0.0.1 --port 1369

# Connect to AI proxy
ahnlich --agent AI --host 127.0.0.1 --port 1370

# Connect to remote server
ahnlich --agent DB --host 192.168.1.10 --port 1369
```

---

## Algorithm Configuration

### Similarity Algorithms

Used in `GetSimN` and similar operations:

| Algorithm | Type | Use Case | Characteristics |
|-----------|------|----------|-----------------|
| `EuclideanDistance` | Linear | Absolute distance | Best for comparing magnitudes |
| `DotProductSimilarity` | Linear | Fast comparison | Best when vectors are normalized |
| `CosineSimilarity` | Linear | Direction-based | Best for normalized vectors, ignores magnitude |
| `KDTree` | Non-linear | Fast spatial search | Best for high-dimensional nearest neighbor |

**Usage:**
```
# Linear algorithms - available by default
GETSIMN 10 WITH [1.0, 2.0, 3.0] USING cosinesimilarity IN my_store

# Non-linear - must be created
CREATESTORE my_store DIMENSION 128 NONLINEARALGORITHMINDEX (KDTree)
GETSIMN 10 WITH [1.0, 2.0, 3.0] USING kdtree IN my_store
```

---

### Preprocessing Options

For AI queries:

| Option | Description | When to Use |
|--------|-------------|-------------|
| `NoPreprocessing` | Skip preprocessing | Input already preprocessed |
| `ModelPreprocessing` | Apply model's preprocessing | Raw inputs (recommended) |

**Usage:**
```python
Set(
    store="my_store",
    inputs=[...],
    preprocess_action=PreprocessAction.ModelPreprocessing,
)
```

---

### Execution Providers

Hardware acceleration for AI models:

| Provider | Description | Requirements |
|----------|-------------|--------------|
| `TensorRT` | NVIDIA TensorRT | CUDA ≥12, TensorRT |
| `CUDA` | NVIDIA CUDA | CUDA ≥12, libcudnn9 |
| `DirectML` | DirectX ML | Windows, DirectX 12 |
| `CoreML` | Apple CoreML | macOS, Apple Silicon (not recommended for NLP) |

**Usage:**
```python
GetSimN(
    store="my_store",
    search_input=...,
    closest_n=10,
    algorithm=Algorithm.CosineSimilarity,
    preprocess_action=PreprocessAction.ModelPreprocessing,
    execution_provider=ExecutionProvider.CUDA,  # GPU acceleration
)
```

---

## Validation Rules

### Allocator Size
- Minimum: 10 MiB (10,485,760 bytes)
- With persistence: Must be >2x persistence file size
- Recommended: Based on expected data volume

### Persistence
- `--persist-location` required if `--enable-persistence` is set
- Parent directory must exist and be writable
- File should be on fast storage (SSD recommended)

### Tracing
- `--otel-endpoint` required if `--enable-tracing` is set
- Endpoint must be accessible (network connectivity)
- Use gRPC endpoint (not HTTP)

### Database Connection (AI)
- Cannot use `--without-db` with `--db-*` flags
- DB must be running before AI proxy (unless `--without-db`)
- DB port must be reachable from AI proxy

---

## Configuration Best Practices

### Production Deployment

```bash
# Database
ahnlich-db run \
  --host 0.0.0.0 \
  --port 1369 \
  --allocator-size 53687091200 \        # 50 GiB for large datasets
  --enable-persistence \
  --persist-location /mnt/data/db.dat \
  --persistence-interval 300000 \
  --maximum-clients 5000 \
  --threadpool-size 64 \                # Match CPU cores
  --enable-tracing \
  --otel-endpoint http://jaeger:4317 \
  --log-level "info,ahnlich_db=info"

# AI Proxy
ahnlich-ai run \
  --host 0.0.0.0 \
  --port 1370 \
  --db-host ahnlich-db \
  --db-port 1369 \
  --db-client-pool-size 50 \
  --supported-models all-minilm-l6-v2,bge-base-en-v1.5 \  # Only needed models
  --ai-model-idle-time 600 \
  --model-cache-location /mnt/models \
  --enable-streaming \                  # For image workloads
  --allocator-size 53687091200 \
  --enable-persistence \
  --persist-location /mnt/data/ai.dat \
  --maximum-clients 5000 \
  --enable-tracing \
  --otel-endpoint http://jaeger:4317
```

### Development

```bash
# Simple local setup
ahnlich-db run --log-level debug

ahnlich-ai run \
  --db-host 127.0.0.1 \
  --supported-models all-minilm-l6-v2 \  # Just one model for testing
  --log-level debug
```

---

## Docker Compose Configuration

See [Production Deployment](/ahnlich-in-production/deployment) for complete Docker Compose examples.

---

## See Also

- [Error Codes Reference](/reference/error-codes) - Understanding error messages
- [Troubleshooting](/troubleshooting/common-issues) - Common configuration issues
- [Production Deployment](/ahnlich-in-production/deployment) - Docker and cloud deployment
