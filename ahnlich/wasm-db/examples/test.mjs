import { test } from 'node:test';
import assert from 'node:assert';
import { readFile } from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

import { CreateStore, Set as SetRequest, GetKey, GetSimN as GetSimNRequest, GetPred as GetPredRequest, ListStores, DropStore } from '../../../sdk/ahnlich-client-node/dist/grpc/db/query_pb.js';
import { Set as SetResponse, Get, GetSimN as GetSimNResponse, StoreList, Unit } from '../../../sdk/ahnlich-client-node/dist/grpc/db/server_pb.js';
import { StoreKey, StoreValue, DbStoreEntry } from '../../../sdk/ahnlich-client-node/dist/grpc/keyval_pb.js';
import { MetadataValue } from '../../../sdk/ahnlich-client-node/dist/grpc/metadata_pb.js';
import { PredicateCondition, Predicate, Equals } from '../../../sdk/ahnlich-client-node/dist/grpc/predicate_pb.js';

const { AhnlichDB } = await import('../pkg/ahnlich_wasm_db.js');

const db = new AhnlichDB();

function normalize(vec) {
    const magnitude = Math.sqrt(vec.reduce((sum, val) => sum + val * val, 0));
    return vec.map(val => val / magnitude);
}

test('List stores (empty)', () => {
    const req = new ListStores();
    const resp = StoreList.fromBinary(db.list_stores(req.toBinary()));
    assert.strictEqual(resp.stores.length, 0);
});

test('Create stores with different schemas', () => {
    const publicStore = new CreateStore({
        store: 'sentences',
        dimension: 384,
        createPredicates: ['category', 'language'],
        nonLinearIndices: [],
        errorIfExists: false,
        schema: 'public'
    });
    db.create_store(publicStore.toBinary());

    const visionStore = new CreateStore({
        store: 'images',
        dimension: 512,
        createPredicates: ['label'],
        nonLinearIndices: [],
        errorIfExists: false,
        schema: 'vision'
    });
    db.create_store(visionStore.toBinary());
});

test('Insert sentence embeddings with metadata', () => {
    const entries = [
        new DbStoreEntry({
            key: new StoreKey({ key: normalize([0.1, 0.5, 0.2, ...Array(381).fill(0.01)]) }),
            value: new StoreValue({ 
                value: { 
                    category: new MetadataValue({ value: { case: 'rawString', value: 'greeting' } }),
                    language: new MetadataValue({ value: { case: 'rawString', value: 'en' } }),
                    text: new MetadataValue({ value: { case: 'rawString', value: 'Hello, how are you?' } })
                } 
            })
        }),
        new DbStoreEntry({
            key: new StoreKey({ key: normalize([0.15, 0.48, 0.25, ...Array(381).fill(0.015)]) }),
            value: new StoreValue({ 
                value: { 
                    category: new MetadataValue({ value: { case: 'rawString', value: 'greeting' } }),
                    language: new MetadataValue({ value: { case: 'rawString', value: 'en' } }),
                    text: new MetadataValue({ value: { case: 'rawString', value: 'Hi there!' } })
                } 
            })
        }),
        new DbStoreEntry({
            key: new StoreKey({ key: normalize([0.8, 0.1, 0.3, ...Array(381).fill(0.005)]) }),
            value: new StoreValue({ 
                value: { 
                    category: new MetadataValue({ value: { case: 'rawString', value: 'weather' } }),
                    language: new MetadataValue({ value: { case: 'rawString', value: 'en' } }),
                    text: new MetadataValue({ value: { case: 'rawString', value: 'It is sunny today' } })
                } 
            })
        }),
        new DbStoreEntry({
            key: new StoreKey({ key: normalize([0.12, 0.52, 0.18, ...Array(381).fill(0.012)]) }),
            value: new StoreValue({ 
                value: { 
                    category: new MetadataValue({ value: { case: 'rawString', value: 'greeting' } }),
                    language: new MetadataValue({ value: { case: 'rawString', value: 'es' } }),
                    text: new MetadataValue({ value: { case: 'rawString', value: 'Hola, ¿cómo estás?' } })
                } 
            })
        })
    ];

    const req = new SetRequest({ store: 'sentences', inputs: entries, schema: 'public' });
    const resp = SetResponse.fromBinary(db.set(req.toBinary()));
    assert.strictEqual(Number(resp.upsert?.inserted), 4);
});

test('List all stores across schemas', () => {
    const req = new ListStores();
    const resp = StoreList.fromBinary(db.list_stores(req.toBinary()));
    assert.ok(resp.stores.length >= 1);
});

test('Filter by metadata predicate (category = greeting)', () => {
    const req = new GetPredRequest({
        store: 'sentences',
        condition: new PredicateCondition({
            kind: {
                case: 'value',
                value: new Predicate({
                    kind: {
                        case: 'equals',
                        value: new Equals({
                            key: 'category',
                            value: new MetadataValue({ value: { case: 'rawString', value: 'greeting' } })
                        })
                    }
                })
            }
        }),
        schema: 'public'
    });

    const resp = Get.fromBinary(db.get_pred(req.toBinary()));
    assert.strictEqual(resp.entries.length, 3);
});

test('Similarity search (top 3)', () => {
    const queryVec = normalize([0.1, 0.5, 0.2, ...Array(381).fill(0.01)]);
    const req = new GetSimNRequest({
        store: 'sentences',
        searchInput: new StoreKey({ key: queryVec }),
        closestN: BigInt(3),
        schema: 'public'
    });

    const resp = GetSimNResponse.fromBinary(db.get_sim_n(req.toBinary()));
    assert.strictEqual(resp.entries.length, 3);
});

test('Get by exact key', () => {
    const queryVec = normalize([0.1, 0.5, 0.2, ...Array(381).fill(0.01)]);
    const req = new GetKey({
        store: 'sentences',
        keys: [new StoreKey({ key: queryVec })],
        schema: 'public'
    });

    const resp = Get.fromBinary(db.get_key(req.toBinary()));
    assert.strictEqual(resp.entries.length, 1);
});

test('Drop store', () => {
    const req = new DropStore({ store: 'images', schema: 'vision', errorIfNotExists: true });
    db.drop_store(req.toBinary());

    const listReq = new ListStores();
    const listResp = StoreList.fromBinary(db.list_stores(listReq.toBinary()));
    assert.strictEqual(listResp.stores.length, 1);
});

test('Export and import snapshot', () => {
    const snapshot = db.export_snapshot();
    assert.ok(snapshot.length > 0);

    const db2 = new AhnlichDB();
    db2.import_snapshot(snapshot);

    const listReq = new ListStores();
    const listResp = StoreList.fromBinary(db2.list_stores(listReq.toBinary()));
    assert.strictEqual(listResp.stores.length, 1);
    assert.strictEqual(listResp.stores[0].name, 'sentences');
});
