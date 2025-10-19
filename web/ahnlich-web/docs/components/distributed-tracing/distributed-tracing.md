---
title: 🕸️ Distributed Tracing
---

# Distributed Tracing in Ahnlich


Distributed tracing allows you to track requests across Ahnlich DB and Ahnlich AI. Since many queries flow from the AI service into the DB, tracing provides end-to-end visibility of what happens inside the pipeline. This is critical for diagnosing bottlenecks, monitoring performance, and ensuring reliability in production.

Both **Ahnlich DB** and **Ahnlich AI** support tracing through **OpenTelemetry (OTel)**. You can send spans to a backend like **Jaeger** for visualization.
