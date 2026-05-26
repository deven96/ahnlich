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

The umbrella is a thin aggregator: it declares `ahnlich-db` and `ahnlich-ai` as sub-chart dependencies and forwards values into them under their respective namespaces. Every sub-chart toggle remains independently controllable, so mixed configurations (DB clustered, AI standalone for testing, etc.) are a normal part of the workflow.

Set anything from the [`ahnlich-db`](../ahnlich-db/README.md) or [`ahnlich-ai`](../ahnlich-ai/README.md) charts via the namespaced key: `--set ahnlich-db.<key>=<value>` or `--set ahnlich-ai.<key>=<value>`.

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| ahnlich-ai.db.host | string | `"ahnlich-ahnlich-db"` | DB Service name (auto-wired; assumes release name "ahnlich"). |
| ahnlich-ai.db.port | int | `1369` | DB port. |
| ahnlich-ai.tracing.enabled | bool | `false` | Send AI traces to the bundled backend. |
| ahnlich-ai.tracing.otelEndpoint | string | `"http://ahnlich-tracing-backend:4317"` | OTLP endpoint (pre-wired to the bundled backend Service for release name "ahnlich"). |
| ahnlich-db.tracing.enabled | bool | `false` | Send DB traces to the bundled backend. |
| ahnlich-db.tracing.otelEndpoint | string | `"http://ahnlich-tracing-backend:4317"` | OTLP endpoint (pre-wired to the bundled backend Service for release name "ahnlich"). |
| tracing.backend.enabled | bool | `false` | Deploy an in-cluster OTLP backend (Deployment + Service). |
| tracing.backend.env | list | `[{"name":"COLLECTOR_OTLP_ENABLED","value":"true"}]` | Backend container env. Default is Jaeger-shaped. |
| tracing.backend.image.pullPolicy | string | `"IfNotPresent"` | Backend image pull policy. |
| tracing.backend.image.repository | string | `"jaegertracing/all-in-one"` | Backend image repository (default Jaeger all-in-one; swap for other OTLP receivers). |
| tracing.backend.image.tag | string | `"latest"` | Backend image tag. |
| tracing.backend.ports.otlp | int | `4317` | OTLP gRPC port the sub-charts send to. |
| tracing.backend.ports.ui | int | `16686` | Browser UI port. |
| tracing.backend.service.type | string | `"ClusterIP"` | Backend Service type. |
| tracing.enabled | bool | `false` | Master switch for tracing wiring. Does nothing alone; pair with `tracing.backend.enabled` and/or per-sub-chart `tracing.enabled`. |

The backend's Service is named `<release>-tracing-backend`. With the default release name `ahnlich`, that resolves to `ahnlich-tracing-backend`, which is what the pre-wired `otelEndpoint` defaults point at. Install under a different name and you must override both `ahnlich-ai.db.host` and the `otelEndpoint`s — the chart fails at template time with the exact override command if you don't.

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

- **The bundled tracing backend is dev/demo only.** Jaeger all-in-one runs as a single replica with in-memory span storage, no auth, and `:latest`. For anything beyond local poking, set `tracing.backend.enabled=false` and point the sub-chart `otelEndpoint`s at your own OTLP collector (Tempo, Honeycomb, vendor SDK, etc.).
- **TLS for Ingress.** Most gRPC clients reject plaintext HTTP/2 on a public hostname; enable `ingress.tls` (and bring a cert via cert-manager or a manual Secret) for any non-local install.
- **Sub-chart version bumps need `helm dependency update`.** The umbrella declares `ahnlich-db` and `ahnlich-ai` by version. When you bump either sub-chart's `Chart.yaml` `version:`, run `helm dependency update charts/ahnlich` to refresh `Chart.lock` and re-vendor.
- **Toggling `persistence.enabled` between upgrades fails.** StatefulSet `volumeClaimTemplates` is immutable; flipping persistence on/off requires `helm uninstall` then re-install (which orphans PVCs by default — clean them up via `kubectl delete pvc -l app.kubernetes.io/instance=<release>`).
- **`:latest` and `imagePullPolicy`.** When `image.tag=latest`, the chart sets `imagePullPolicy: Always` so `helm upgrade` actually pulls the latest digest. Tagged versions default to `IfNotPresent`.

## Uninstalling

```bash
helm uninstall ahnlich
kubectl delete pvc -l app.kubernetes.io/instance=ahnlich   # drops cached models and persistence
```
