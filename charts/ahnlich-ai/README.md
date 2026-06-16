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

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| affinity | object | `{}` | Affinity rules for pod scheduling. |
| cluster.dataDir | string | `"/root/.ahnlich/raft"` | `--cluster-data-dir` (RocksDB log + snapshots). |
| cluster.enabled | bool | `false` | Run N replicas as an AI-local Raft cluster instead of a single standalone node. |
| cluster.persistence.size | string | `"10Gi"` | PVC size for the Raft data dir (AI replicates only metadata, so logs grow slower than DB's). |
| cluster.persistence.storageClass | string | `""` | StorageClass for the Raft data dir. Empty uses the cluster default. |
| cluster.podDisruptionBudget.enabled | bool | `true` | Create a PodDisruptionBudget in cluster mode. |
| cluster.podDisruptionBudget.maxUnavailable | int | `1` | Pods that may be evicted at once; scales with replicas. |
| cluster.port | int | `1371` | `--cluster-addr` port for Raft RPCs. |
| cluster.replicas | int | `3` | StatefulSet replica count in cluster mode. |
| cluster.snapshotIntervalMs | int | `300000` | `--cluster-snapshot-interval` time-based snapshot trigger (ms). |
| cluster.snapshotLogs | int | `1000` | `--cluster-snapshot-logs` count-based snapshot trigger. |
| cluster.storage | string | `"rocksdb"` | Raft storage backend: `rocksdb` (production) or `memory` (testing only, no durability). |
| db.host | string | `""` | DNS name of the backing ahnlich-db Service (required), e.g. `my-ahnlich-ahnlich-db`. |
| db.port | int | `1369` | DB port. |
| db.waitForReady.enabled | bool | `true` | Block AI startup with an initContainer until the DB Service port opens (mirrors docker-compose `depends_on: service_healthy`). |
| db.waitForReady.image | string | `"busybox:1.36.1"` | Image used for the wait probe (needs `nc`). |
| db.waitForReady.intervalSeconds | int | `2` | Poll interval between port checks (s). |
| db.waitForReady.timeoutSeconds | int | `300` | How long to keep retrying before failing the pod (s). |
| env | list | `[]` | Extra environment variables (name/value or valueFrom). Use for `RUST_BACKTRACE`, `ORT_DYLIB_PATH`, etc. |
| envFrom | list | `[]` | Bulk-import env from ConfigMaps or Secrets. |
| gateway.enabled | bool | `false` | Create a GRPCRoute attached to an existing Gateway (mutually exclusive with `ingress`). |
| gateway.hostnames | list | `[]` | Hostnames the route claims, e.g. `["ai.example.com"]`. |
| gateway.parentRefs | list | `[{"name":"","namespace":"","sectionName":""}]` | Existing Gateway references; `name` is required when `gateway.enabled`. |
| image.pullPolicy | string | `""` | Pull policy. Empty selects `Always` for the `latest` tag, `IfNotPresent` otherwise. |
| image.repository | string | `"ghcr.io/deven96/ahnlich-ai"` | Image repository. |
| image.tag | string | `"latest"` | Image tag. |
| ingress.annotations | object | `{}` | Controller-specific annotations (e.g. nginx-ingress `backend-protocol: GRPC`). |
| ingress.className | string | `""` | IngressClass name. Empty uses the cluster default. |
| ingress.enabled | bool | `false` | Create an Ingress resource (requires an Ingress Controller in the cluster). |
| ingress.host | string | `""` | Hostname, e.g. `ai.example.com`. Required when `ingress.tls.enabled`. |
| ingress.path | string | `"/"` | Path prefix. |
| ingress.pathType | string | `"Prefix"` | Path type: `Prefix`, `Exact`, or `ImplementationSpecific`. |
| ingress.tls.enabled | bool | `false` | Add a TLS block to the Ingress. |
| ingress.tls.secretName | string | `""` | Name of an existing TLS Secret in this namespace. |
| logLevel | string | `""` | `--log-level` (env_logger / tracing-subscriber syntax). Empty uses the binary default (`info,hf_hub=warn`). |
| models.cache.mountPath | string | `"/root/.ahnlich/models"` | In-container model cache path. |
| models.cache.size | string | `"20Gi"` | PVC size for the model cache. |
| models.cache.storageClass | string | `""` | StorageClass for the model cache. Empty uses the cluster default. |
| models.supported | list | `["all-minilm-l6-v2","resnet-50"]` | Model names served by this instance; comma-joined into `--supported-models`. |
| nodeSelector | object | `{}` | Node selector for pod scheduling. |
| persistence.enabled | bool | `true` | Mount a PVC and pass `--enable-persistence` (standalone mode only). |
| persistence.fileName | string | `"ai.dat"` | Snapshot file name. |
| persistence.intervalMs | int | `30000` | `--persistence-interval` value (ms). |
| persistence.mountPath | string | `"/root/.ahnlich/data"` | In-container mount point for the snapshot. |
| persistence.size | string | `"5Gi"` | PVC size for the AI-local metadata snapshot (small). |
| persistence.storageClass | string | `""` | StorageClass for the snapshot PVC. Empty uses the cluster default. |
| podAnnotations | object | `{}` | Annotations added to the pod template. |
| podLabels | object | `{}` | Extra labels added to the pod template (not the selector). |
| probes.liveness.failureThreshold | int | `3` | Liveness probe failure threshold. |
| probes.liveness.initialDelaySeconds | int | `60` | Liveness probe initial delay (s). The startup probe gates first-run model download, so this only applies once the app has started responding. |
| probes.liveness.periodSeconds | int | `30` | Liveness probe period (s). |
| probes.liveness.timeoutSeconds | int | `5` | Liveness probe timeout (s). |
| probes.readiness.failureThreshold | int | `3` | Readiness probe failure threshold. |
| probes.readiness.initialDelaySeconds | int | `10` | Readiness probe initial delay (s). |
| probes.readiness.periodSeconds | int | `10` | Readiness probe period (s). |
| probes.readiness.timeoutSeconds | int | `5` | Readiness probe timeout (s). |
| probes.startup.failureThreshold | int | `30` | Startup probe failure threshold. `periodSeconds` × `failureThreshold` is the longest a first-run model download may take before the pod is restarted (default 10s × 30 = 5 min). Liveness and readiness are suspended until the startup probe first succeeds, so the pod is never killed mid-download. Raise this for large models or slow networks. |
| probes.startup.initialDelaySeconds | int | `10` | Startup probe initial delay (s). |
| probes.startup.periodSeconds | int | `10` | Startup probe period (s). |
| probes.startup.timeoutSeconds | int | `5` | Startup probe timeout (s). |
| resources | object | `{}` | Container resource requests/limits. AI is heavier than DB; for GPU add `nvidia.com/gpu` to limits plus a matching nodeSelector. |
| service.port | int | `1370` | `--port`, the public client-facing gRPC port. |
| service.type | string | `"ClusterIP"` | Service type: `ClusterIP`, `LoadBalancer`, or `NodePort`. |
| tolerations | list | `[]` | Tolerations for pod scheduling. |
| tracing.enabled | bool | `false` | Pass `--enable-tracing` and `--otel-endpoint`. |
| tracing.otelEndpoint | string | `""` | OTLP gRPC endpoint, e.g. `http://jaeger-collector:4317`. |

## Notes

- **Backing DB.** The AI pod connects to the DB over gRPC at `db.host:db.port`. For a co-located DB install named `my-ahnlich`, that's typically `my-ahnlich-ahnlich-db` (same namespace) or `my-ahnlich-ahnlich-db.<db-namespace>.svc.cluster.local` (cross-namespace). The `db.waitForReady` initContainer blocks startup until that port is reachable; disable it only if the DB lives somewhere k8s can't reach via a Service.
- **Model cache.** Each replica gets its own model-cache PVC. Models are downloaded on first use and reused across restarts. The liveness probe's 60s initial delay covers first-run download.
- **AI cluster scope.** In cluster mode the AI Raft cluster only replicates store metadata (CreateStore, DropStore, PurgeStores). All other mutations are proxied to the DB cluster via `db.host` and replicated by the DB's own Raft cluster.
- **Logging.** `ahnlich-ai` reads its log filter from `--log-level`, **not** `RUST_LOG`. Use `logLevel` for verbosity; `env`/`envFrom` for other process env.
- **GPU.** The image ships the CUDA ONNX Runtime. Add `nvidia.com/gpu` to `resources.limits` and a matching `nodeSelector` to schedule on GPU nodes.

The Service port advertises `appProtocol: grpc` for Ingress/Gateway controller auto-discovery. Enabling `ingress.enabled` and `gateway.enabled` together is a misconfiguration; the chart fails the install with an explicit error. See the `ahnlich-db` README for the controller-specific gRPC notes and external-access patterns — the surface is identical here.

## Upgrading

`helm upgrade my-ai charts/ahnlich-ai` rolls pods one at a time. PVCs (model cache and persistence data) survive the upgrade. In cluster mode, the PodDisruptionBudget (`maxUnavailable: 1`) keeps quorum intact.

## Uninstalling

```bash
helm uninstall my-ai
kubectl delete pvc -l app.kubernetes.io/instance=my-ai   # only if you also want to drop cached models / data
```
