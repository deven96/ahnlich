---
title: Get By Predicate
sidebar_position: 10
---

# Get By Predicate

The GetPred request retrieves entries from a store that match a specified predicate condition on their metadata.

* **Input**: Store name and predicate condition.

* **Behavior**: Returns all entries whose metadata matches the predicate condition.

* **Response**: A list of matching entries.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { GetPred } from "ahnlich-client-node/grpc/db/query_pb";
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function getByPredicate() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.getPred(
    new GetPred({
      store: "my_store",
      condition: new PredicateCondition({
        kind: {
          case: "value",
          value: new Predicate({
            kind: {
              case: "equals",
              value: new Equals({
                key: "label",
                value: new MetadataValue({ value: { case: "rawString", value: "A" } }),
              }),
            },
          }),
        },
      }),
    })
  );

  console.log(`Found ${response.entries.length} entries with label='A'`);
}

getByPredicate();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `condition` | `PredicateCondition` | Yes | The filter condition |

## Available Predicates

| Predicate | Description |
|-----------|-------------|
| `Equals` | Match exact value |
| `NotEquals` | Exclude exact value |
| `In` | Match if value is in a given set |
| `NotIn` | Match if value is not in a given set |
| `Contains` | Match if string contains substring |
| `NotContains` | Match if string does not contain substring |

## Example with NotEquals

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { GetPred } from "ahnlich-client-node/grpc/db/query_pb";
import { PredicateCondition, Predicate, NotEquals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function getNotEqualsPredicate() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.getPred(
    new GetPred({
      store: "my_store",
      condition: new PredicateCondition({
        kind: {
          case: "value",
          value: new Predicate({
            kind: {
              case: "notEquals",
              value: new NotEquals({
                key: "status",
                value: new MetadataValue({ value: { case: "rawString", value: "archived" } }),
              }),
            },
          }),
        },
      }),
    })
  );

  console.log(`Found ${response.entries.length} non-archived entries`);
}

getNotEqualsPredicate();
```
</details>

## Example with AND Condition

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { GetPred } from "ahnlich-client-node/grpc/db/query_pb";
import { 
  PredicateCondition, 
  Predicate, 
  Equals, 
  AndCondition 
} from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function getWithAndCondition() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.getPred(
    new GetPred({
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
                      key: "category",
                      value: new MetadataValue({ value: { case: "rawString", value: "electronics" } }),
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
                      key: "brand",
                      value: new MetadataValue({ value: { case: "rawString", value: "Apple" } }),
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

  console.log(`Found ${response.entries.length} Apple electronics`);
}

getWithAndCondition();
```
</details>

## Notes

- For optimal performance, create predicate indices on frequently filtered fields
- Complex conditions (AND/OR) are supported for advanced filtering
- See [Type Meanings](/docs/client-libraries/node/type-meanings) for more details on predicates
