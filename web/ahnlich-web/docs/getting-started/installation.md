---
title: 📦 Installation
sidebar_position: 10
description: Install Ahnlich with Docker, pre-built binaries, or from source.
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# Installation

Ahnlich ships two services — the **DB engine** (`ahnlich-db`) and the **AI proxy**
(`ahnlich-ai`) — plus an interactive **CLI** (`ahnlich-cli`). Pick the install
method that fits your environment; the tabs below stay in sync across the page.

<Tabs groupId="install-method" queryString>
<TabItem value="docker" label="🐳 Docker" default>

Best for isolated sandboxes, CI, and quick testing — the only dependency is Docker.

Pull the latest official images:

```bash
docker pull ghcr.io/deven96/ahnlich-db:latest
docker pull ghcr.io/deven96/ahnlich-ai:latest
```

Run both services with their default ports (DB → `1369`, AI → `1370`):

```bash
docker run -d --name ahnlich-db -p 1369:1369 ghcr.io/deven96/ahnlich-db:latest
docker run -d --name ahnlich-ai -p 1370:1370 ghcr.io/deven96/ahnlich-ai:latest
```

:::tip
For advanced setups — tracing, persistence, and model caching — use the example
[`docker-compose.yml`](https://github.com/deven96/ahnlich/blob/main/docker-compose.yml)
in the main repository.
:::

</TabItem>
<TabItem value="binaries" label="📦 Pre-built binaries">

Great for servers without Rust or Docker. Download OS-specific binaries for
`db`, `ai`, and `cli` from the
[GitHub Releases page](https://github.com/deven96/ahnlich/releases).

Example for a Linux (`x86_64-unknown-linux-gnu`) environment — download and
extract all three binaries:

```bash
# Download the db, ai, and cli binaries for your version/platform
wget -L https://github.com/deven96/ahnlich/releases/download/bin%2Fdb%2F0.1.0/x86_64-unknown-linux-gnu-ahnlich-db.tar.gz
wget -L https://github.com/deven96/ahnlich/releases/download/bin%2Fai%2F0.1.0/x86_64-unknown-linux-gnu-ahnlich-ai.tar.gz
wget -L https://github.com/deven96/ahnlich/releases/download/bin%2Fcli%2F0.1.0/x86_64-unknown-linux-gnu-ahnlich-cli.tar.gz

# Extract each archive
tar -xvzf x86_64-unknown-linux-gnu-ahnlich-db.tar.gz
tar -xvzf x86_64-unknown-linux-gnu-ahnlich-ai.tar.gz
tar -xvzf x86_64-unknown-linux-gnu-ahnlich-cli.tar.gz

# Make the binaries executable
chmod +x ahnlich-db ahnlich-ai ahnlich-cli
```

Then start the services (DB → `1369`, AI → `1370`):

```bash
# Start the database engine
./ahnlich-db run --port 1369

# Start the AI proxy, pointing it at the DB
./ahnlich-ai run --db-host 127.0.0.1 --port 1370 --supported-models all-minilm-l6-v2

# Add --help to any binary to see all flags
./ahnlich-ai --help
```

:::note
Windows and macOS builds are available too — see the
[repository README](https://github.com/deven96/ahnlich/blob/main/README.md) for
platform-specific download instructions.
:::

</TabItem>
<TabItem value="source" label="🦀 From source">

For developers and Rust contributors who want to review code, customize
defaults, or stay on the cutting edge. Requires the Rust toolchain via
[rustup.rs](https://rustup.rs/).

```bash
git clone https://github.com/deven96/ahnlich.git
cd ahnlich

cargo build --release --bin ahnlich-db
cargo build --release --bin ahnlich-ai
cargo build --release --bin ahnlich-cli
```

The executables land in `target/release/`. Move them into your `$PATH` or run
them directly:

```bash
./target/release/ahnlich-db --help
./target/release/ahnlich-ai --help
./target/release/ahnlich-cli --help
```

</TabItem>
</Tabs>

## Which method should I use?

| Method | External dependencies | Best for | Upgrade workflow |
| --- | --- | --- | --- |
| **Docker** | Docker only | Isolated sandbox, CI, testing | `docker pull ghcr.io/deven96/ahnlich-*` |
| **Official binaries** | None (just a download tool) | Servers without Rust or Docker | Download updated files from Releases |
| **Source (Cargo)** | Rust toolchain | Custom builds, contributions | `git pull && cargo build` |

## Post-installation checklist

<div className="ahn-checklist">

- **Permissions** — for binaries, add execution rights with `chmod +x <binary>`.
- **Ports** — `ahnlich-db` defaults to `1369`, `ahnlich-ai` to `1370`. Override with the `--host` and `--port` flags.
- **Upgrades** — keep each install method up to date:
  - **Docker** — pull the `:latest` tag.
  - **Binaries** — download again from the Releases page.
  - **Source** — run `git pull && cargo build`.

</div>

:::info Next step
With the services running, head to the [**Quickstart**](./quickstart) to create
your first store and run a semantic search.
:::

## Uninstalling

<Tabs groupId="install-method" queryString>
<TabItem value="docker" label="🐳 Docker" default>

```bash
docker compose down -v
docker rmi ghcr.io/deven96/ahnlich-ai:latest ghcr.io/deven96/ahnlich-db:latest
```

</TabItem>
<TabItem value="binaries" label="📦 Pre-built binaries">

```bash
rm -f ahnlich-db ahnlich-ai ahnlich-cli
```

</TabItem>
<TabItem value="source" label="🦀 From source">

```bash
cargo clean          # remove build artifacts
rm -rf target/release/ahnlich-*
```

</TabItem>
</Tabs>

## Helpful links

- 🏠 [Main repository & documentation](https://github.com/deven96/ahnlich)
- 📦 [Releases page for downloading binaries](https://github.com/deven96/ahnlich/releases)
- 🧾 [Example Docker Compose file](https://github.com/deven96/ahnlich/blob/main/docker-compose.yml)
- 📖 [Full README (installation & usage guidance)](https://github.com/deven96/ahnlich/blob/main/README.md)
