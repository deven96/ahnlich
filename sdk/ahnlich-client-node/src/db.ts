import { createPromiseClient, PromiseClient } from "@connectrpc/connect";
import { createGrpcTransport } from "@connectrpc/connect-node";
import { DBService } from "../grpc/services/db_service_connect.js";
import { type AhnlichClientOptions, buildInterceptors } from "./client-options.js";

export type DbClient = PromiseClient<typeof DBService>;

export function createDbClient(address: string, opts: AhnlichClientOptions = {}): DbClient {
  const transport = createGrpcTransport({
    baseUrl: opts.caCert ? `https://${address}` : `http://${address}`,
    httpVersion: "2",
    nodeOptions: opts.caCert ? { ca: opts.caCert } : undefined,
    interceptors: buildInterceptors(opts),
  });
  return createPromiseClient(DBService, transport);
}

export { DBService };
