---
title: Tracing
sidebar_position: 5
---

# Tracing

The Node.js SDK supports distributed tracing using W3C Trace Context format. This allows you to correlate requests across services for debugging and monitoring.

## Setting Up Tracing

Pass a W3C trace ID when creating the client:

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";

const client = createDbClient("127.0.0.1:1369", {
  traceId: "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01",
});

// All requests from this client will include the trace ID
const response = await client.ping(new Ping());
```
</details>

## Trace ID Format

The trace ID follows the W3C Trace Context format:

```
version-trace_id-parent_id-trace_flags
```

Example: `00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01`

| Part | Description |
|------|-------------|
| `00` | Version (always `00` for W3C format) |
| `80e1afed08e019fc1110464cfa66635c` | 32-character hex trace ID |
| `7a085853722dc6d2` | 16-character hex parent span ID |
| `01` | Trace flags (01 = sampled) |

## Using with OpenTelemetry

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { trace, context } from "@opentelemetry/api";

async function tracedOperation() {
  const tracer = trace.getTracer("ahnlich-app");

  return tracer.startActiveSpan("db-operation", async (span) => {
    // Get the current trace context
    const spanContext = span.spanContext();
    const traceId = `00-${spanContext.traceId}-${spanContext.spanId}-01`;

    // Create client with trace ID
    const client = createDbClient("127.0.0.1:1369", {
      traceId,
    });

    try {
      const response = await client.listStores(new ListStores());
      return response;
    } finally {
      span.end();
    }
  });
}
```
</details>

## Header Details

When a trace ID is provided, the SDK sets the `ahnlich-trace-id` header on every request. This header is used by the Ahnlich server for distributed tracing and logging.

## Notes

- Trace IDs are optional but recommended for production systems
- The same trace ID can be used across multiple clients to correlate requests
- See [Distributed Tracing](/docs/components/distributed-tracing) for server-side setup
