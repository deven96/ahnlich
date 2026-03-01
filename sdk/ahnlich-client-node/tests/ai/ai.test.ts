import { createAiClient } from "../../src/ai.js";
import {
  CreateStore,
  GetStore,
  Ping,
  ListStores,
  CreateNonLinearAlgorithmIndex,
  DropNonLinearAlgorithmIndex,
} from "../../grpc/ai/query_pb.js";
import { AIModel } from "../../grpc/ai/models_pb.js";
import {
  NonLinearIndex,
  NonLinearAlgorithm,
  KDTreeConfig,
  HNSWConfig,
} from "../../grpc/algorithm/nonlinear_pb.js";
import { spawnDb, spawnAi } from "../utils.js";

describe("AI client", () => {
  let address: string;
  let killDb: () => void;
  let killAi: () => void;

  beforeAll(async () => {
    const db = await spawnDb();
    killDb = db.kill;
    const ai = await spawnAi(db.port);
    address = ai.address;
    killAi = ai.kill;
  }, 300_000);

  afterAll(() => {
    killAi();
    killDb();
  });

  test("ping returns pong", async () => {
    const client = createAiClient(address);
    const resp = await client.ping(new Ping());
    expect(resp).toBeDefined();
  });

  test("create store and list stores", async () => {
    const client = createAiClient(address);
    const storeName = "ai_test_store";

    await client.createStore(
      new CreateStore({
        store: storeName,
        queryModel: AIModel.ALL_MINI_LM_L6_V2,
        indexModel: AIModel.ALL_MINI_LM_L6_V2,
        errorIfExists: false,
      }),
    );

    const resp = await client.listStores(new ListStores());
    const names = resp.stores.map((s) => s.name);
    expect(names).toContain(storeName);
  });

  test("get store returns AI store info", async () => {
    const client = createAiClient(address);
    const storeName = "ai_get_store_test";

    await client.createStore(
      new CreateStore({
        store: storeName,
        queryModel: AIModel.ALL_MINI_LM_L6_V2,
        indexModel: AIModel.ALL_MINI_LM_L6_V2,
        errorIfExists: false,
      }),
    );

    const resp = await client.getStore(new GetStore({ store: storeName }));
    expect(resp).toBeDefined();
    expect(resp.name).toBe(storeName);
    expect(resp.queryModel).toBe(AIModel.ALL_MINI_LM_L6_V2);
    expect(resp.indexModel).toBe(AIModel.ALL_MINI_LM_L6_V2);
    expect(Number(resp.embeddingSize)).toBe(0);
  });

  test("create and drop kdtree non-linear algorithm index", async () => {
    const client = createAiClient(address);
    const storeName = "ai_kdtree_index_store";

    await client.createStore(
      new CreateStore({
        store: storeName,
        queryModel: AIModel.ALL_MINI_LM_L6_V2,
        indexModel: AIModel.ALL_MINI_LM_L6_V2,
        errorIfExists: false,
      }),
    );

    // Create KDTree index
    const createResp = await client.createNonLinearAlgorithmIndex(
      new CreateNonLinearAlgorithmIndex({
        store: storeName,
        nonLinearIndices: [
          new NonLinearIndex({ index: { case: "kdtree", value: new KDTreeConfig() } }),
        ],
      }),
    );
    expect(createResp).toBeDefined();

    // Drop KDTree index
    const dropResp = await client.dropNonLinearAlgorithmIndex(
      new DropNonLinearAlgorithmIndex({
        store: storeName,
        nonLinearIndices: [NonLinearAlgorithm.KDTree],
        errorIfNotExists: true,
      }),
    );
    expect(dropResp).toBeDefined();
  });

  test("create and drop hnsw non-linear algorithm index", async () => {
    const client = createAiClient(address);
    const storeName = "ai_hnsw_index_store";

    await client.createStore(
      new CreateStore({
        store: storeName,
        queryModel: AIModel.ALL_MINI_LM_L6_V2,
        indexModel: AIModel.ALL_MINI_LM_L6_V2,
        errorIfExists: false,
      }),
    );

    // Create HNSW index with default config
    const createResp = await client.createNonLinearAlgorithmIndex(
      new CreateNonLinearAlgorithmIndex({
        store: storeName,
        nonLinearIndices: [
          new NonLinearIndex({ index: { case: "hnsw", value: new HNSWConfig() } }),
        ],
      }),
    );
    expect(createResp).toBeDefined();

    // Drop HNSW index
    const dropResp = await client.dropNonLinearAlgorithmIndex(
      new DropNonLinearAlgorithmIndex({
        store: storeName,
        nonLinearIndices: [NonLinearAlgorithm.HNSW],
        errorIfNotExists: true,
      }),
    );
    expect(dropResp).toBeDefined();
  });
});
