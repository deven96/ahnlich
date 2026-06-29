---
title: 🔨 Usage
sidebar_position: 20
---

# Usage

This section covers how to **install, run, and interact** with Ahnlich. You’ll learn how to start the database and AI proxy, create stores, insert/query data, and manage lifecycle operations.

## Quickstart: Run Ahnlich
The fastest way to get started is by using the CLI with the binaries.

#### 1. Start the Database
Download and extract the latest binary from [GitHub Releases](https://github.com/deven96/ahnlich/releases).

Run the database:
```bash
./ahnlich-db run --port 1369
```

#### 2. Start the AI Proxy
In another terminal, run the AI proxy:
```bash
./ahnlich-ai run --db-host 127.0.0.1 --port 1370 \
  --supported-models all-minilm-l6-v2
```

This connects the AI proxy to the database and makes embedding models available.

#### 3. Create a Store

<!--Create a store with an index and query model:-->
In another terminal, run the command to open the ahnlich-cli interactive shell:
```bash
./ahnlich-cli ahnlich --agent ai --host 127.0.0.1 --port 1370
```

Create a store with an index and query model:
```bash
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
```

#### 4. Insert Data
Insert text with metadata:
```bash
SET (([This is the life of Haks paragraphed], {name: Haks, category: dev}), ([This is the life of Deven paragraphed], {name: Deven, category: dev})) in my_store preprocessaction nopreprocessing
```

#### 5. Query Data
Search for the two closest matches:
```bash
GETSIMN 2 WITH [life of deven paragraphed] USING cosinesimilarity IN my_store WHERE (category = dev)
```
You now have a running Ahnlich instance with similarity search working end-to-end.

## Querying the Database
The CLI provides a **declarative command style** for database operations.
 Commands generally follow this format:
 ```bash
<COMMAND> <ARGS> IN <STORE>
```

### Example DB Commands
#### Create a Store
```bash
CREATESTORE test_store DIMENSION 2 PREDICATES (author, country)
```

#### Insert Data
```bash
SET (([1.0, 2.1], {name: Haks, category: dev}), ([3.1, 4.8], {name: Deven, category: dev})) IN test_store
```

#### Retrieve Data
```bash
GETKEY ([1.0, 2.0], [3.0, 4.0]) IN test_store
```

#### Combine Multiple Commands
```bash
GETKEY ([1.0, 2.0], [3.0, 4.0]) IN test_store; CREATEPREDINDEX (name, category) IN test_store
```

### Supported DB Commands
`PING` – check if the server is responsive


`LISTCLIENTS` – list active connections


`LISTSTORES [SCHEMA <schema>]` – list stores in a schema; defaults to `public`


`INFOSERVER` – get server metadata/version


`DROPSTORE store_name IF EXISTS` – delete a store


`CREATEPREDINDEX (key_1, key_2) IN store_name` – create predicate index


`GETSIMN n WITH [vector] USING cosinesimilarity IN store_name WHERE (predicate)` – query nearest neighbors


`SET (...) IN store_name` – insert one or more vectors


…and more as the CLI evolves


## Querying the AI Proxy
When running the AI proxy alongside the DB, you can issue AI-aware commands that automatically embed text/images before storage.

### Example AI Commands
#### Create an AI Store
```bash
CREATESTORE my_store QUERYMODEL resnet-50 INDEXMODEL resnet-50 PREDICATES (author, country) NONLINEARALGORITHMINDEX (kdtree)
```

#### Insert AI Data
```bash
SET (([This is the life of Haks], {name: Haks, category: dev}), ([This is the life of Deven], {name: Deven, category: dev})) IN my_store PREPROCESSACTION nopreprocessing
```

#### Query AI Data
```bash
GETSIMN 4 WITH [random text] USING cosinesimilarity IN store_name WHERE (author = dickens)
```

Here, the `input string is automatically embedded` by the AI proxy before being compared against the stored vectors.

### Supported AI Commands
`PING` – check if the server is responsive


`LISTSTORES` – list all stores


`INFOSERVER` – get server metadata/version


`DROPSTORE store_name IF EXISTS` – delete a store


`CREATEPREDINDEX (key_1, key_2) IN store_name` – create predicate index


`GETSIMN n WITH [raw input] USING cosinesimilarity IN store_name WHERE (predicate)` – query nearest neighbors


`SET (...) IN store_name PREPROCESSACTION nopreprocessing` – insert one or more AI inputs


…and more as the CLI evolves

## How to Uninstall Ahnlich
Depending on your setup, uninstall is straightforward.
#### If Installed via Binaries
```bash
rm -rf ./ahnlich-db ./ahnlich-ai ./ahnlich-cli
```

#### If Installed via Docker
```bash
docker compose down -v
docker rmi ghcr.io/deven96/ahnlich-ai:latest \
           ghcr.io/deven96/ahnlich-db:latest
```
