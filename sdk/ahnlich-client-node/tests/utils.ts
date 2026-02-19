import { ConnectError, Code } from "@connectrpc/connect";
import * as crypto from "crypto";
import { execSync, spawn, ChildProcess } from "child_process";
import * as fs from "fs";
import * as net from "net";
import * as os from "os";
import * as path from "path";
import { createDbClient } from "../src/db.js";
import { createAiClient } from "../src/ai.js";
import { Ping as DbPing } from "../grpc/db/query_pb.js";
import { Ping as AiPing } from "../grpc/ai/query_pb.js";

const MAX_RETRIES = 240;
const RETRY_INTERVAL_MS = 1000;

export interface AhnlichProcess {
  host: string;
  port: number;
  address: string;
  kill: () => void;
}

export interface AuthSetup {
  certPath: string;
  keyPath: string;
  certPem: Buffer;
  authConfigPath: string;
  tmpDir: string;
  cleanup: () => void;
}

function ahnlichDir(): string {
  return path.resolve(__dirname, "..", "..", "..", "ahnlich");
}

function targetDir(): string {
  if (process.env.CARGO_TARGET_DIR) return process.env.CARGO_TARGET_DIR;
  const meta = execSync(
    `cargo metadata --format-version 1 --no-deps --manifest-path ${ahnlichDir()}/Cargo.toml`,
    { encoding: "utf8" },
  );
  return JSON.parse(meta).target_directory as string;
}

function binaryPath(name: "ahnlich-db" | "ahnlich-ai"): string {
  return path.join(targetDir(), "debug", name);
}

export async function getFreePort(): Promise<number> {
  return new Promise((resolve, reject) => {
    const server = net.createServer();
    server.listen(0, "127.0.0.1", () => {
      const port = (server.address() as net.AddressInfo).port;
      server.close(() => resolve(port));
    });
    server.on("error", reject);
  });
}

/**
 * Generate a self-signed P-256 TLS cert valid for 127.0.0.1 / localhost.
 * Writes cert and key to a temp directory (caller must call cleanup()).
 */
export function generateTlsCert(): {
  certPath: string;
  keyPath: string;
  certPem: Buffer;
  cleanup: () => void;
} {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ahnlich-tls-"));
  const keyPath = path.join(tmpDir, "server.key");
  const certPath = path.join(tmpDir, "server.crt");

  // Node's built-in crypto has no x509 cert creation with SANs before Node 24,
  // so we delegate to openssl (available on all CI runners).
  execSync(
    `openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:P-256` +
      ` -keyout "${keyPath}" -out "${certPath}"` +
      ` -days 1 -nodes -subj "/CN=localhost"` +
      ` -addext "subjectAltName=DNS:localhost,IP:127.0.0.1"`,
    { stdio: "ignore" },
  );

  const certPem = fs.readFileSync(certPath);
  return {
    certPath,
    keyPath,
    certPem,
    cleanup: () => fs.rmSync(tmpDir, { recursive: true }),
  };
}

/** Write an auth.toml with SHA-256-hashed API keys, matching the server's expected format. */
export function writeAuthConfig(users: Record<string, string>): {
  authConfigPath: string;
  tmpDir: string;
  cleanup: () => void;
} {
  const tmpDir = fs.mkdtempSync(path.join(os.tmpdir(), "ahnlich-auth-"));
  const lines = ["[users]"];
  for (const [username, apiKey] of Object.entries(users)) {
    const hash = crypto.createHash("sha256").update(apiKey).digest("hex");
    lines.push(`${username} = "${hash}"`);
  }
  lines.push("", "[security]", "min_key_length = 8", "");
  const authConfigPath = path.join(tmpDir, "auth.toml");
  fs.writeFileSync(authConfigPath, lines.join("\n"));
  return { authConfigPath, tmpDir, cleanup: () => fs.rmSync(tmpDir, { recursive: true }) };
}

async function waitForGrpcDb(address: string, proc: ChildProcess, caCert?: Buffer): Promise<void> {
  const client = createDbClient(address, caCert ? { caCert } : {});
  let procError: Error | null = null;
  proc.once("error", (e) => (procError = e));

  for (let i = 0; i < MAX_RETRIES; i++) {
    if (procError) throw new Error(`DB process error: ${procError.message}`);
    try {
      await client.ping(new DbPing());
      return;
    } catch (e) {
      // An Unauthenticated error means the server is up and enforcing auth — ready.
      if (e instanceof ConnectError && e.code === Code.Unauthenticated) return;
      await new Promise((r) => setTimeout(r, RETRY_INTERVAL_MS));
    }
  }
  throw new Error(`DB gRPC at ${address} did not become ready within ${MAX_RETRIES}s`);
}

async function waitForGrpcAi(address: string, proc: ChildProcess): Promise<void> {
  const client = createAiClient(address);
  let procError: Error | null = null;
  proc.once("error", (e) => (procError = e));

  for (let i = 0; i < MAX_RETRIES; i++) {
    if (procError) throw new Error(`AI process error: ${procError.message}`);
    try {
      await client.ping(new AiPing());
      console.log(`[waitForGrpcAi] ready after ${i + 1}s`);
      return;
    } catch (e) {
      if (i % 10 === 0) {
        console.log(`[waitForGrpcAi] attempt ${i + 1}/${MAX_RETRIES}: ${(e as Error).message}`);
      }
      await new Promise((r) => setTimeout(r, RETRY_INTERVAL_MS));
    }
  }
  throw new Error(`AI gRPC at ${address} did not become ready within ${MAX_RETRIES}s`);
}

function spawnBinary(
  name: "ahnlich-db" | "ahnlich-ai",
  args: string[],
  onError: (err: Error) => void,
): ChildProcess {
  const bin = binaryPath(name);
  const debugDir = path.dirname(bin);
  // libonnxruntime.so is placed next to the binary in the target/debug dir by ort-sys.
  // When spawning the binary directly (not via `cargo run`), we must set LD_LIBRARY_PATH
  // so the dynamic linker can find it on Linux CI runners.
  const env = {
    ...process.env,
    LD_LIBRARY_PATH: [debugDir, process.env.LD_LIBRARY_PATH].filter(Boolean).join(":"),
  };
  console.log(`[spawnBinary] launching ${bin} run ${args.join(" ")}`);
  const proc = spawn(bin, ["run", ...args], { stdio: "inherit", env });
  proc.on("error", (err) => {
    console.error(`[spawnBinary] ${name} process error: ${err.message}`);
    onError(err);
  });
  return proc;
}

export async function spawnDb(extraArgs: string[] = []): Promise<AhnlichProcess> {
  const port = await getFreePort();
  const host = "127.0.0.1";
  const address = `${host}:${port}`;
  let rejectOnError!: (e: Error) => void;
  const proc = spawnBinary("ahnlich-db", ["--port", String(port), ...extraArgs], (e) =>
    rejectOnError?.(e),
  );
  await waitForGrpcDb(address, proc);
  return { host, port, address, kill: () => proc.kill() };
}

export async function spawnDbWithAuth(
  certPath: string,
  keyPath: string,
  authConfigPath: string,
  caCert: Buffer,
): Promise<AhnlichProcess> {
  const port = await getFreePort();
  const host = "127.0.0.1";
  const address = `${host}:${port}`;
  const proc = spawnBinary(
    "ahnlich-db",
    [
      "--port",
      String(port),
      "--enable-auth",
      "--auth-config",
      authConfigPath,
      "--tls-cert",
      certPath,
      "--tls-key",
      keyPath,
    ],
    (e) => console.error(`[spawnDbWithAuth] process error: ${e.message}`),
  );
  await waitForGrpcDb(address, proc, caCert);
  return { host, port, address, kill: () => proc.kill() };
}

export async function spawnAi(dbPort: number, extraArgs: string[] = []): Promise<AhnlichProcess> {
  const port = await getFreePort();
  const host = "127.0.0.1";
  const address = `${host}:${port}`;
  const proc = spawnBinary(
    "ahnlich-ai",
    [
      "--port",
      String(port),
      "--db-port",
      String(dbPort),
      // Only load the one model used in tests — avoids downloading all 7 models on a cold cache
      "--supported-models",
      "all-minilm-l6-v2",
      ...extraArgs,
    ],
    (e) => console.error(`[spawnAi] process error: ${e.message}`),
  );
  await waitForGrpcAi(address, proc);
  return { host, port, address, kill: () => proc.kill() };
}
