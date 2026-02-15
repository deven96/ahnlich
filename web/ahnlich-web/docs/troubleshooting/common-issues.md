---
title: Common Issues
sidebar_position: 10
---

# Troubleshooting Common Issues

This guide covers the most common issues users encounter and how to resolve them.

## Memory and Performance Issues

### Out of Memory Errors

**Symptoms:**
```
allocation error: CapacityOverflow
Server crashes unexpectedly
```

**Causes:**
- Hitting the `--allocator-size` limit
- Large batch operations
- Image processing without streaming enabled

**Solutions:**

1. **Increase allocator size:**
```bash
ahnlich-db run --allocator-size 21474836480  # 20 GiB (default is 10 GiB)
ahnlich-ai run --allocator-size 21474836480
```

2. **Enable streaming for images (AI proxy):**
```bash
ahnlich-ai run --enable-streaming  # 10x less memory, 40% slower
```

3. **Reduce batch sizes:**
```python
# Instead of:
large_batch = [entry1, entry2, ..., entry1000]
client.set(Set(store="my_store", inputs=large_batch))

# Do this:
batch_size = 100
for i in range(0, len(large_batch), batch_size):
    batch = large_batch[i:i+batch_size]
    client.set(Set(store="my_store", inputs=batch))
```

4. **Monitor memory usage:**
```bash
# Check process memory
ps aux | grep ahnlich

# Monitor with top
top -p $(pgrep ahnlich)
```

---

### Slow Query Performance

**Symptoms:**
- Queries taking longer than expected
- High CPU usage

**Diagnostic Steps:**

1. **Enable tracing to identify bottlenecks:**
```bash
ahnlich-db run --enable-tracing --otel-endpoint http://localhost:4317
```
View traces in Jaeger UI at `http://localhost:16686`

2. **Check store size:**
```
INFOSERVER
```

3. **Verify algorithm choice:**
- Linear algorithms (Cosine, Euclidean, DotProduct) scale linearly with data size
- Use `KDTree` for faster searches with large datasets:
```
CREATESTORE my_store DIMENSION 128 NONLINEARALGORITHMINDEX (KDTree)
```

**Solutions:**

1. **Use predicate indices for filtering:**
```
# Index frequently filtered fields
CREATEPREDINDEX my_store PREDICATES (category, author)

# Then filter efficiently
GETPRED 10 IN my_store WHERE (category = science)
```

2. **Optimize batch operations:**
```python
# Batch SET operations
entries = [entry1, entry2, ..., entry100]
client.set(Set(store="my_store", inputs=entries))
```

3. **Use appropriate similarity algorithm:**
- **CosineSimilarity**: Best for normalized vectors, direction-based similarity
- **EuclideanDistance**: Best for absolute distance measures
- **DotProduct**: Fast when vectors are pre-normalized
- **KDTree**: Best for high-dimensional spatial searches

4. **Adjust thread pool size:**
```bash
ahnlich-db run --threadpool-size 32  # Default: 16
```

---

## Connection Issues

### Cannot Connect to Server

**Symptoms:**
```
connection refused
Failed to dial server
Transport issues with tonic
```

**Diagnostic Steps:**

1. **Check if server is running:**
```bash
# Check DB
curl http://localhost:1369  # or use telnet
ps aux | grep ahnlich-db

# Check AI
curl http://localhost:1370
ps aux | grep ahnlich-ai
```

2. **Verify port availability:**
```bash
# Check if port is in use
lsof -i :1369
lsof -i :1370

# Or with netstat
netstat -tuln | grep 1369
```

3. **Check firewall rules:**
```bash
# Ubuntu/Debian
sudo ufw status
sudo ufw allow 1369
sudo ufw allow 1370

# CentOS/RHEL
sudo firewall-cmd --list-all
sudo firewall-cmd --add-port=1369/tcp --permanent
sudo firewall-cmd --reload
```

**Solutions:**

1. **Start server on all interfaces:**
```bash
# Allow connections from any IP
ahnlich-db run --host 0.0.0.0 --port 1369
ahnlich-ai run --host 0.0.0.0 --port 1370
```

2. **Check host/port configuration:**
```python
# Correct
client = DbClient("http://127.0.0.1:1369")

# Wrong - missing protocol
client = DbClient("127.0.0.1:1369")  # Invalid URI error
```

3. **Verify network connectivity:**
```bash
# Test connectivity
ping <server-host>
telnet <server-host> 1369
```

---

### Maximum Clients Reached

**Symptoms:**
```
Max Connected Clients Reached
Connection rejected
```

**Cause:** Hit the `--maximum-clients` limit (default: 1000)

**Solutions:**

1. **Increase client limit:**
```bash
ahnlich-db run --maximum-clients 5000
```

2. **Check current connections:**
```
LISTCLIENTS
```

3. **Implement connection pooling:**
```python
# Reuse connections instead of creating new ones
class ClientPool:
    def __init__(self, uri, pool_size=10):
        self.pool = [DbClient(uri) for _ in range(pool_size)]
        self.index = 0
    
    def get_client(self):
        client = self.pool[self.index]
        self.index = (self.index + 1) % len(self.pool)
        return client
```

4. **Close idle connections:**
```python
async def cleanup():
    await client.close()
```

---

### AI Proxy Cannot Connect to Database

**Symptoms:**
```
Proxy Errored with connection refused
DatabaseClientError
```

**Diagnostic Steps:**

1. **Verify DB is running:**
```bash
ps aux | grep ahnlich-db
```

2. **Check DB host/port:**
```bash
# See what DB is listening on
netstat -tuln | grep 1369
```

**Solutions:**

1. **Start DB before AI:**
```bash
# Terminal 1
ahnlich-db run --port 1369

# Terminal 2 (wait for DB to start)
ahnlich-ai run --db-host 127.0.0.1 --db-port 1369
```

2. **Verify connection settings:**
```bash
# If DB is on different host
ahnlich-ai run --db-host 192.168.1.10 --db-port 1369

# If DB uses non-default port
ahnlich-ai run --db-port 1400
```

3. **For standalone mode (no DB):**
```bash
ahnlich-ai run --without-db
```

4. **Adjust connection pool:**
```bash
ahnlich-ai run --db-client-pool-size 20  # Default: 10
```

---

## Data and Store Issues

### Store Not Found

**Symptoms:**
```
Store "my_store" not found
```

**Diagnostic Steps:**

1. **List all stores:**
```
LISTSTORES
```

2. **Check store name spelling:**
```
# Store names are case-sensitive
"MyStore" ≠ "mystore"
```

**Solutions:**

1. **Create the store:**
```
# DB
CREATESTORE my_store DIMENSION 128

# AI
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
```

2. **Check persistence loaded:**
```bash
# If using persistence
ahnlich-db run \
  --enable-persistence \
  --persist-location /path/to/data.dat \
  --fail-on-startup-if-persist-load-fails true  # Fail loudly if load fails
```

3. **Verify correct server:**
```python
# Make sure you're connecting to the right instance
client = DbClient("http://localhost:1369")  # Not a different instance
```

---

### Dimension Mismatch Errors

**Symptoms:**
```
Store dimension is [128], input dimension of [256] was specified
```

**Cause:** Vector dimensions don't match store configuration.

**Solutions:**

1. **Check store dimension:**
```
INFOSERVER
# Look at store details
```

2. **For AI stores, verify model dimensions:**

| Model | Embedding Dimension |
|-------|---------------------|
| all-minilm-l6-v2 | 384 |
| all-minilm-l12-v2 | 384 |
| bge-base-en-v1.5 | 768 |
| bge-large-en-v1.5 | 1024 |
| resnet-50 | 2048 |
| clip-vit-b32-* | 512 |

3. **Match query and index models:**
```python
# Both must have same dimensions
CreateStore(
    store="my_store",
    query_model=AiModel.BGE_BASE_EN_V15,  # 768-dim
    index_model=AiModel.BGE_BASE_EN_V15,  # 768-dim (same)
)
```

4. **Recreate store with correct dimension:**
```
DROPSTORE my_store IFTRUE
CREATESTORE my_store DIMENSION 768
```

---

### Predicate Not Found

**Symptoms:**
```
Predicate "author" not found in store
```

**Cause:** Querying by a predicate that wasn't indexed.

**Solutions:**

1. **Create predicate index:**
```
CREATEPREDINDEX my_store PREDICATES (author, category)
```

2. **Or include when creating store:**
```
CREATESTORE my_store DIMENSION 128 PREDICATES (author, category)
```

3. **Verify predicates exist:**
```
INFOSERVER
# Check store predicates
```

---

## Model and AI Issues

### Model Not Loading

**Symptoms:**
```
index_model or query_model not selected or loaded
Error initializing a model thread
Tokenizer for model failed to load
```

**Diagnostic Steps:**

1. **Check supported models:**
```bash
ahnlich-ai run --supported-models all-minilm-l6-v2,resnet-50
```

2. **Verify model cache:**
```bash
# Default location
ls -la ~/.ahnlich/models

# Custom location
ahnlich-ai run --model-cache-location /path/to/models
```

3. **Check disk space:**
```bash
df -h ~/.ahnlich/models
```

4. **Test network connectivity:**
```bash
# Models download from HuggingFace
curl https://huggingface.co
```

**Solutions:**

1. **Wait for initial download:**
```bash
# First time loading a model downloads from HuggingFace
# This can take several minutes depending on model size
# Watch logs for progress
```

2. **Clear corrupted cache:**
```bash
rm -rf ~/.ahnlich/models/model_name
# Restart server to re-download
```

3. **Increase idle time:**
```bash
# Keep models loaded longer
ahnlich-ai run --ai-model-idle-time 600  # 10 minutes (default: 5 min)
```

4. **Pre-download models:**
```bash
# Download models before starting server
python -c "from transformers import AutoModel; AutoModel.from_pretrained('sentence-transformers/all-MiniLM-L6-v2')"
```

---

### Token Limit Exceeded

**Symptoms:**
```
Max Token Exceeded. Model Expects [256], input type was [512]
```

**Cause:** Text input exceeds model's token limit.

**Token Limits:**
- all-minilm-*: 256 tokens
- bge-*: 512 tokens
- clip-vit-b32-text: 77 tokens

**Solutions:**

1. **Truncate text:**
```python
def truncate_text(text, max_length=200):
    words = text.split()
    return ' '.join(words[:max_length])

text = truncate_text(long_text)
```

2. **Split into chunks:**
```python
def chunk_text(text, chunk_size=200):
    words = text.split()
    return [' '.join(words[i:i+chunk_size]) 
            for i in range(0, len(words), chunk_size)]

chunks = chunk_text(long_document)
for chunk in chunks:
    client.set(Set(store="docs", inputs=[...]))
```

3. **Use model with larger limit:**
```python
# Switch from AllMiniLM (256) to BGE (512)
CreateStore(
    store="my_store",
    query_model=AiModel.BGE_BASE_EN_V15,  # 512 tokens
    index_model=AiModel.BGE_BASE_EN_V15,
)
```

---

### Image Dimension Errors

**Symptoms:**
```
Image Dimensions [(512, 512)] does not match expected [(224, 224)]
Image can't have zero dimension
```

**Cause:** Images not matching model requirements (224x224 pixels).

**Solutions:**

1. **Resize images:**
```python
from PIL import Image

def prepare_image(image_path):
    img = Image.open(image_path)
    img = img.resize((224, 224))
    return img.tobytes()

image_bytes = prepare_image("photo.jpg")
```

2. **Use model preprocessing:**
```python
Set(
    store="my_store",
    inputs=[...],
    preprocess_action=PreprocessAction.ModelPreprocessing,  # Auto-resize
)
```

3. **Validate images before sending:**
```python
def validate_image(image_bytes):
    img = Image.open(io.BytesIO(image_bytes))
    if img.width == 0 or img.height == 0:
        raise ValueError("Invalid image dimensions")
    return img

img = validate_image(image_bytes)
```

---

## Persistence Issues

### Persistence File Won't Load

**Symptoms:**
```
Failed to load persistence file
Corruption detected
```

**Diagnostic Steps:**

1. **Check file permissions:**
```bash
ls -l /path/to/persistence.dat
```

2. **Verify file size vs allocator:**
```bash
# File size
du -h persistence.dat

# Allocator must be >2x file size
```

**Solutions:**

1. **Increase allocator size:**
```bash
# If persistence file is 5 GB, use at least 10 GB allocator
ahnlich-db run \
  --enable-persistence \
  --persist-location /path/to/data.dat \
  --allocator-size 10737418240  # 10 GB
```

2. **Skip corrupted persistence:**
```bash
ahnlich-db run \
  --enable-persistence \
  --persist-location /path/to/data.dat \
  --fail-on-startup-if-persist-load-fails false  # Continue without persistence
```

3. **Backup and delete:**
```bash
# Backup
cp persistence.dat persistence.dat.backup

# Start fresh
rm persistence.dat
ahnlich-db run --enable-persistence --persist-location persistence.dat
```

4. **Check disk space:**
```bash
df -h /path/to/persistence/
```

---

### Data Lost After Restart

**Cause:** Persistence not enabled.

**Solution:**

Enable persistence when starting server:
```bash
ahnlich-db run \
  --enable-persistence \
  --persist-location /var/lib/ahnlich/db.dat \
  --persistence-interval 300000  # 5 minutes
```

---

## Debugging Tips

### Enable Detailed Logging

```bash
# Set log level
ahnlich-db run --log-level debug

# Or specific modules
ahnlich-db run --log-level "info,ahnlich_db=debug,hf_hub=warn"
```

### Enable Distributed Tracing

```bash
# Start Jaeger
docker run -d \
  -p 16686:16686 \
  -p 4317:4317 \
  jaegertracing/all-in-one:latest

# Start server with tracing
ahnlich-db run \
  --enable-tracing \
  --otel-endpoint http://localhost:4317

# View traces at http://localhost:16686
```

### Use CLI for Testing

```bash
# Interactive mode
ahnlich --agent DB --host 127.0.0.1 --port 1369

# Test commands
PING
INFOSERVER
LISTSTORES
```

### Check Server Health

```bash
# Process status
ps aux | grep ahnlich

# Resource usage
top -p $(pgrep ahnlich)

# Network connections
netstat -anp | grep ahnlich

# Open files
lsof -p $(pgrep ahnlich)
```

---

## Getting More Help

Still having issues? Try these resources:

1. **Check Error Codes**: [Error Codes Reference](/reference/error-codes)
2. **Read Configuration Docs**: [Configuration Reference](/reference/configuration)
3. **Enable Tracing**: See detailed request flow
4. **Community**: [WhatsApp Group](https://chat.whatsapp.com/E4CP7VZ1lNH9dJUxpsZVvD)
5. **GitHub**: [Report Issues](https://github.com/deven96/ahnlich/issues)

When reporting issues, include:
- Error messages (full text)
- Server version
- Configuration flags used
- Steps to reproduce
- Server logs (with `--log-level debug`)
