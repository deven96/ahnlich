# Releasing Ahnlich

Every artifact releases the same way: **bump its version, merge, and CI (or one manual step) publishes it.** You can never re-release an existing version — always bump to a new one.

## Surfaces

| Artifact | Version file | Publishes to | Trigger |
|---|---|---|---|
| `ahnlich_types` + `ahnlich_client_rs` | `ahnlich/client/Cargo.toml` (+ `ahnlich/types/Cargo.toml`) | crates.io | PR merge |
| `ahnlich-client-py` | `sdk/ahnlich-client-py/VERSION` | PyPI | PR merge |
| `ahnlich-client-node` | `sdk/ahnlich-client-node/package.json` | npm | PR merge |
| `ahnlich-db` / `ahnlich-ai` / `ahnlich-cli` | `ahnlich/<db\|ai\|cli>/Cargo.toml` | GitHub Releases + ghcr (db/ai) | manual GitHub Release |
| Helm charts | `charts/<name>/Chart.yaml` | `oci://ghcr.io/deven96/charts` → ArtifactHub | PR merge |

## Bump rules

New feature → **minor**. Bug fix → **patch**. Breaking change → **major**.

A **wire-protocol change** (`ahnlich/types` or `protos/`) is lockstep: bump `types`, `client-rs`, `db`, `ai`, and `python` together, plus `node` if its generated code changed. `types` and `client-rs` publish together to crates.io, so both must bump.

## Release a client (rust / python / node)

1. Bump the version file.
2. Open a PR and merge to `main`.
3. CI tags and publishes automatically. Confirm the new version on the registry.

## Release a binary (db / ai / cli)

1. Bump `ahnlich/<name>/Cargo.toml`, then merge. This keeps the binary's self-reported `--version` correct; it does not create a tag.
2. Create a GitHub Release with tag `bin/<name>/<version>` and hand-written notes.
3. `release.yml` builds the binaries and pushes Docker images (db and ai only; cli has no image).

## Release a Helm chart

1. Bump `charts/<name>/Chart.yaml` `version`, then merge.
2. `charts-release.yml` packages and pushes the changed charts to `oci://ghcr.io/deven96/charts`.
3. ArtifactHub reindexes automatically.

First-time setup only: run the **Publish Helm charts** workflow manually, set the three ghcr packages to Public, and register `oci://ghcr.io/deven96/charts` on artifacthub.io.

## Gotchas

- Clients publish only when the watched version line changes in a **merged** PR.
- Binaries publish only from a **GitHub Release**, never a bare tag.
- ghcr package pages always show the repo README — per-image READMEs are a GitHub limitation, not a bug.

## Current versions

`types` / `client-rs` / `client-py` / `client-node` 0.4.0 · `db` 0.3.0 · `ai` 0.4.0 · `cli` 0.3.0 · charts 0.1.0
