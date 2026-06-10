---
title: Kubernetes (Helm)
sidebar_position: 15
---

# Kubernetes (Helm)

Ahnlich ships official Helm charts for running both services on Kubernetes:

- **`ahnlich-db`** — the in-memory vector store (StatefulSet, client gRPC on port `1369`)
- **`ahnlich-ai`** — the embedding/AI proxy that sits in front of the DB (StatefulSet, client gRPC on port `1370`)
- **`ahnlich`** — an **umbrella** chart that deploys both, pre-wired so the AI proxy talks to the DB out of the box, with an optional in-cluster tracing backend

Reach for Kubernetes when you want self-healing pods, rolling upgrades, persistent
volumes managed by the cluster, and horizontal scaling. For a single box or local
experimentation, the [Docker Compose setup](./deployment) is simpler.

:::note No Helm repository yet
The charts are not published to a Helm/OCI registry at this time. You install them
straight from the source tree, as shown below. (`helm repo add` is not available yet.)
:::

## Prerequisites

- A running Kubernetes cluster and a `kubectl` context pointing at it. Any cluster
  works — [Rancher Desktop](https://rancherdesktop.io/), kind, or minikube locally;
  EKS / GKE / AKS in production.
- [Helm](https://helm.sh/docs/intro/install/) 3.8+ (or 4.x).
- Kubernetes 1.20+ (the Services set `appProtocol: grpc`; the Gateway API path additionally
  needs the Gateway API CRDs and a controller — see [External access](#external-access)).
- `kubectl`, configured for the target cluster:

  ```bash
  kubectl config current-context
  kubectl get nodes
  ```

## Get the Charts

There is no chart repository yet, so fetch the charts from GitHub. You only need the
`charts/` directory — not the rest of the codebase.

Clone the whole repo:

```bash
git clone https://github.com/deven96/ahnlich.git
cd ahnlich
```

Or grab only `charts/` with a sparse checkout (smaller, no source code):

```bash
git clone --depth 1 --filter=blob:none --sparse https://github.com/deven96/ahnlich.git
cd ahnlich
git sparse-checkout set charts
```

Then vendor the umbrella's sub-chart dependencies once:

```bash
helm dependency build charts/ahnlich
```

This resolves the `ahnlich-db` and `ahnlich-ai` sub-charts that the umbrella depends on.
Re-run it whenever a sub-chart version changes.

## Install (Umbrella Chart)

The umbrella is the recommended path: it installs `ahnlich-db` and `ahnlich-ai` together
and wires the AI proxy to the DB automatically.

```bash
kubectl create namespace ahnlich

helm install ahnlich charts/ahnlich \
  --namespace ahnlich \
  --set 'ahnlich-ai.models.supported={all-minilm-l6-v2}'
```

:::warning The release name must be `ahnlich`
The umbrella's default DB wiring (`ahnlich-ai.db.host=ahnlich-ahnlich-db`) assumes the
release is named `ahnlich`. Install under any other name and the chart **fails at
template time** with the exact override to use. To use a different name, also pass
`--set ahnlich-ai.db.host=<release>-ahnlich-db`.
:::

:::tip Keep the first install fast
The AI proxy downloads its embedding models on first start. The example overrides
`ahnlich-ai.models.supported` to just `all-minilm-l6-v2` so the initial model pull is
quick. Drop the `--set` to get the chart default (`all-minilm-l6-v2`, `resnet-50`), or
list whatever models you need.
:::

The AI pod blocks on a `wait-for-db` init container until the DB is reachable, so it is
normal for `ahnlich-ahnlich-ai-0` to sit in `Init:0/1` for a moment while the DB comes up
and the model downloads. Watch progress with:

```bash
kubectl get pods -n ahnlich -w
```

## Verify the Deployment

Once both StatefulSets report ready:

```bash
kubectl get pods,sts -n ahnlich
```

Send a `PING` to each service from inside its pod:

```bash
# DB
kubectl exec -n ahnlich sts/ahnlich-ahnlich-db -- \
  sh -c "echo PING | ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369 --no-interactive"

# AI
kubectl exec -n ahnlich sts/ahnlich-ahnlich-ai -- \
  sh -c "echo PING | ahnlich-cli ahnlich --agent ai --host 127.0.0.1 --port 1370 --no-interactive"
```

Prove the full AI → DB path by creating a store through the AI proxy and confirming it exists in the DB:

```bash
# Create a store via AI (AI proxies the write to DB)
kubectl exec -n ahnlich sts/ahnlich-ahnlich-ai -- \
  sh -c "echo 'CREATESTORE smoke QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2' | ahnlich-cli ahnlich --agent ai --host 127.0.0.1 --port 1370 --no-interactive"

# Confirm it landed in DB
kubectl exec -n ahnlich sts/ahnlich-ahnlich-db -- \
  sh -c "echo LISTSTORES | ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369 --no-interactive"
```

The `LISTSTORES` output should include `smoke`.

## Connect from Clients

**In-cluster** — other workloads in the cluster reach the services by their Service DNS:

```
ahnlich-ahnlich-db.ahnlich.svc.cluster.local:1369
ahnlich-ahnlich-ai.ahnlich.svc.cluster.local:1370
```

Point an [Ahnlich client](/docs/client-libraries) at the AI proxy's address (or the DB
address for raw vector operations).

**From your laptop** — port-forward for local testing:

```bash
kubectl port-forward -n ahnlich svc/ahnlich-ahnlich-ai 1370:1370
# now connect a client to 127.0.0.1:1370
```

For permanent external access, see [External access](#external-access) below.

## Configuration

Override any value with `--set key=value` or, better for real deployments, a values file:

```bash
helm install ahnlich charts/ahnlich -n ahnlich -f my-values.yaml
```

Umbrella values are namespaced per sub-chart — set DB values under `ahnlich-db.*` and AI
values under `ahnlich-ai.*`, e.g. `--set ahnlich-db.persistence.size=50Gi`.

Common knobs:

| What | Key | Default |
|------|-----|---------|
| DB snapshot PVC size | `ahnlich-db.persistence.size` | `10Gi` |
| AI model cache PVC size | `ahnlich-ai.models.cache.size` | `20Gi` |
| PVC StorageClass | `ahnlich-db.persistence.storageClass`, `ahnlich-ai.models.cache.storageClass` | `""` (cluster default) |
| DB snapshot interval (ms) | `ahnlich-db.persistence.intervalMs` | `30000` |
| Models served by AI | `ahnlich-ai.models.supported` | `[all-minilm-l6-v2, resnet-50]` |
| Container resources | `ahnlich-db.resources`, `ahnlich-ai.resources` | `{}` (unset) |
| Log level | `ahnlich-db.logLevel`, `ahnlich-ai.logLevel` | binary default (`info,hf_hub=warn`) |
| Extra env vars | `ahnlich-db.env`, `ahnlich-ai.env` | `[]` |
| Bulk env from ConfigMap/Secret | `ahnlich-db.envFrom`, `ahnlich-ai.envFrom` | `[]` |

Persistence is **on by default** for both services: each writes a periodic snapshot
(`db.dat` / `ai.dat`) to its PVC every `persistence.intervalMs` (default 30s) and reloads
it on startup. Data survives pod restarts, but an ungraceful kill can lose up to one
interval's worth of the most recent writes. The full set of values, with descriptions,
lives in each chart's README:

- [`ahnlich-db` values](https://github.com/deven96/ahnlich/blob/main/charts/ahnlich-db/README.md)
- [`ahnlich-ai` values](https://github.com/deven96/ahnlich/blob/main/charts/ahnlich-ai/README.md)
- [`ahnlich` (umbrella) values](https://github.com/deven96/ahnlich/blob/main/charts/ahnlich/README.md)

:::note GPU for the AI service
`ahnlich-ai`'s image bundles the CUDA ONNX Runtime. To run embeddings on GPU nodes, add
`nvidia.com/gpu` to `ahnlich-ai.resources.limits` and a matching `ahnlich-ai.nodeSelector`
(the cluster needs the NVIDIA device plugin).
:::

:::note Log level is `--log-level`, not `RUST_LOG`
Set verbosity via `ahnlich-db.logLevel` / `ahnlich-ai.logLevel` (mapped to the binary's
`--log-level`). Setting `RUST_LOG` through `env` has no effect.
:::

## External Access

The umbrella imposes no global external-access pattern; each sub-chart's `service.type`,
`ingress.*`, and `gateway.*` blocks work independently and apply to **both** `ahnlich-db`
and `ahnlich-ai`. Typically you expose **only the AI proxy** (it fronts the DB), but the
same knobs expose the DB on `1369` if you need raw vector access from outside.

### LoadBalancer Service (Simplest)

```bash
helm upgrade ahnlich charts/ahnlich -n ahnlich --reuse-values \
  --set ahnlich-ai.service.type=LoadBalancer
```

Your cloud provider assigns an external IP (locally, Rancher Desktop / k3s `servicelb`
uses the node IP). Connect a gRPC client to `<EXTERNAL-IP>:1370`. Set
`ahnlich-db.service.type=LoadBalancer` likewise to expose DB on `1369`.

### Gateway API (GRPCRoute) — Recommended for gRPC

:::info Requirements — what you provide vs. what the chart provides
When you set `gateway.enabled=true`, the chart contributes **only a `GRPCRoute`** that
attaches the `ahnlich-ai` / `ahnlich-db` Service to a Gateway you already run. The chart
does **not** install a gateway, and intentionally so — the gateway is shared cluster
infrastructure that you own and operate. Before enabling it, your cluster must already
have:

1. **A Gateway API controller** — the component that actually proxies traffic (Envoy
   Gateway, NGINX Gateway Fabric, Contour, Istio, or your cloud's implementation).
2. **The Gateway API CRDs** (`gateway.networking.k8s.io`) — normally installed with the
   controller.
3. **A `GatewayClass`** registered by that controller.
4. **A `Gateway`** with a listener on the port your clients connect to, using a gRPC-capable
   protocol (`HTTP` for h2c, or `HTTPS` with TLS).

The chart binds to that Gateway through `gateway.parentRefs` (and `sectionName` to pick a
listener). If any of the above is missing, the `GRPCRoute` is created but stays unattached
and no traffic flows. If you don't run a Gateway, use the
[LoadBalancer Service](#loadbalancer-service-simplest) path instead.
:::

The walkthrough below uses **Envoy Gateway** as a concrete, working example; any
conformant controller follows the same shape (install controller → `GatewayClass` →
`Gateway` → enable the chart's route).

1. Install a controller:

   ```bash
   helm install eg oci://docker.io/envoyproxy/gateway-helm \
     -n envoy-gateway-system --create-namespace
   kubectl wait --for=condition=available deploy --all -n envoy-gateway-system --timeout=180s
   ```

2. Create a `GatewayClass` and a `Gateway` with a gRPC (HTTP/h2c) listener on the AI port:

   ```yaml
   # gateway.yaml
   apiVersion: gateway.networking.k8s.io/v1
   kind: GatewayClass
   metadata:
     name: eg
   spec:
     controllerName: gateway.envoyproxy.io/gatewayclass-controller
   ---
   apiVersion: gateway.networking.k8s.io/v1
   kind: Gateway
   metadata:
     name: ahnlich-eg
     namespace: ahnlich
   spec:
     gatewayClassName: eg
     listeners:
       - name: ai
         protocol: HTTP        # h2c — gRPC over cleartext HTTP/2
         port: 1370
         allowedRoutes:
           namespaces:
             from: Same
   ```

   ```bash
   kubectl apply -f gateway.yaml
   ```

3. Point the chart's `GRPCRoute` at that listener:

   ```bash
   helm upgrade ahnlich charts/ahnlich -n ahnlich --reuse-values \
     --set ahnlich-ai.service.type=ClusterIP \
     --set ahnlich-ai.gateway.enabled=true \
     --set 'ahnlich-ai.gateway.parentRefs[0].name=ahnlich-eg' \
     --set 'ahnlich-ai.gateway.parentRefs[0].sectionName=ai'
   ```

   `sectionName` binds the route to a named listener. We set it explicitly here; it is
   strictly required only when the Gateway has multiple listeners (e.g. a DB `1369` and an
   AI `1370` listener side by side). Keep the Service at `ClusterIP` so it doesn't contend
   for the node port the Gateway now owns.

   :::note Overriding a multi-Gateway `parentRefs`
   `parentRefs` is a list with a one-element default. Under `--set` / `--reuse-values` Helm
   merges it by index (it patches `[0]`, it does not replace the list). To attach to
   several parent Gateways, pass the whole list via `-f values.yaml` instead of `--set`.
   :::

4. Verify and connect:

   ```bash
   kubectl get gateway ahnlich-eg -n ahnlich     # PROGRAMMED=True, ADDRESS assigned
   kubectl get grpcroute -n ahnlich              # Accepted / ResolvedRefs True
   ```

   Point your client at the Gateway's address on the listener port.

### Ingress

```bash
helm upgrade ahnlich charts/ahnlich -n ahnlich --reuse-values \
  --set ahnlich-ai.ingress.enabled=true \
  --set ahnlich-ai.ingress.className=<your-ingress-class> \
  --set ahnlich-ai.ingress.host=ai.example.com
```

:::warning Plaintext gRPC over Ingress does not work — use TLS/h2c, or prefer Gateway API
Ahnlich speaks gRPC (HTTP/2). Through a typical Ingress controller without h2c/TLS the
controller answers HTTP/1.1 and the client fails with an `invalid compression flag` /
`500` error. For real use, enable TLS (`ahnlich-ai.ingress.tls.enabled=true`, with a cert
via cert-manager or a Secret) and set the controller's gRPC backend hint — for
nginx-ingress that is `nginx.ingress.kubernetes.io/backend-protocol: GRPC`. The
**Gateway API path above is the more reliable choice for gRPC.** `ingress` and `gateway`
are mutually exclusive.
:::

## Tracing

Enable the bundled in-cluster Jaeger backend and send both services' traces to it:

```bash
helm upgrade ahnlich charts/ahnlich -n ahnlich --reuse-values \
  --set tracing.enabled=true \
  --set tracing.backend.enabled=true \
  --set ahnlich-db.tracing.enabled=true \
  --set ahnlich-ai.tracing.enabled=true

# open the Jaeger UI
kubectl port-forward -n ahnlich svc/ahnlich-tracing-backend 16686:16686
```

:::note Bundled backend is dev/demo only
The bundled Jaeger all-in-one runs as a single replica with in-memory span storage and
no auth. For production, set `tracing.backend.enabled=false` and point
`ahnlich-db.tracing.otelEndpoint` / `ahnlich-ai.tracing.otelEndpoint` at your own OTLP
collector (Tempo, Honeycomb, an OpenTelemetry Collector, etc.).
:::

## Installing the Sub-Charts Individually

You don't have to use the umbrella. Install either service on its own — useful when
`ahnlich-db` and `ahnlich-ai` live in different namespaces or you manage them separately.

```bash
# DB only
helm install my-db charts/ahnlich-db -n ahnlich

# AI only — db.host is required and must resolve to a reachable DB Service
helm install my-ai charts/ahnlich-ai -n ahnlich \
  --set db.host=my-db \
  --set 'models.supported={all-minilm-l6-v2}'
```

With the standalone charts there is no enforced release name; the AI's `db.host` is
whatever you point it at.

## Operations

**Upgrade** — change values and re-apply:

```bash
helm upgrade ahnlich charts/ahnlich -n ahnlich --reuse-values --set ahnlich-db.persistence.size=50Gi
```

**Uninstall** — and clean up the PVCs (they are retained by default, holding the cached
models and persisted data):

```bash
helm uninstall ahnlich -n ahnlich
kubectl delete pvc -n ahnlich -l app.kubernetes.io/instance=ahnlich
```

Operational caveats:

- **Toggling `persistence.enabled` between upgrades fails.** A StatefulSet's
  `volumeClaimTemplates` is immutable. Flipping persistence on or off requires
  `helm uninstall` then a fresh install (then clean up orphaned PVCs as above).
- **`:latest` images pull `Always`.** When an image tag is `latest`, the chart sets
  `imagePullPolicy: Always` so `helm upgrade` actually pulls the newest digest. Pin a
  real tag (`--set ahnlich-db.image.tag=<version>`) for reproducible deploys; tagged
  images default to `IfNotPresent`.

## Cluster (Raft) Mode — In Development

The charts already carry the wiring for multi-replica Raft clusters — a headless
Service, per-replica RocksDB volumes, a PodDisruptionBudget, and bootstrap/join logic —
enabled like this:

```bash
# NOT yet functional — shown for reference only
helm install ahnlich charts/ahnlich -n ahnlich \
  --set ahnlich-db.cluster.enabled=true \
  --set ahnlich-ai.cluster.enabled=true \
  --set 'ahnlich-ai.models.supported={all-minilm-l6-v2}'
```

:::warning Not ready for use
Cluster/Raft mode is still in development. The server binary does not yet accept the
`--cluster-*` flags the chart passes, so enabling it will not produce a working cluster.
It is also excluded from the project's automated tests. Run the standalone (default)
mode for now and track the repository for cluster-mode availability.
:::

## Troubleshooting

- **AI pod stuck in `Init:0/1` (`wait-for-db`).** The init container blocks until the DB
  Service port is reachable. Check the DB pod is running and that `ahnlich-ai.db.host`
  resolves to the DB Service (`<release>-ahnlich-db` under the umbrella). Inspect with
  `kubectl logs -n ahnlich ahnlich-ahnlich-ai-0 -c wait-for-db`.
- **`helm install` fails with a release-name error.** You installed under a name other
  than `ahnlich` without overriding `ahnlich-ai.db.host`. Use the override the error
  message prints, or name the release `ahnlich`.
- **AI takes a long time to become ready on first start.** It is downloading models.
  Restrict `ahnlich-ai.models.supported` to only what you need, and give the model cache
  PVC (`ahnlich-ai.models.cache.size`) enough room. The cache persists across restarts.
- **AI pod stays `Running` but not `Ready` for a while on first install.** Expected: a
  `startupProbe` holds liveness and readiness until the embedding model has downloaded and
  the server answers a `PING`, so the pod is **not** killed mid-download. The default
  tolerance is 5 minutes (`ahnlich-ai.probes.startup.periodSeconds` ×
  `failureThreshold`). If a large model or a slow network needs longer, raise
  `ahnlich-ai.probes.startup.failureThreshold` — otherwise the pod restarts once the window
  is exceeded. The model cache persists on the PVC, so later restarts are fast.
- **Inspect a wedged install:**

  ```bash
  kubectl get pods,svc,pvc,events -n ahnlich
  kubectl logs -n ahnlich sts/ahnlich-ahnlich-db --tail 200
  kubectl logs -n ahnlich sts/ahnlich-ahnlich-ai --tail 200
  ```

## References

- [Ahnlich GitHub](https://github.com/deven96/ahnlich)
- [Chart source (`charts/`)](https://github.com/deven96/ahnlich/tree/main/charts)
- [Docker deployment](./deployment)
- [Distributed tracing](./tracing)
- [Client libraries](/docs/client-libraries)
