---
title: Distributed Tracing
---

# Distributed Tracing

The clients support **W3C Trace Context** via optional `traceparent` headers, enabling distributed tracing across microservices.

## Usage
* Many client methods accept an `Option<String>` parameter named `tracing_id`.

* Passing a `Some(trace_id)` propagates context to downstream services for observability.

* Leaving it as `None` will execute the request without attaching tracing metadata.

## Benefits
* Provides full request visibility across DB and AI pipelines.

* Facilitates debugging and performance monitoring in distributed systems.

* Supports correlation of requests across multiple services and pipelines.

