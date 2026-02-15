---
title: Quick Reference
sidebar_position: 10
---

# Predicates Quick Reference

## Operators

### Comparison Operators

| Operator | Syntax | Example |
|----------|--------|---------|
| Equals | `key = value` | `author = "Alice"` |
| Not Equals | `key != value` | `status != "draft"` |
| In | `key IN [values]` | `category IN ["tech", "ai"]` |
| Not In | `key NOT IN [values]` | `priority NOT IN ["low"]` |

### Logical Operators

| Operator | Syntax | Example |
|----------|--------|---------|
| AND | `(condition) AND (condition)` | `(author = "Alice") AND (category = "tech")` |
| OR | `(condition) OR (condition)` | `(status = "draft") OR (status = "review")` |

## Common Patterns

### Single Field Filter
```
GETPRED (category = "tech") IN articles
```

### Multiple Field Filter (AND)
```
GETPRED (author = "Alice") AND (status = "published") IN articles
```

### Multiple Field Filter (OR)
```
GETPRED (priority = "high") OR (priority = "urgent") IN tasks
```

### Using IN for Multiple Values
```
GETPRED category IN ["tech", "science", "ai"] IN articles
```

### Nested Conditions
```
GETPRED 
  ((author = "Alice") OR (author = "Bob")) 
  AND 
  (category = "tech") 
IN articles
```

### Hybrid Query (Similarity + Filter)

**AI:**
```
GETSIMN 10 WITH [machine learning] 
  USING cosinesimilarity 
  IN articles 
  WHERE (author = "Alice")
```

**DB:**
```
GETSIMN 10 WITH [0.1, 0.2, 0.3, ...] 
  USING cosinesimilarity 
  IN articles 
  WHERE (category = "tech")
```

## Index Management

### Create Index
```
CREATEPREDINDEX author, category, status IN my_store
```

### Drop Index
```
DROPPREDINDEX category IN my_store
```

### Drop Index with Error Handling
```
DROPPREDINDEX category IN my_store ERRORIFDOESNOTEXIST
```

## Store Creation with Predicates

### Ahnlich AI
```
CREATESTORE my_store 
  QUERYMODEL all-minilm-l6-v2 
  INDEXMODEL all-minilm-l6-v2 
  PREDICATES (author, category, status)
```

### Ahnlich DB
```
CREATESTORE my_store 
  DIMENSION 384 
  CREATEPREDINDEX (author, category, status)
```

## Delete by Predicate

### Simple Delete
```
DELPRED (status = "deleted") IN articles
```

### Complex Delete
```
DELPRED 
  (status = "draft") 
  AND 
  (last_modified < "2024-01-01") 
IN articles
```

## Tips

✅ **Do:**
- Create indexes for frequently filtered fields
- Use `IN` for multiple values instead of chaining `OR`
- Keep metadata values small and simple
- Declare predicates when creating stores

❌ **Don't:**
- Store large text in metadata (use for filtering only)
- Create indexes on rarely used fields
- Forget to index high-cardinality fields (many unique values)

## Syntax Cheat Sheet

```
Predicate:
  key = "value"
  key != "value"
  key IN ["val1", "val2"]
  key NOT IN ["val1"]

PredicateCondition:
  (predicate)
  (condition) AND (condition)
  (condition) OR (condition)
  ((cond1) AND (cond2)) OR (cond3)  # Nested

Commands:
  CREATEPREDINDEX field1, field2 IN store
  DROPPREDINDEX field1 IN store
  GETPRED (condition) IN store
  GETSIMN n WITH [input] USING algorithm IN store WHERE (condition)
  DELPRED (condition) IN store
```
