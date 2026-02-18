import { createDbClient } from "../../src/db.js";
import { CreateStore, Set, GetSimN, ListStores, Ping } from "../../grpc/db/query_pb.js";
import { StoreKey, StoreValue, DbStoreEntry } from "../../grpc/keyval_pb.js";
import { MetadataValue } from "../../grpc/metadata_pb.js";
import { Algorithm } from "../../grpc/algorithm/algorithm_pb.js";
import { spawnDb } from "../utils.js";

describe("DB client", () => {
  let address: string;
  let kill: () => void;

  beforeAll(async () => {
    const proc = await spawnDb();
    address = proc.address;
    kill = proc.kill;
  }, 60_000);

  afterAll(() => kill());

  test("ping returns pong", async () => {
    const client = createDbClient(address);
    const resp = await client.ping(new Ping());
    expect(resp).toBeDefined();
  });

  test("create store and list stores", async () => {
    const client = createDbClient(address);
    const storeName = "test_store";

    await client.createStore(
      new CreateStore({ store: storeName, dimension: 3, errorIfExists: false }),
    );

    const resp = await client.listStores(new ListStores());
    const names = resp.stores.map((s) => s.name);
    expect(names).toContain(storeName);
  });

  test("set and get_sim_n returns nearest neighbour", async () => {
    const client = createDbClient(address);
    const storeName = "sim_store";

    await client.createStore(
      new CreateStore({ store: storeName, dimension: 3, errorIfExists: false }),
    );

    const entries: DbStoreEntry[] = [
      new DbStoreEntry({
        key: new StoreKey({ key: [1.0, 0.0, 0.0] }),
        value: new StoreValue({
          value: {
            label: new MetadataValue({ value: { case: "rawString", value: "x-axis" } }),
          },
        }),
      }),
      new DbStoreEntry({
        key: new StoreKey({ key: [0.0, 1.0, 0.0] }),
        value: new StoreValue({
          value: {
            label: new MetadataValue({ value: { case: "rawString", value: "y-axis" } }),
          },
        }),
      }),
      new DbStoreEntry({
        key: new StoreKey({ key: [0.0, 0.0, 1.0] }),
        value: new StoreValue({
          value: {
            label: new MetadataValue({ value: { case: "rawString", value: "z-axis" } }),
          },
        }),
      }),
    ];

    await client.set(new Set({ store: storeName, inputs: entries }));

    const resp = await client.getSimN(
      new GetSimN({
        store: storeName,
        searchInput: new StoreKey({ key: [0.9, 0.1, 0.0] }),
        closestN: BigInt(1),
        algorithm: Algorithm.CosineSimilarity,
      }),
    );

    expect(resp.entries.length).toBe(1);
    const topLabel = resp.entries[0].value?.value["label"];
    expect(topLabel?.value.case).toBe("rawString");
    expect(topLabel?.value.value).toBe("x-axis");
  });
});
