---
title: Type Meanings
sidebar_posiiton: 5
---

# Type Meanings

Key concepts and their meanings in the SDK:

1. StoreKey

- DB: represented as a raw `[]float32` vector.

- AI: represented as a `StoreInput` object, which may be raw text, numbers, or serialized embeddings.

2. StoreValue

- A `map` of metadata fields associated with a stored entry (e.g., labels, tags, or custom attributes).

3. Predicates

- Filtering conditions that restrict which stored values are retrieved or indexed. Useful for constrained similarity search.

Pipeline

- A batch RPC builder that allows combining multiple requests (such as `Set`, `GetKey`, `GetSimN`, etc.) into a single execution for efficiency.

This section provides developers with clear guidance for **building, testing, and releasing** the SDK, as well as an understanding of **core data types** used in both DB and AI contexts.
