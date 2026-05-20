# ahnlich-ai

Helm chart for `ahnlich-ai`, the embedding / AI proxy that sits in front of `ahnlich-db`.

Architecturally, `ahnlich-ai` accepts raw inputs (text / image / audio), turns them into embeddings via ONNX models, and forwards everything to a backing `ahnlich-db` cluster over gRPC. This chart deploys the AI process; it does **not** deploy `ahnlich-db` — point `db.host` at an existing DB.

The chart supports two deployment modes:

- **Standalone** (default): single replica with periodic JSON-snapshot persistence of AI-local store metadata.
- **Cluster (Raft)**: N replicas forming a Raft cluster for AI-local state. Activate with `--set cluster.enabled=true`.

## Prerequisites

- Kubernetes 1.20+ (for `appProtocol: grpc`).
- A default `StorageClass` (the chart falls back to it unless overridden).
- A reachable `ahnlich-db` deployment. Point `db.host` at its Service DNS name.
- Optional, for external L7 access: an Ingress Controller or Gateway API implementation.

## Quick start

```bash
# Standalone AI talking to an existing DB
helm install my-ai charts/ahnlich-ai \
  --set db.host=my-ahnlich-ahnlich-db

# With tracing
helm install my-ai charts/ahnlich-ai \
  --set db.host=my-ahnlich-ahnlich-db \
  --set tracing.enabled=true \
  --set tracing.otelEndpoint=http://jaeger-collector:4317

# Exposed externally
helm install my-ai charts/ahnlich-ai \
  --set db.host=my-ahnlich-ahnlich-db \
  --set service.type=LoadBalancer
```

## Values

### Image

| Key | Default | Description |
|---|---|---|
| `image.repository` | `ghcr.io/deven96/ahnlich-ai` | Image repository. |
| `image.tag` | `latest` | Image tag. |
| `image.pullPolicy` | `IfNotPresent` | Standard k8s pull policy. |

### Backing DB

| Key | Default | Description |
|---|---|---|
| `db.host` | `""` | **Required.** DNS name of the backing ahnlich-db Service. |
| `db.port` | `1369` | DB port. |
| `db.waitForReady.enabled` | `true` | Block AI startup with an initContainer until the DB Service port opens. Mirrors docker-compose `depends_on: service_healthy`. |
| `db.waitForReady.image` | `busybox:1.36` | Image used for the wait probe (just needs `nc`). |
| `db.waitForReady.timeoutSeconds` | `300` | How long to keep retrying before failing the pod. |
| `db.waitForReady.intervalSeconds` | `2` | Poll interval between port checks. |

The AI pod connects to the DB over gRPC. For a co-located DB install with default release name `my-ahnlich`, the value is typically `my-ahnlich-ahnlich-db` (same namespace) or `my-ahnlich-ahnlich-db.<db-namespace>.svc.cluster.local` (cross-namespace). Disable `db.waitForReady.enabled` if the DB lives somewhere k8s can't reach via a Service (external host, etc.) — though in that case the AI will still crash-restart until the connection succeeds, just without the initContainer's friendlier delay.

### Models

| Key | Default | Description |
|---|---|---|
| `models.supported` | `[all-minilm-l6-v2, resnet-50]` | List of model names; joined into `--supported-models`. |
| `models.cache.size` | `20Gi` | PVC size for the model cache. |
| `models.cache.storageClass` | `""` | StorageClass; empty = cluster default. |
| `models.cache.mountPath` | `/root/.ahnlich/models` | In-container model cache path. |

Each replica gets its own model-cache PVC. Models are downloaded on first use and reused across restarts.

### Service

| Key | Default | Description |
|---|---|---|
| `service.type` | `ClusterIP` | One of `ClusterIP`, `LoadBalancer`, `NodePort`. |
| `service.port` | `1370` | Public client-facing gRPC port. |

`appProtocol: grpc` is set on the port for Ingress/Gateway controller auto-discovery.

### Standalone persistence

| Key | Default | Description |
|---|---|---|
| `persistence.enabled` | `true` | Mount a PVC at `mountPath` and pass `--enable-persistence`. |
| `persistence.size` | `5Gi` | AI-local metadata is small; smaller default than DB. |
| `persistence.storageClass` | `""` | StorageClass; empty = cluster default. |
| `persistence.mountPath` | `/root/.ahnlich/data` | In-container mount point. |
| `persistence.fileName` | `ai.dat` | Snapshot file name. |
| `persistence.intervalMs` | `30000` | `--persistence-interval` value in ms. |

Automatically suppressed when `cluster.enabled=true` (Raft snapshots and log replication replace standalone persistence).

### Cluster (Raft) mode

| Key | Default | Description |
|---|---|---|
| `cluster.enabled` | `false` | Run N replicas as an AI-local Raft cluster. |
| `cluster.replicas` | `3` | StatefulSet replica count. |
| `cluster.port` | `1371` | `--cluster-addr` port for Raft RPCs. |
| `cluster.storage` | `rocksdb` | `rocksdb` (production) or `memory` (testing only). |
| `cluster.dataDir` | `/root/.ahnlich/raft` | `--cluster-data-dir`. |
| `cluster.snapshotLogs` | `1000` | Count-based snapshot trigger. |
| `cluster.snapshotIntervalMs` | `300000` | Time-based snapshot trigger. |
| `cluster.persistence.size` | `10Gi` | PVC size for the Raft data dir. |
| `cluster.persistence.storageClass` | `""` | StorageClass; empty = cluster default. |
| `cluster.podDisruptionBudget.enabled` | `true` | Protect quorum during voluntary disruptions. |
| `cluster.podDisruptionBudget.minAvailable` | `2` | Minimum pods that must stay up. |

The AI cluster only replicates store metadata (CreateStore, DropStore, PurgeStores). All other mutations are proxied to the DB cluster via `db.host` and replicated by the DB's own Raft cluster.

### Tracing

| Key | Default | Description |
|---|---|---|
| `tracing.enabled` | `false` | Pass `--enable-tracing --otel-endpoint=...`. |
| `tracing.otelEndpoint` | `""` | OTLP gRPC endpoint. |

The chart does not deploy a tracing backend.

### Ingress and Gateway

Identical surface to `ahnlich-db`:

- `ingress.*` produces an `Ingress` resource (controller required in-cluster).
- `gateway.*` produces a `GRPCRoute` attached to an existing Gateway.
- Both enabled simultaneously is a misconfiguration; the chart fails the install.

See the `ahnlich-db` README for the full key reference.

### Logging

| Key | Default | Description |
|---|---|---|
| `logLevel` | `""` | Sets `--log-level` on the binary. Empty = the binary's default (`info,hf_hub=warn`). Example values: `debug`, `trace`, `info,ahnlich_ai_proxy=debug`. |

`ahnlich-ai` reads its log filter from `--log-level`, **not** from `RUST_LOG`. Setting `RUST_LOG` via `env` has no effect.

### Environment variables

| Key | Default | Description |
|---|---|---|
| `env` | `[]` | List of `EnvVar` entries (`name/value` or `valueFrom`). |
| `envFrom` | `[]` | List of `EnvFromSource` entries (`configMapRef` or `secretRef`). |

For process env not exposed as a flag (`RUST_BACKTRACE=1`, `ORT_DYLIB_PATH` for a custom ONNX Runtime build, etc.). Both keys take the standard k8s shape. For log verbosity use `logLevel` above instead.

### Pod scheduling and resources

| Key | Default | Description |
|---|---|---|
| `resources` | `{}` | Container `resources` block. AI is heavier than DB; set requests/limits per workload. |
| `nodeSelector` | `{}` | Schedule on nodes matching these labels. |
| `tolerations` | `[]` | Tolerate node taints. |
| `affinity` | `{}` | Pod / node affinity. |
| `podAnnotations` | `{}` | Free-form pod annotations. |

GPU acceleration: add `nvidia.com/gpu` to `resources.limits` and a matching `nodeSelector` (typically `nvidia.com/gpu.present: "true"` or a vendor-specific label). The image ships with the CUDA ONNX Runtime.

### Probes

`probes.readiness.*` and `probes.liveness.*` accept the standard k8s timing fields. Both probes run `ahnlich-cli PING` (with `--agent ai`) against the local pod. Liveness `initialDelaySeconds` defaults to 60s to cover first-time model download on container start.

## External access patterns

Same three options as `ahnlich-db`:

1. `service.type=LoadBalancer` — L4, one external IP per Service.
2. `ingress.enabled=true` — shared edge router, TLS, hostname routing.
3. `gateway.enabled=true` — Gateway API, gRPC-aware via `GRPCRoute` with optional per-method matching.

For in-cluster clients (e.g., your own app calling the AI), leave `service.type=ClusterIP` and reach the AI at `<release>-ahnlich-ai.<namespace>.svc.cluster.local:<port>`.

## Upgrading

`helm upgrade my-ai charts/ahnlich-ai` rolls pods one at a time. PVCs (models cache and persistence data) survive the upgrade.

In cluster mode, the PodDisruptionBudget at `minAvailable: 2` keeps quorum intact through rolling upgrades.

## Uninstalling

```bash
helm uninstall my-ai
kubectl delete pvc -l app.kubernetes.io/instance=my-ai      # only if you also want to drop the cached models / data
```
