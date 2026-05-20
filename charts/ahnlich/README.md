# ahnlich

Umbrella Helm chart that deploys [`ahnlich-db`](../ahnlich-db/) and [`ahnlich-ai`](../ahnlich-ai/) together, pre-wired so the AI talks to the DB out of the box. Optionally deploys an in-cluster OTLP tracing backend (Jaeger by default; image is configurable for other OTLP receivers).

## Quick start

The release name must be `ahnlich` for the default DB host wiring to resolve. To use a different release name, also override `ahnlich-ai.db.host`.

```bash
# Build sub-chart dependencies (one time, after cloning)
helm dependency build charts/ahnlich

# Standalone DB + AI
helm install ahnlich charts/ahnlich

# Cluster (Raft) mode for both
helm install ahnlich charts/ahnlich \
  --set ahnlich-db.cluster.enabled=true \
  --set ahnlich-ai.cluster.enabled=true

# DB in cluster mode, AI standalone (mix-and-match is fine)
helm install ahnlich charts/ahnlich \
  --set ahnlich-db.cluster.enabled=true

# With the in-cluster tracing backend (defaults to Jaeger all-in-one)
helm install ahnlich charts/ahnlich \
  --set tracing.enabled=true \
  --set tracing.backend.enabled=true \
  --set ahnlich-db.tracing.enabled=true \
  --set ahnlich-ai.tracing.enabled=true
```

For real deployments, a `-f values-prod.yaml` file is friendlier than chaining `--set`.

## Pattern

The umbrella is a thin aggregator: it declares `ahnlich-db` and `ahnlich-ai` as sub-chart dependencies and forwards values into them under their respective namespaces. Every sub-chart toggle remains independently controllable:

```yaml
# values.yaml
ahnlich-db:
  cluster:
    enabled: true
  service:
    type: LoadBalancer
ahnlich-ai:
  cluster:
    enabled: false       # mixed mode is fine
  db:
    host: ahnlich-ahnlich-db
  models:
    supported: [all-minilm-l6-v2, resnet-50, clip-vit-b32-text]
```

This is intentional — debugging or running mixed configurations (DB clustered, AI standalone for testing, etc.) is a normal part of the workflow.

## Values

### Top-level (umbrella's own resources)

| Key | Default | Description |
|---|---|---|
| `tracing.enabled` | `false` | Master switch for any tracing wiring. Setting this alone does nothing until at least one of `tracing.backend.enabled` or per-sub-chart `tracing.enabled` is also set. |
| `tracing.backend.enabled` | `false` | Deploy an in-cluster OTLP receiver as a Deployment + Service. |
| `tracing.backend.image.repository` | `jaegertracing/all-in-one` | Default backend image. Override for other OTLP receivers. |
| `tracing.backend.image.tag` | `latest` | |
| `tracing.backend.env` | `[{COLLECTOR_OTLP_ENABLED: "true"}]` | Backend container env. The default is Jaeger-shaped. |
| `tracing.backend.ports.otlp` | `4317` | OTLP gRPC port that ahnlich-db / ahnlich-ai send to. |
| `tracing.backend.ports.ui` | `16686` | Browser UI port (Jaeger default). |
| `tracing.backend.service.type` | `ClusterIP` | |

The backend's Service is named `<release>-tracing-backend`. With the default release name `ahnlich`, that resolves to `ahnlich-tracing-backend`, which is what `ahnlich-db.tracing.otelEndpoint` and `ahnlich-ai.tracing.otelEndpoint` default to.

### Sub-chart values

Set anything from the [`ahnlich-db`](../ahnlich-db/README.md) or [`ahnlich-ai`](../ahnlich-ai/README.md) charts via the namespaced key:

```bash
--set ahnlich-db.<key>=<value>
--set ahnlich-ai.<key>=<value>
```

Pre-wired defaults in the umbrella's `values.yaml`:

- `ahnlich-ai.db.host` → `ahnlich-ahnlich-db` (the DB Service name when release is `ahnlich`).
- `ahnlich-db.tracing.otelEndpoint` → `http://ahnlich-tracing-backend:4317`.
- `ahnlich-ai.tracing.otelEndpoint` → `http://ahnlich-tracing-backend:4317`.

To enable tracing per-sub-chart, override `tracing.enabled` under each:

```bash
--set ahnlich-db.tracing.enabled=true
--set ahnlich-ai.tracing.enabled=true
```

### Using a non-`ahnlich` release name

The DB Service is named `<release>-ahnlich-db`. If you install as `foo`, override the AI's `db.host`:

```bash
helm install foo charts/ahnlich \
  --set ahnlich-ai.db.host=foo-ahnlich-db
```

The post-install message will warn you when the release name doesn't match `ahnlich`.

## External access

The umbrella does not impose a global external-access pattern. Each sub-chart's `service.type`, `ingress.*`, and `gateway.*` blocks work independently. Typically you only expose `ahnlich-ai` externally (it sits in front of `ahnlich-db`):

```bash
helm install ahnlich charts/ahnlich \
  --set ahnlich-ai.service.type=LoadBalancer
# or
helm install ahnlich charts/ahnlich \
  --set ahnlich-ai.gateway.enabled=true \
  --set 'ahnlich-ai.gateway.parentRefs[0].name=my-gateway'
```

## Operational notes

- **The bundled tracing backend is dev/demo only.** Jaeger all-in-one runs as a single replica with in-memory span storage, no auth, and `:latest`. For anything beyond local poking, set `tracing.backend.enabled=false` and point `ahnlich-db.tracing.otelEndpoint` / `ahnlich-ai.tracing.otelEndpoint` at your own OTLP collector (Tempo, Honeycomb, vendor SDK, etc.).
- **TLS for Ingress.** Most gRPC clients reject plaintext HTTP/2 on a public hostname; enable `ingress.tls` (and bring a cert via cert-manager or a manual Secret) for any non-local install.
- **Sub-chart version bumps need `helm dependency update`.** The umbrella declares `ahnlich-db` and `ahnlich-ai` by version. When you bump either sub-chart's `Chart.yaml` `version:`, run `helm dependency update charts/ahnlich` to refresh `Chart.lock` and re-vendor.
- **Toggling `persistence.enabled` between upgrades fails.** StatefulSet `volumeClaimTemplates` is immutable; flipping persistence on/off requires `helm uninstall` then re-install (which orphans PVCs by default — clean them up via `kubectl delete pvc -l app.kubernetes.io/instance=<release>`).
- **`:latest` and `imagePullPolicy`.** When `image.tag=latest`, the chart sets `imagePullPolicy: Always` so `helm upgrade` actually pulls the latest digest. Tagged versions default to `IfNotPresent`. Override `image.pullPolicy` per sub-chart to force either.

## Uninstalling

```bash
helm uninstall ahnlich
kubectl delete pvc -l app.kubernetes.io/instance=ahnlich   # drops cached models and persistence
```
