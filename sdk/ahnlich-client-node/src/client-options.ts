import type { Interceptor } from "@connectrpc/connect";

export const TRACE_HEADER = "ahnlich-trace-id";

export interface AhnlichClientOptions {
  /** Bearer auth credentials. Requires the server to be started with --enable-auth and TLS. */
  auth?: {
    username: string;
    apiKey: string;
  };
  /**
   * PEM-encoded CA certificate for TLS verification.
   * Required when connecting to a server with --enable-auth (which mandates TLS).
   */
  caCert?: Buffer;
  /**
   * W3C traceparent string to forward as the ahnlich-trace-id header,
   * enabling distributed tracing through the server.
   */
  traceId?: string;
}

export function buildInterceptors(opts: AhnlichClientOptions): Interceptor[] {
  const interceptors: Interceptor[] = [];

  if (opts.auth) {
    const { username, apiKey } = opts.auth;
    interceptors.push((next) => (req) => {
      req.header.set("authorization", `Bearer ${username}:${apiKey}`);
      return next(req);
    });
  }

  if (opts.traceId) {
    const traceId = opts.traceId;
    interceptors.push((next) => (req) => {
      req.header.set(TRACE_HEADER, traceId);
      return next(req);
    });
  }

  return interceptors;
}
