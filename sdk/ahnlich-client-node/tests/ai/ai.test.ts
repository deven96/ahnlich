import { createAiClient } from "../../src/ai.js";
import { CreateStore, Ping, ListStores } from "../../grpc/ai/query_pb.js";
import { AIModel } from "../../grpc/ai/models_pb.js";
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
});
