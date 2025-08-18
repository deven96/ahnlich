---
title: Architecture
sidebar_position: 60
---

<!-- # Architecture V1

Here’s a deeply detailed architecture overview of Ahnlich, synthesizing the repo’s structure, the Mermaid diagram, and how the components interact in a production scenario:


## 📦 1. System Overview

Ahnlich comprises **two separate services**, each focusing on distinct responsibilities:

1. ahnlich-db – the **Vector Store Layer**
2. ahnlich-ai – the **Intelligence / Embedding Layer**

These services communicate through network calls, forming a modular and scalable system. Clients may use direct access to either service (via gRPC), or utilize CLI/SDK wrappers.

## 2. Core Services & Components

```ahnlich-db``` (Vector Store Layer)

**DB Client API**: Handles low-level operations like SET, GETSIMN, CREATESTORE; exposed via gRPC.

**StoreHandlerDB**: Dispatcher that routes requests to the correct per-store context.

**Store (StoreA_DB)**:

Maintains **in-memory vector index**: brute-force or KD-tree variants.
Keeps a **metadata map** (key-value for each vector).
Executes similarity searches (cosine/euclidean) and supports metadata filtering queries (GitHub).

**PersistenceDB** *(optional)*: Upon --enable-persistence, it serializes vectors and metadata to disk at configured intervals.


```ahnlich-ai``` (Embedding & AI Proxy Layer)
**AI Client API**: Accepts RAW inputs (text, image, metadata) and high-level commands (e.g. CREATESTORE, GETSIMN).

**StoreHandlerAI**: Per-store manager that handles:

Which models to use.
Preprocessing pipelines.

**Store (StoreA_AI)**:

Picks between **Index Model** (for embedding storage) and **Query Model** (for search input).
Preprocessing (e.g. tokenization, normalization).

**Model Node**:

Runs preprocessing -> model inference (ONNX models).
Index and query function models (GitHub).

**PersistenceAI** (optional): Stores model cache, pipeline metadata, and optional state across restarts.

## 3.  Data Flow Techniques

### A. Indexing Path (Write)

1. Client sends raw data + metadata to **AI layer**.

2. **StoreHandlerAI**:
a. Preprocesses text/image.
b. Runs Index Model via ONNX to generate embedding.

1. Preprocesses text/image.
1. Runs **Index Model** via ONNX to generate embedding.

3. Sends SET(store, vector, metadata) to **DB layer**.

4. **DB layer**:

a. Inserts vector into index.
b. Stores metadata map.
c. Optionally persists snapshot to disk.

### B. Similarity Query Path
1. Client queries via AI layer (e.g. input text/image).

2. **AI layer**:

+ Runs **Query Model** to generate embedding.

3. Invokes GETSIMN on DB:

a. Includes algorithm choice (cosine/euclidean/KD-tree).
b. Optional metadata predicates.

4. **DB layer**:

a. Computes similarity.
b. Applies metadata filters.
c. Returns Top-N results + scores.

5. **AI layer**:

a. Optionally repackages response, adds metadata.
C. Direct DB Access
For advanced users: precomputed embeddings and metadata bypass AI layer entirely.

## 4. Inter-Service Communication
1. **AI ➜ DB**:

a. SET: vector + metadata requests.
b. GETSIMN: search_vector + filters.

2. **DB ➜ AI** responses via Top-N results.

Bindings typically over internal network (gRPC), enabling flexibility and decoupled deployment.

## 5. Persistence Strategy
1. **Opt-in persistence**: Activated by flag --enable-persistence and configured interval (default 300s).
2. **DB**: Binary snapshot of vectors and metadata.
3. **AI**: Cached models and metadata state (e.g. supported models per store).
4. **Recovery**: On startup, each service reloads its data from disk before accepting traffic.
5. No replication or distributed storage yet—designed for single-node or sharded deployments (GitHub, Hacker News, GitHub, GitHub).

## 6. Scaling & Deployment Patterns
| Architecture | Description | Ideal Use Case | 
| ----- | ----- | ----- |
| **Single‑Node** | 1×AI + 1×DB container (e.g. Docker Compose in README) (GitHub) | Local dev, testing |
| **Vertical Scaling** | Add GPU to AI; allocate more RAM/CPU to DB | Medium-scale workloads | 
| **Store‑Level Sharding** | Multiple DB instances handle different stores, fronted by single AI | Multi-tenant, large corpora | 
| **Model‑Specific AI Instances** | AI instances per model type (e.g. text vs image) | Heterogeneous pipelines |
| **Future Roadmap** | Distributed consensus, replication, transparent sharding (via consistent hashing) (GitHub) | High availability & scale |

## 7. Observability & Monitoring
Both services include **OpenTelemetry tracing** → send to backend like Jaeger or Prometheus.

Enable via --enable-tracing and --otel-endpoint.

Key metrics:

Request latency, vector index size, memory usage.
Model inference time, throughput.


## 8. Extension Points
**Add new similarity metrics**: Implement SimAlgorithm trait in ahnlich-db (e.g. Jaccard or Manhattan).
**Add custom ONNX models**: Supply your own with --supported-models.
**Extend metadata predicates**: Add regex/full-text support.
**Upgrade store**: Swap vector index (e.g. Annoy or HNSW) by plugging into store handler.


## 9. Security Considerations
Currently, no built-in authentication.

Recommended setup:

- Place services behind an API gateway.
- Enforce JWT/OAuth token on AI client side.
- Use mTLS between AI and DB in multi-host setups.

## 10. Limitations (as of July 2025)
**No built-in replication**: Single instance durability only.
**Concurrency**: Single-writer lock per store – may be a bottleneck.
**Model hot‑swap**: Requires recreating the store to swap Query/Index models.

## 🔍 Summary
Ahnlich cleanly separates vector intelligence (AI embedding + model logic) from vector persistence and retrieval. This lets teams scale and optimize each layer independently, while retaining flexibility and ease of use—very much like Kafka’s producer/broker/consumer paradigm, but applied to embeddings and similarity search (GitHub, GitHub).
 -->

<!-- [Entity -xxx-num-ref-](entity-page#entity-id) -->
<!-- - Macro Chapter X
    1. Chapter A
        - Section 1.1
            ... subsections
        - Section 1.2
    2. Chapter B
        - Section 2.1
            ... subsections
        - Section 2.2
    - Macro Chapter Y
    etc. -->




# Ahnlich Architecture V2
**Status**: *Alpha / testing – subject to breaking changes.***

Ahnlich is split into two independent, network‑accessible services that work in tandem:

- ahnlich‑ai – **the Intelligence Layer**
- ahnlich‑db – **the Vector Store Layer**

Clients can speak to either layer through gRPC/HTTP or the bundled CLI/SDKs. The AI layer adds automated embedding and model management on top of the raw vector store exposed by the DB layer.

## 1.  High‑Level Design
flowchart TD

  subgraph ai [ahnlich‑ai]

    direction TB

    AIClient["AI Client"]

    StoreHandlerAI["Store Handler"]

    StoreA_AI["Store A"]

    ModelNode["Index Model → Model B<br/>Query Model → Model A<br/>Pre‑process"]

    PersistenceAI[(Persistence)]

    AIClient --> |"original + metadata"| StoreHandlerAI

    StoreHandlerAI --> StoreA_AI

    StoreA_AI --> ModelNode

    ModelNode -.-> PersistenceAI

  end


  subgraph db [ahnlich‑db]

    direction TB

    DBClient["DB Client"]

    StoreHandlerDB["Store Handler"]

    StoreA_DB["Store A"]

    PersistenceDB[(Persistence)]

    DBClient --> |"DB query"| StoreHandlerDB

    StoreHandlerDB --> StoreA_DB

    StoreA_DB -.-> PersistenceDB

  end

  %% Inter‑service calls

  StoreHandlerAI -.-> |"Set: vector + metadata"| StoreHandlerDB

  StoreHandlerAI -.-> |"GetSimN: vector"| StoreHandlerDB

  StoreHandlerDB -.-> |"Top‑N results"| StoreHandlerAI

### Analogy to Kafka
| Kafka | Ahnlich | 
| ----- | ----- |
| **Producer** | AI Client / DB Client | 
| **Broker** | ahnlich‑ai & ahnlich‑db services | 
| **Topic / Partition** | Store (logical namespace) |
| **Message** | Vector + metadata |
| **Consumer** | Client fetching GetSimN |


## 2. Key Components
### 2.1  `ahnlich‑ai` – Intelligence Layer
| Sub‑component | Responsibility | 
| ----- | ----- |
| **AI Client API** | External gRPC/HTTP endpoints – accepts raw documents (text, images…) & metadata. |
| **Store Handler** | Maps incoming requests to a Store; maintains per‑store configuration (models, preprocess pipeline). | 
| **Store** | Logical namespace. Each holds a pair of ONNX models (Index & Query) plus preprocessing logic. | 
| **Model Node** | Executes preprocessing → model inference → produces embedding. |
| **Optional Persistence** | Periodic snapshots of store metadata & model cache to disk. |


### 2.2 `ahnlich‑db` – Vector Store Layer
| Sub‑component | Responsibility | 
| ----- | ----- |
| **DB Client API** | Accepts vector‑level commands: SET, GETSIMN, CREATESTORE, etc. |
| **Store Handler** | Routes to correct Store; enforces isolation; coordinates concurrent reads/writes. | 
| **Store (Vector Index)** | In‑memory index (brute‑force or KD‑Tree) plus metadata map. Supports cosine & Euclidean similarity. | 
| **Filter Engine** | Applies boolean predicates on metadata during query. |
| **Optional Persistence** | Snapshots vectors & metadata to on‑disk binary file for warm restarts. |


## 3.  Data Flow
### 3.1  Indexing (Write) Path
1. **Client** ➜ AI Layer – Sends raw document + metadata.
2. **Preprocessing & Embedding** – AI layer cleans input, runs Index Model to yield vector.
3. **AI ➜ DB** – Issues SET carrying vector & metadata.
4. **DB Store** – Writes vector into index, stores metadata.

### 3.2  Similarity Query Path
1. **Client ➜ AI Layer** – Provides search text/image.
2. **Embedding** – AI layer runs Query Model to create search vector.
3. **AI ➜ DB (GETSIMN)** – Vector + algorithm + optional predicate.
4. **DB** – Computes distance, applies metadata filter, returns Top‑N IDs & scores.
5. **AI Layer** – (Optional) post‑processes or joins additional metadata before responding to client.

### 3.3  Direct DB Access
Advanced users can bypass AI and push pre‑computed vectors directly into ahnlich‑db for maximum control.


## 4  Persistence & Durability
- **Opt‑in via** --enable-persistence.
- **Snapshot interval** configurable (--persistence-interval, default 300 s).
- **DB** writes a flat binary file; **AI** persists model cache & store manifests.
- On startup each service checks for the snapshot file and hydrates memory before accepting traffic.
- No replication yet; Ahnlich currently targets single‑node or shared‑nothing sharded deployments.

## 5. Scaling & Deployment Topologies
| Pattern | How it works | When to use | 
| ----- | ----- | ----- |
| **Single‑Node** | One `ahnlich‑ai` & one `ahnlich‑db` container (shown in README Compose). | Prototyping, local dev. |
| **Vertical Scaling** | Give DB more RAM/CPU; mount NVIDIA GPU for AI layer. | Medium workloads where a single node still fits in memory. | 
| **Store‑Level Sharding** | Run multiple DB instances, each owning a subset of Stores; fronted by one AI layer. | Multi‑tenant SaaS or very large corpora. | 
| **Function Sharding** | Isolate heavy NLP image pipelines by model type: one AI instance per model group. | Heterogeneous workloads, GPU scheduling. |

**Roadmap**: cluster‑wide replication & consistent hashing for transparent sharding.


## 6.  Observability
- Both services instrumented with **OpenTelemetry**; enable with --enable-tracing and send spans to Jaeger, Prometheus, etc.
- Internal metrics: query latency, index size, RAM usage, model inference time.


## 7.  Extensibility
- **Add a new similarity metric** – implement SimAlgorithm trait in ahnlich‑db.
- **Bring your own model** – point ahnlich‑ai to an ONNX file or HuggingFace repo via --supported-models.
- **Custom predicates** – extend the predicate DSL to support regex or full‑text.


## 8.  Security Considerations
Currently no built‑in auth. Recommend placing behind an API gateway or reverse proxy that enforces:

- JWT / OAuth 2 bearer tokens.
- Mutual TLS between AI ⇄ DB if running across hosts.


## 9.  Limitations (July 2025)
- No distributed consensus – durability limited to local snapshots.
- Single‑writer per Store lock may become a bottleneck under heavy concurrent writes.
- Model hot‑swap requires store recreation.


## Summary
*Ahnlich decouples vector intelligence* (embedding generation, model lifecycle) from vector persistence & retrieval. This split allows you to scale and tune each layer independently while keeping a simple mental model—much like Kafka separates producers, brokers, and consumers around an immutable log.
