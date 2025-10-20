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
```
./ahnlich-db run --port 1369
```

#### 2. Start the AI Proxy
In another terminal, run the AI proxy:
```
./ahnlich-ai run --db-host 127.0.0.1 --port 1370 \
  --supported-models all-minilm-l6-v2,resnet-50
```

This connects the AI proxy to the database and makes embedding models available.

#### 3. Create a Store
Create a store with an index and query model:
```
./ahnlich-cli create_store my_store \
  --index_model all-minilm-l6-v2 \
  --query_model all-minilm-l6-v2
```

#### 4. Insert Data
Insert vectors with metadata:
```
./ahnlich-cli insert my_store \
  --vector [0.1,0.2,0.3] \
  --metadata '{"page":"about"}'
```

#### 5. Query Data
Search for the two closest vectors:
```
./ahnlich-cli get_sim_n my_store \
  --input [0.2,0.1,0.3] \
  --n 2 \
  --algorithm cosinesimilarity \
  --where 'page != hidden'
```
You now have a running Ahnlich instance with similarity search working end-to-end.

## Querying the Database
The CLI also provides a **declarative command style** for database operations.
 Commands generally follow this format:
 ```
<COMMAND> <ARGS> IN <STORE>
```

### Example DB Commands
#### Create a Store
```
CREATESTORE test_store DIMENSION 2 PREDICATES (author, country)
```

#### Insert Data
```
SET (([1.0, 2.1], {name: Haks, category: dev}), 
     ([3.1, 4.8], {name: Deven, category: dev})) IN test_store
```

#### Retrieve Data
```
GETKEY ([1.0, 2.0], [3.0, 4.0]) IN test_store
```

#### Combine Multiple Commands
```
GETKEY ([1.0, 2.0], [3.0, 4.0]) IN test_store; 
CREATEPREDINDEX (name, category) IN test_store
```

### Supported DB Commands
`PING` – check if the server is responsive


`LISTCLIENTS` – list active connections


`LISTSTORES` – list all stores


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
```
CREATESTORE my_store 
  QUERYMODEL resnet-50 
  INDEXMODEL resnet-50 
  PREDICATES (author, country) 
  NONLINEARALGORITHMINDEX (kdtree)
```

#### Insert AI Data
```
SET (([This is the life of Haks], {name: Haks, category: dev}), 
     ([This is the life of Deven], {name: Deven, category: dev})) IN store
```

#### Query AI Data
```
GETSIMN 4 WITH [random text] USING cosinesimilarity IN store_name WHERE (author = dickens)
```

Here, the `input string is automatically embedded` by the AI proxy before being compared against the stored vectors.

## How to Uninstall Ahnlich
Depending on your setup, uninstall is straightforward.
#### If Installed via Binaries
```
rm -rf ./ahnlich-db ./ahnlich-ai ./ahnlich-cli
```

#### If Installed via Docker
```
docker compose down -v
docker rmi ghcr.io/deven96/ahnlich-ai:latest \
           ghcr.io/deven96/ahnlich-db:latest
```


