---
title: Installation
sidebar_position: 10
---

# Installation

You can use the Ahnlich CLI either by downloading **prebuilt binaries** or by building from **source**.

## Download Binaries

Prebuilt binaries are available on GitHub Releases: https://github.com/deven96/ahnlich/releases

### Extract the Archive

#### Extract the downloaded archive
```bash
tar -xvzf <archive-name>.tar.gz
```

#### Move the binary to a directory in your PATH (optional)
```bash
sudo mv ahnlich-cli /usr/local/bin/
```

#### Verify installation
```bash
ahnlich-cli --version
```


#### Example: Run CLI against DB server

First run the DB server:

```bash
ahnlich-db run
```

Then run the CLI against the DB server:

```bash
ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369
```

### Run from Source (Development Mode)

Clone the repo and run from the **workspace root**:

#### Clone the repo
```bash
git clone https://github.com/deven96/ahnlich.git

cd ahnlich
```

#### Run the DB Server
```bash
cargo run -p db --bin ahnlich-db run
```

#### Run the AI Server
```bash
cargo run -p ai --bin ahnlich-ai run
```

#### Run the CLI
```bash
cargo run -p cli --bin ahnlich-cli -- ahnlich --agent db --host 127.0.0.1 --port 1369
```
Replace `db` with `ai` to connect to the AI server

### Running the CLI
General command format:
```bash
ahnlich-cli ahnlich --agent <agent> --host <host> --port <port>
```

- `agent` → `ai` or `db`

- `host` → defaults to `127.0.0.1`

- `port` → defaults: `1370` (AI), `1369` (DB)



#### Example Usage

##### Connect to DB Agent
```bash
ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369
```

##### Connect to AI Agent
```bash
ahnlich-cli ahnlich --agent ai --host 127.0.0.1 --port 1370
```
