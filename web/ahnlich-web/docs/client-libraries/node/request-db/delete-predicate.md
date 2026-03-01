---
title: Delete Predicate
sidebar_position: 16
---

# Delete Predicate

The DelPred request deletes all entries from a store that match a specified predicate condition.

* **Input**: Store name and predicate condition.

* **Behavior**: Removes all entries whose metadata matches the predicate.

* **Response**: Confirmation of deletion with count of deleted entries.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { DelPred } from "ahnlich-client-node/grpc/db/query_pb";
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function deletePredicate() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.delPred(
    new DelPred({
      store: "my_store",
      condition: new PredicateCondition({
        kind: {
          case: "value",
          value: new Predicate({
            kind: {
              case: "equals",
              value: new Equals({
                key: "status",
                value: new MetadataValue({ value: { case: "rawString", value: "archived" } }),
              }),
            },
          }),
        },
      }),
    })
  );

  console.log(`Deleted ${response.deleted} entries`);
}

deletePredicate();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `condition` | `PredicateCondition` | Yes | The filter condition for deletion |

## Example with Complex Condition

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { DelPred } from "ahnlich-client-node/grpc/db/query_pb";
import { 
  PredicateCondition, 
  Predicate, 
  Equals, 
  AndCondition 
} from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function deleteWithComplexCondition() {
  const client = createDbClient("127.0.0.1:1369");

  // Delete all archived items in category "temp"
  const response = await client.delPred(
    new DelPred({
      store: "my_store",
      condition: new PredicateCondition({
        kind: {
          case: "and",
          value: new AndCondition({
            left: new PredicateCondition({
              kind: {
                case: "value",
                value: new Predicate({
                  kind: {
                    case: "equals",
                    value: new Equals({
                      key: "status",
                      value: new MetadataValue({ value: { case: "rawString", value: "archived" } }),
                    }),
                  },
                }),
              },
            }),
            right: new PredicateCondition({
              kind: {
                case: "value",
                value: new Predicate({
                  kind: {
                    case: "equals",
                    value: new Equals({
                      key: "category",
                      value: new MetadataValue({ value: { case: "rawString", value: "temp" } }),
                    }),
                  },
                }),
              },
            }),
          }),
        },
      }),
    })
  );

  console.log(`Deleted ${response.deleted} archived temp entries`);
}

deleteWithComplexCondition();
```
</details>

## Notes

- Use with caution - this operation permanently deletes data
- For deleting specific entries, use [Delete Key](/docs/client-libraries/node/request-db/delete-key)
- Predicate indices improve the performance of predicate-based deletions
