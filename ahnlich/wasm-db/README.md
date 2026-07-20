# Ahnlich WASM DB

Vector database for browsers and Node.js via WebAssembly.

## Install

```bash
npm install @deven96/ahnlich-wasm-db @deven96/ahnlich-client-node
```

## Usage

```typescript
import init, { AhnlichDB } from '@deven96/ahnlich-wasm-db';
import { CreateStore, Set, GetSimN } from '@deven96/ahnlich-client-node/dist/grpc/db/query_pb.js';
import { StoreKey, DbStoreEntry, StoreValue } from '@deven96/ahnlich-client-node/dist/grpc/keyval_pb.js';

// Initialize WASM
await init();

// Create database
const db = new AhnlichDB();

// Create a vector store
const createReq = new CreateStore({
    store: 'embeddings',
    dimension: 384,
    createPredicates: ['category'],
    errorIfExists: false
});
db.create_store(createReq.toBinary());

// Insert vectors
const setReq = new Set({
    store: 'embeddings',
    inputs: [
        new DbStoreEntry({
            key: new StoreKey({ key: [0.1, 0.2, 0.3, /* ... 381 more */] }),
            value: new StoreValue({ value: {} })
        })
    ]
});
db.set(setReq.toBinary());

// Find similar vectors
const searchReq = new GetSimN({
    store: 'embeddings',
    searchInput: new StoreKey({ key: [0.1, 0.2, 0.3, /* ... */] }),
    closestN: BigInt(10)
});
const results = GetSimNResponse.fromBinary(db.get_sim_n(searchReq.toBinary()));
```

## API

All methods take/return protobuf bytes:

- `create_store` - Create vector store
- `set` - Insert/update vectors
- `get_key` - Retrieve by exact key
- `get_sim_n` - Find N most similar vectors
- `get_pred` - Query by metadata predicate
- `list_stores` - List all stores
- `drop_store` - Delete a store
- `del_key` - Delete specific keys

See [examples/](./examples/)

## Performance

Fast enough for browser use. SIMD optimizations included.
