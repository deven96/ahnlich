import { ConnectError, Code } from "@connectrpc/connect";
import { createDbClient } from "../../src/db.js";
import { Ping } from "../../grpc/db/query_pb.js";
import { generateTlsCert, writeAuthConfig, spawnDbWithAuth } from "../utils.js";

const USERS = { alice: "alicepass1", bob: "bobspass1" };

async function expectUnauthenticated(promise: Promise<unknown>): Promise<void> {
  await expect(promise).rejects.toThrow(ConnectError);
  await promise.catch((e: unknown) => {
    expect((e as ConnectError).code).toBe(Code.Unauthenticated);
  });
}

describe("DB auth", () => {
  let address: string;
  let kill: () => void;
  let certPem: Buffer;
  let cleanupTls: () => void;
  let cleanupAuth: () => void;

  beforeAll(async () => {
    const tls = generateTlsCert();
    certPem = tls.certPem;
    cleanupTls = tls.cleanup;

    const auth = writeAuthConfig(USERS);
    cleanupAuth = auth.cleanup;

    const proc = await spawnDbWithAuth(tls.certPath, tls.keyPath, auth.authConfigPath, certPem);
    address = proc.address;
    kill = proc.kill;
  }, 30_000);

  afterAll(() => {
    kill();
    cleanupTls();
    cleanupAuth();
  });

  test("unauthenticated request is rejected", async () => {
    const client = createDbClient(address, { caCert: certPem });
    await expectUnauthenticated(client.ping(new Ping()));
  });

  test("wrong credentials are rejected", async () => {
    const client = createDbClient(address, {
      caCert: certPem,
      auth: { username: "alice", apiKey: "wrongpassword" },
    });
    await expectUnauthenticated(client.ping(new Ping()));
  });

  test("valid credentials are accepted", async () => {
    const client = createDbClient(address, {
      caCert: certPem,
      auth: { username: "alice", apiKey: "alicepass1" },
    });
    await expect(client.ping(new Ping())).resolves.toBeDefined();
  });

  test("multiple users are independently authenticated", async () => {
    const alice = createDbClient(address, {
      caCert: certPem,
      auth: { username: "alice", apiKey: "alicepass1" },
    });
    const bob = createDbClient(address, {
      caCert: certPem,
      auth: { username: "bob", apiKey: "bobspass1" },
    });
    const charlie = createDbClient(address, {
      caCert: certPem,
      auth: { username: "charlie", apiKey: "charliepass" },
    });

    await expect(alice.ping(new Ping())).resolves.toBeDefined();
    await expect(bob.ping(new Ping())).resolves.toBeDefined();
    await expectUnauthenticated(charlie.ping(new Ping()));
  });
});
