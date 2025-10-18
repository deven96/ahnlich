---
title: ♾️ Persistence In Ahnlich 
sidebar_position: 10
---


# Persistence in Ahnlich
## Overview
By default, both **Ahnlich DB** and **Ahnlich AI** services run in **in-memory mode**, meaning all data (keys, predicates, embeddings, and indices) is stored only in RAM. This is fast for retrieval, but it also means:
- All data is lost when the service restarts.

- Stores and indices must be recreated after every shutdown.

- Models and embeddings need to be recomputed repeatedly.

**Persistence** solves this by periodically saving data to disk and reloading it on startup. This ensures that **your DB and AI services are fault-tolerant, production-ready, and stateful across restarts**.
