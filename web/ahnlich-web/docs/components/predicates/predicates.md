---
title: 🔍 Predicates
sidebar_position: 40
---

# Predicates

Predicates in Ahnlich are **metadata-based filters** that allow you to query and filter stored data beyond just similarity search. They work alongside vector similarity to enable powerful hybrid queries.

## Overview

When you store data in Ahnlich (both DB and AI), each entry consists of:
- **Key**: The vector embedding (or raw key for DB stores)
- **Value**: Metadata as key-value pairs (e.g., `{"author": "Alice", "category": "tech"}`)

Predicates let you filter results based on this metadata.

## Core Concepts

### Predicate

A **predicate** is a single comparison operation on a metadata field. Ahnlich supports four operators:

| Operator | Description | Example |
|----------|-------------|---------|
| `EQUALS` | Exact match | `author = "Alice"` |
| `NOT EQUALS` | Not equal | `status != "draft"` |
| `IN` | Value in list | `category IN ["tech", "science"]` |
| `NOT IN` | Value not in list | `priority NOT IN ["low", "spam"]` |

### Predicate Condition

A **predicate condition** combines predicates using logical operators:

| Operator | Description | Example |
|----------|-------------|---------|
| `AND` | Both conditions must be true | `(author = "Alice") AND (category = "tech")` |
| `OR` | Either condition must be true | `(status = "published") OR (priority = "high")` |

Conditions can be nested to create complex queries:
```
(author = "Alice" AND category = "tech") OR (priority = "high")
```

### Predicate Index

A **predicate index** optimizes queries on specific metadata fields. Without an index, Ahnlich scans all entries. With an index, lookups are much faster.

**Creating an index:**
```
CREATEPREDINDEX author, category IN my_store
```

**Dropping an index:**
```
DROPPREDINDEX category IN my_store
```

Indexes are idempotent - creating an existing index won't error, it just adds new ones.

## Supported Data Types

Predicates work with metadata values of type:
- **String** - Text values like `"Alice"`, `"tech"`, `"published"`
- **Binary** - Raw bytes (for image hashes, etc.)

## Examples

### Simple Predicate Query

Find all entries where author is "Alice":

```
GETPRED (author = "Alice") IN my_store
```

### Complex Condition

Find tech articles by Alice or Bob:

```
GETPRED ((author = "Alice") OR (author = "Bob")) AND (category = "tech") IN my_store
```

### Hybrid Query (Similarity + Predicate)

Find similar documents, filtered by author:

**Ahnlich AI:**
```
GETSIMN 5 WITH [machine learning tutorial] USING cosinesimilarity IN my_store WHERE (author = "Alice")
```

**Ahnlich DB:**
```
GETSIMN 5 WITH [0.1, 0.2, ...] USING cosinesimilarity IN my_store WHERE (author = "Alice")
```

This combines vector similarity with metadata filtering.

### Using IN Operator

Find entries with multiple allowed values:

```
GETPRED category IN ["tech", "science", "ai"] IN my_store
```

### Delete by Predicate

Remove all draft entries:

```
DELPRED (status = "draft") IN my_store
```

## Best Practices

### 1. Create Indexes for Frequently Queried Fields

If you often filter by `author`, create an index:
```
CREATEPREDINDEX author IN my_store
```

### 2. Declare Predicates at Store Creation

Specify expected predicates upfront:

**Ahnlich AI:**
```
CREATESTORE my_store 
  QUERYMODEL all-minilm-l6-v2 
  INDEXMODEL all-minilm-l6-v2 
  PREDICATES (author, category, status)
```

**Ahnlich DB:**
```
CREATESTORE my_store 
  DIMENSION 384 
  CREATEPREDINDEX (author, category, status)
```

### 3. Keep Metadata Lightweight

Predicates are for filtering, not bulk data storage. Keep values small:
- ✅ Good: `{"author": "Alice", "category": "tech"}`
- ❌ Bad: `{"content": "<entire article text>"}`

### 4. Use AND/OR Efficiently

- Use `AND` when both conditions must match (narrows results)
- Use `OR` when either condition matches (broadens results)

### 5. Index High-Cardinality Fields

If a field has many unique values (like `author`), index it. Low-cardinality fields (like `status` with only 2-3 values) benefit less from indexing.

## CLI Command Reference

| Command | Description |
|---------|-------------|
| `CREATEPREDINDEX` | Create index on metadata fields |
| `DROPPREDINDEX` | Remove index from fields |
| `GETPRED` | Query by predicate only |
| `GETSIMN ... WHERE` | Similarity search filtered by predicate |
| `DELPRED` | Delete entries matching predicate |

See detailed examples in:
- [AI CLI Commands](../ahnlich-cli/ai-commands)
- [DB CLI Commands](../ahnlich-cli/db-commands)

## Limitations

- Predicates only support **equality-based comparisons** (no regex, range queries, or full-text search yet)
- Index updates are **synchronous** - creating an index on a large store may take time
- No support for **numeric comparisons** (e.g., `age > 18`) - use string equality as a workaround

## Advanced: Predicate Internals

Under the hood, predicates are structured as:

```
PredicateCondition:
  - Value(Predicate)      // Single predicate
  - And(left, right)      // Logical AND
  - Or(left, right)       // Logical OR

Predicate:
  - Equals(key, value)
  - NotEquals(key, value)
  - In(key, [values])
  - NotIn(key, [values])
```

This structure allows nesting and complex boolean logic while keeping the implementation efficient.
