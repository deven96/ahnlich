---
title: Persistence for Ahnlich AI
sidebar_position: 30
---

# Persistence for Ahnlich AI
### Why need persistence?
Ahnlich AI builds on DB by adding embeddings, vector similarity search, and model-driven queries. Without persistence:
- AI stores and embeddings vanish after restarts.

- Indices must be recomputed, delaying queries.

- Models re-download weights on each startup.

Persistence ensures that:
- Embeddings and indices survive container restarts.

- Models remain cached locally for faster startup.

- Production workloads avoid costly recomputation.

### Configuring persistence from in-memory mode to disk
Enable persistence for AI with the following configuration:
```
ahnlich_ai:
  image: ghcr.io/deven96/ahnlich-ai:latest
  command: >
    "ahnlich-ai run --db-host ahnlich_db --host 0.0.0.0 \
    --supported-models all-minilm-l6-v2,resnet-50 \
    --enable-persistence --persist-location /root/.ahnlich/data/ai.dat \
    --persistence-interval 300"
  ports:
    - "1370:1370"
  volumes:
    - "./ahnlich_ai_model_cache:/root/.ahnlich/models" # Model cache storage
    - "./data/:/root/.ahnlich/data" # Persistence Location
```

- `--enable-persistence` → activates persistence for embeddings and AI stores.

- `--persist-location` → file path for AI persistence (e.g., `ai.dat`).

- `--persistence-interval` → interval (in seconds) for saving snapshots.

- `./ahnlich_ai_model_cache` → stores model weights persistently.

- `./data/` → keeps AI persistence files safe.

### Loading from persistence
On restart, Ahnlich AI automatically restores state from `ai.dat`.
- All embeddings, similarity indices, and AI stores are reloaded.

- Cached models are loaded from `./ahnlich_ai_model_cache`, avoiding redownloads.

- Commands like `LISTSTORES`, `GETSIMN`, and `GETPRED` can confirm successful recovery.

- Recovery from backups is as easy as restoring the persistence file and model cache.
