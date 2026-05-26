# ahnlich-db

Helm chart for `ahnlich-db`, the in-memory vector database.

The chart supports two deployment modes from a single chart, toggled by a values flag:

- **Standalone** (default): single replica with periodic JSON-snapshot persistence. Suitable for development and single-node production.
- **Cluster (Raft)**: N replicas forming a Raft cluster per the [replication spec](../../docs/specs/replication.md). Activate with `--set cluster.enabled=true`.

## Prerequisites

- Kubernetes 1.20+ (for `appProtocol: grpc` on the Service).
- A default `StorageClass` in the cluster (the chart falls back to it unless overridden).
- Optional, only if you want external L7 access:
  - An Ingress Controller (nginx-ingress, Traefik, etc.) for the Ingress path.
  - A Gateway API implementation (Envoy Gateway, Contour, etc.) plus an existing `Gateway` resource for the Gateway path.

The chart does **not** bundle an Ingress Controller or a Gateway implementation — those are cluster-wide infrastructure.

## Quick start

```bash
# Standalone, in-cluster access only
helm install my-ahnlich charts/ahnlich-db

# Standalone, exposed externally via a cloud LoadBalancer
helm install my-ahnlich charts/ahnlich-db --set service.type=LoadBalancer

# With tracing pointed at an existing OTLP collector
helm install my-ahnlich charts/ahnlich-db \
  --set tracing.enabled=true \
  --set tracing.otelEndpoint=http://jaeger-collector:4317
```

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| affinity | object | `{}` | Affinity rules for pod scheduling. |
| cluster.dataDir | string | `"/root/.ahnlich/raft"` | `--cluster-data-dir` (RocksDB log + snapshots). |
| cluster.enabled | bool | `false` | Run N replicas as a Raft cluster instead of a single standalone node. |
| cluster.persistence.size | string | `"20Gi"` | PVC size for the Raft data dir (larger default: logs grow faster than periodic snapshots). |
| cluster.persistence.storageClass | string | `""` | StorageClass for the Raft data dir. Empty uses the cluster default. |
| cluster.podDisruptionBudget.enabled | bool | `true` | Create a PodDisruptionBudget in cluster mode. |
| cluster.podDisruptionBudget.maxUnavailable | int | `1` | Pods that may be evicted at once; scales with replicas. |
| cluster.port | int | `1370` | `--cluster-addr` port for Raft RPCs. |
| cluster.replicas | int | `3` | StatefulSet replica count in cluster mode. |
| cluster.snapshotIntervalMs | int | `300000` | `--cluster-snapshot-interval` time-based snapshot trigger (ms). |
| cluster.snapshotLogs | int | `1000` | `--cluster-snapshot-logs` count-based snapshot trigger. |
| cluster.storage | string | `"rocksdb"` | Raft storage backend: `rocksdb` (production) or `memory` (testing only, no durability). |
| env | list | `[]` | Extra environment variables (name/value or valueFrom). Use for Rust runtime knobs like `RUST_BACKTRACE`. |
| envFrom | list | `[]` | Bulk-import env from ConfigMaps or Secrets. |
| gateway.enabled | bool | `false` | Create a GRPCRoute attached to an existing Gateway (mutually exclusive with `ingress`). |
| gateway.hostnames | list | `[]` | Hostnames the route claims, e.g. `["ahnlich-db.example.com"]`. |
| gateway.parentRefs | list | `[{"name":"","namespace":"","sectionName":""}]` | Existing Gateway references; `name` is required when `gateway.enabled`. |
| image.pullPolicy | string | `""` | Pull policy. Empty selects `Always` for the `latest` tag, `IfNotPresent` otherwise. |
| image.repository | string | `"ghcr.io/deven96/ahnlich-db"` | Image repository. |
| image.tag | string | `"latest"` | Image tag. |
| ingress.annotations | object | `{}` | Controller-specific annotations (e.g. nginx-ingress `backend-protocol: GRPC`). |
| ingress.className | string | `""` | IngressClass name. Empty uses the cluster default. |
| ingress.enabled | bool | `false` | Create an Ingress resource (requires an Ingress Controller in the cluster). |
| ingress.host | string | `""` | Hostname, e.g. `ahnlich-db.example.com`. Required when `ingress.tls.enabled`. |
| ingress.path | string | `"/"` | Path prefix. |
| ingress.pathType | string | `"Prefix"` | Path type: `Prefix`, `Exact`, or `ImplementationSpecific`. |
| ingress.tls.enabled | bool | `false` | Add a TLS block to the Ingress. |
| ingress.tls.secretName | string | `""` | Name of an existing TLS Secret in this namespace. |
| logLevel | string | `""` | `--log-level` (env_logger / tracing-subscriber syntax). Empty uses the binary default (`info,hf_hub=warn`). |
| nodeSelector | object | `{}` | Node selector for pod scheduling. |
| persistence.enabled | bool | `true` | Mount a PVC and pass `--enable-persistence` (standalone mode only). |
| persistence.fileName | string | `"db.dat"` | Snapshot file name. |
| persistence.intervalMs | int | `30000` | `--persistence-interval` value (ms). |
| persistence.mountPath | string | `"/root/.ahnlich/data"` | In-container mount point for the snapshot. |
| persistence.size | string | `"10Gi"` | PVC size for the JSON snapshot file. |
| persistence.storageClass | string | `""` | StorageClass for the snapshot PVC. Empty uses the cluster default. |
| podAnnotations | object | `{}` | Annotations added to the pod template. |
| podLabels | object | `{}` | Extra labels added to the pod template (not the selector). |
| probes.liveness.failureThreshold | int | `3` | Liveness probe failure threshold. |
| probes.liveness.initialDelaySeconds | int | `30` | Liveness probe initial delay (s). |
| probes.liveness.periodSeconds | int | `30` | Liveness probe period (s). |
| probes.liveness.timeoutSeconds | int | `5` | Liveness probe timeout (s). |
| probes.readiness.failureThreshold | int | `3` | Readiness probe failure threshold. |
| probes.readiness.initialDelaySeconds | int | `5` | Readiness probe initial delay (s). |
| probes.readiness.periodSeconds | int | `10` | Readiness probe period (s). |
| probes.readiness.timeoutSeconds | int | `5` | Readiness probe timeout (s). |
| resources | object | `{}` | Container resource requests/limits. |
| service.port | int | `1369` | `--port`, the public client-facing gRPC port. |
| service.type | string | `"ClusterIP"` | Service type: `ClusterIP`, `LoadBalancer`, or `NodePort`. |
| tolerations | list | `[]` | Tolerations for pod scheduling. |
| tracing.enabled | bool | `false` | Pass `--enable-tracing` and `--otel-endpoint`. |
| tracing.otelEndpoint | string | `""` | OTLP gRPC endpoint, e.g. `http://jaeger-collector:4317`. |

The Service port advertises `appProtocol: grpc` so modern Ingress / Gateway controllers auto-configure HTTP/2 to the backend without per-controller annotations.

In cluster mode the chart also creates a headless Service for pod DNS resolution (Raft peer discovery) and a PodDisruptionBudget. The StatefulSet args derive `--cluster-bootstrap` for pod-0 (when its data dir is empty) and `--cluster-join` for higher ordinals pointing at pod-0's headless DNS name; restarts of existing pods leave both flags off so the binary recovers from the persisted Raft log (`ClusterLifecycle::Existing`).

`ahnlich-db` reads its log filter from `--log-level`, **not** from `RUST_LOG`; setting `RUST_LOG` via `env` has no effect. Use `logLevel` for verbosity and `env`/`envFrom` for other process env (`RUST_BACKTRACE`, Secret refs, etc.).

### Controller-specific gRPC notes

- **nginx-ingress**: add `nginx.ingress.kubernetes.io/backend-protocol: GRPC` to `ingress.annotations`. The chart's `appProtocol: grpc` is the standard hint but older nginx versions ignore it.
- **Traefik (Ingress Controller)**: requires service annotation `traefik.ingress.kubernetes.io/service.serversscheme: h2c` for plaintext gRPC. For TLS-terminating gRPC, defaults work.

Enabling `ingress.enabled` and `gateway.enabled` simultaneously is a misconfiguration; the chart fails the install with an explicit error.

## External access patterns

Three ways to expose the DB to clients outside the cluster, in increasing order of features:

1. **`service.type=LoadBalancer`** — one external IP per Service, L4 only. Simplest. Cheap in single-app deployments, expensive at scale (one cloud LB per Service).
2. **`ingress.enabled=true`** — single shared edge router (the Ingress Controller) fans out to many apps. Adds TLS termination, hostname routing, controller-specific policies. Requires an Ingress Controller in the cluster.
3. **`gateway.enabled=true`** — same idea as Ingress with a cleaner, role-split API (Gateway API). Adds first-class gRPC matching (`GRPCRoute` can match on service + method). Requires a Gateway API implementation in the cluster.

For in-cluster service-to-service traffic (e.g. `ahnlich-ai` calling `ahnlich-db`), leave `service.type=ClusterIP` and reach the DB at `<release>-ahnlich-db.<namespace>.svc.cluster.local`. No external access needed.

## Example: attach to an existing Envoy Gateway

```bash
# Cluster-level: GatewayClass + Gateway already exist
kubectl get gatewayclass eg
kubectl -n ahnlich get gateway eg

# Install the chart attached to the Gateway
helm install ahnlich charts/ahnlich-db \
  --namespace ahnlich \
  --set gateway.enabled=true \
  --set gateway.parentRefs[0].name=eg \
  --set gateway.hostnames[0]=ahnlich-db.example.com
```

The chart's GRPCRoute attaches to the Gateway named `eg` in the same namespace. The Gateway's external address becomes the route's address.

## Example: attach to an existing nginx-ingress

```bash
helm install ahnlich charts/ahnlich-db \
  --namespace ahnlich \
  --set ingress.enabled=true \
  --set ingress.className=nginx \
  --set ingress.host=ahnlich-db.example.com \
  --set 'ingress.annotations.nginx\.ingress\.kubernetes\.io/backend-protocol=GRPC' \
  --set ingress.tls.enabled=true \
  --set ingress.tls.secretName=ahnlich-db-tls
```

## Upgrading

`helm upgrade my-ahnlich charts/ahnlich-db` re-renders against the new values and applies the diff. The StatefulSet's pods are rolled one at a time. The PVC survives the upgrade (PVCs are not deleted by Helm).

In cluster mode, the PodDisruptionBudget (`maxUnavailable: 1`) keeps quorum intact through a rolling upgrade.

## Uninstalling

```bash
helm uninstall my-ahnlich
```

PVCs are not deleted automatically (Helm intentionally leaves them; data deletion is destructive). Clean them up with:

```bash
kubectl delete pvc -l app.kubernetes.io/instance=my-ahnlich
```
