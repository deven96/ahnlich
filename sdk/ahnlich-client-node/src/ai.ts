import { createPromiseClient, PromiseClient } from "@connectrpc/connect";
import { createGrpcTransport } from "@connectrpc/connect-node";
import { AIService } from "../grpc/services/ai_service_connect.js";
import { type AhnlichClientOptions, buildInterceptors } from "./client-options.js";

export type AiClient = PromiseClient<typeof AIService>;

export function createAiClient(address: string, opts: AhnlichClientOptions = {}): AiClient {
  const transport = createGrpcTransport({
    baseUrl: opts.caCert ? `https://${address}` : `http://${address}`,
    httpVersion: "2",
    nodeOptions: opts.caCert ? { ca: opts.caCert } : undefined,
    interceptors: buildInterceptors(opts),
  });
  return createPromiseClient(AIService, transport);
}

export { AIService };
export * from "../grpc/ai/models_pb.js";
export * from "../grpc/ai/preprocess_pb.js";
