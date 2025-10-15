---
title: Installation
sidebar_position: 10
---

# Installation

You can use the Ahnlich CLI either by downloading **prebuilt binaries** or by building from **source**.

## Download Binaries

Prebuilt binaries are available on GitHub Releases: https://github.com/deven96/ahnlich/releases

### Extract the Archive

#### extract the downloaded archive
```
tar -xvzf <archive-name>.tar.gz
```

#### move the binary to a directory in your PATH (optional)
```
sudo mv ahnlich-db /usr/local/bin/
```

#### verify installation
```
ahnlich-db --version
```


#### Example: Run CLI against DB agent
```
./ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369
```

### Run from Source (Development Mode)

Clone the repo and run from the **workspace root**:

#### Clone the repo
```
git clone https://github.com/deven96/ahnlich.git

cd ahnlich
```

#### Run the Database Server
```
cargo run -p db --bin ahnlich-db   # Starts the DB server
```

#### Run the AI Server
```
cargo run -p ai --bin ahnlich-ai   # Starts the AI server
```

#### Run the CLI
```
cargo run -p cli --bin ahnlich-cli -- ahnlich --agent db --host 127.0.0.1 --port 1369
```
Replace `db` with `ai` to connect to the AI server

### Running the CLI
General command format:
```
ahnlich-cli ahnlich --agent <agent> --host <host> --port <port>
```

- `agent` → `ai` or `db`

- `host` → defaults to `127.0.0.1`

- `port` → defaults: `1370` (AI), `1369` (DB)



#### Example Usage

##### Connect to DB Agent
```
ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369
```

##### Connect to AI Agent
```
ahnlich-cli ahnlich --agent ai --host 127.0.0.1 --port 1370
```

