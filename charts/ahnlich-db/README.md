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

### Image

| Key | Default | Description |
|---|---|---|
| `image.repository` | `ghcr.io/deven96/ahnlich-db` | Image repository. |
| `image.tag` | `latest` | Image tag. |
| `image.pullPolicy` | `IfNotPresent` | Standard k8s pull policy. |

### Service

| Key | Default | Description |
|---|---|---|
| `service.type` | `ClusterIP` | One of `ClusterIP`, `LoadBalancer`, `NodePort`. |
| `service.port` | `1369` | Public client-facing gRPC port. |

The Service port advertises `appProtocol: grpc` so modern Ingress / Gateway controllers auto-configure HTTP/2 to the backend without per-controller annotations.

### Standalone persistence

| Key | Default | Description |
|---|---|---|
| `persistence.enabled` | `true` | Mount a PVC at `mountPath` and pass `--enable-persistence`. |
| `persistence.size` | `10Gi` | PVC size for the JSON snapshot file. |
| `persistence.storageClass` | `""` | StorageClass name; empty = cluster default. |
| `persistence.mountPath` | `/root/.ahnlich/data` | In-container mount point. |
| `persistence.fileName` | `db.dat` | Snapshot file name. |
| `persistence.intervalMs` | `30000` | `--persistence-interval` value in ms. |

Automatically suppressed when `cluster.enabled=true` (Raft snapshots and log replication replace standalone persistence).

### Cluster (Raft) mode

| Key | Default | Description |
|---|---|---|
| `cluster.enabled` | `false` | Run N replicas as a Raft cluster. |
| `cluster.replicas` | `3` | StatefulSet replica count. |
| `cluster.port` | `1370` | `--cluster-addr` port for Raft RPCs. |
| `cluster.storage` | `rocksdb` | `rocksdb` (production) or `memory` (testing only). |
| `cluster.dataDir` | `/root/.ahnlich/raft` | `--cluster-data-dir`. |
| `cluster.snapshotLogs` | `1000` | Count-based snapshot trigger. |
| `cluster.snapshotIntervalMs` | `300000` | Time-based snapshot trigger. |
| `cluster.persistence.size` | `20Gi` | PVC size for the Raft data dir. |
| `cluster.persistence.storageClass` | `""` | StorageClass name; empty = cluster default. |
| `cluster.podDisruptionBudget.enabled` | `true` | Protect quorum during voluntary disruptions. |
| `cluster.podDisruptionBudget.minAvailable` | `2` | Minimum pods that must stay up. |

When enabled, the chart also creates a headless Service for pod DNS resolution (Raft peer discovery) and a PodDisruptionBudget.

The shell-wrapper in the StatefulSet args derives `--cluster-bootstrap` for pod-0 (when its data dir is empty) and `--cluster-join` for higher ordinals pointing at pod-0's headless DNS name. Restarts of existing pods leave both flags off so the binary recovers from the persisted Raft log (`ClusterLifecycle::Existing`).

### Tracing

| Key | Default | Description |
|---|---|---|
| `tracing.enabled` | `false` | Pass `--enable-tracing --otel-endpoint=...`. |
| `tracing.otelEndpoint` | `""` | OTLP gRPC endpoint (e.g. `http://jaeger-collector:4317`). |

The chart does not deploy a tracing backend. Point `tracing.otelEndpoint` at any OTLP-compatible receiver: Jaeger, Tempo, an OpenTelemetry Collector, Honeycomb, etc.

### Ingress (optional)

| Key | Default | Description |
|---|---|---|
| `ingress.enabled` | `false` | Create an `Ingress` resource. |
| `ingress.className` | `""` | IngressClass name; empty = cluster default. |
| `ingress.host` | `""` | Required if `ingress.tls.enabled=true`. |
| `ingress.path` | `/` | Path prefix. |
| `ingress.pathType` | `Prefix` | One of `Prefix`, `Exact`, `ImplementationSpecific`. |
| `ingress.annotations` | `{}` | Controller-specific annotations (see below). |
| `ingress.tls.enabled` | `false` | Add a `tls` block. |
| `ingress.tls.secretName` | `""` | Name of an existing TLS Secret. |

Controller-specific gRPC notes:

- **nginx-ingress**: add `nginx.ingress.kubernetes.io/backend-protocol: GRPC` to `ingress.annotations`. The chart's `appProtocol: grpc` is the standard hint but older nginx versions ignore it.
- **Traefik**: as Ingress Controller, requires service annotation `traefik.ingress.kubernetes.io/service.serversscheme: h2c` for plaintext gRPC. For TLS-terminating gRPC, defaults work.

### Gateway API (optional, mutually exclusive with Ingress)

| Key | Default | Description |
|---|---|---|
| `gateway.enabled` | `false` | Create a `GRPCRoute` attached to an existing Gateway. |
| `gateway.parentRefs` | `[{name: "", ...}]` | Existing Gateway references; `name` is required when enabled. |
| `gateway.hostnames` | `[]` | Hostnames the route claims. |

The chart only ships the `GRPCRoute`. The `Gateway` itself is cluster-level and must be created out-of-band (typically by your platform team).

Enabling `ingress.enabled` and `gateway.enabled` simultaneously is a misconfiguration; the chart fails the install with an explicit error.

### Pod scheduling

| Key | Default | Description |
|---|---|---|
| `resources` | `{}` | Container `resources` block (requests/limits). |
| `nodeSelector` | `{}` | Schedule on nodes matching these labels. |
| `tolerations` | `[]` | Tolerate node taints. |
| `affinity` | `{}` | Pod / node affinity rules. |
| `podAnnotations` | `{}` | Free-form annotations on the pod template. |

### Probes

`probes.readiness.*` and `probes.liveness.*` accept the standard k8s timing fields: `initialDelaySeconds`, `periodSeconds`, `timeoutSeconds`, `failureThreshold`. Both probes run `ahnlich-cli PING` against the local pod.

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

In cluster mode, an upgrade rolls one pod at a time; with the PodDisruptionBudget at `minAvailable: 2`, quorum stays intact through the upgrade.

## Uninstalling

```bash
helm uninstall my-ahnlich
```

PVCs are not deleted automatically (Helm intentionally leaves them; data deletion is destructive). Clean them up with:

```bash
kubectl delete pvc -l app.kubernetes.io/instance=my-ahnlich
```
