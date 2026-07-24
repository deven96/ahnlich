---
title: DROP PREDICATE INDEX
sidebar_label: DROP PREDICATE INDEX
---

# DROP PREDICATE INDEX

Remove a predicate index. The data stays; queries just lose the speed-up.

## Syntax

```bash
DROP PREDICATE INDEX <field> IN <store_name>
DROP PREDICATE INDEX <field> IN <store_name> SCHEMA analytics
```

## Example

```bash
DROP PREDICATE INDEX category IN my_store
```

---

See [SDK examples](/docs/stores/drop-predicate-index) for Python, Node, Go, and Rust.
