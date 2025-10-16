---
title: Persistence for Ahnlich DB
sidebar_position: 30
---


# Persistence for Ahnlich DB
### Why need persistence?
Ahnlich DB manages structured data like keys, values, and predicates. Running it in-memory only means:
- Stored data disappears after restarts.

- Predicate indexes must be rebuilt manually.

- Any production state is volatile.

Enabling persistence ensures:

- Data and indexes survive restarts or container upgrades.

- Your stores can be backed up and restored easily.

- Mission-critical applications don’t lose their data.

### Configuring persistence from in-memory mode to disk

Enable persistence by starting the DB service with persistence flags:
```
ahnlich_db:
  image: ghcr.io/deven96/ahnlich-db:latest
  command: >
    "ahnlich-db run --host 0.0.0.0 \
    --enable-persistence --persist-location /root/.ahnlich/data/db.dat \
    --persistence-interval 300"
  ports:
    - "1369:1369"
  volumes:
    - "./data/:/root/.ahnlich/data" # Persistence Location
```

- `--enable-persistence` → turns on persistence.


- `--persist-location` → file path to store snapshots (e.g., `db.dat`).


- `--persistence-interval` → snapshot interval in seconds (default: 300).


- `volumes` → ensures snapshots persist outside container lifecycle.


### Loading from persistence

When restarted, the DB automatically loads the latest snapshot from `db.dat`.
- All stores, keys, and predicates return to their previous state.

- No manual re-indexing is required.

- You can back up or restore DB state simply by copying the persistence file.
