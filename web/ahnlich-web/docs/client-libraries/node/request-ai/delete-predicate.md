---
title: Delete Predicate
---

# Delete Predicate

This request removes all entries in a store that match a specified predicate condition.

* **Input:**
  * `store`: the name of the store.
  * `condition`: a logical predicate that filters which entries should be deleted.

* **Behavior:** Instead of deleting by a specific key, the server scans the store and deletes all entries that satisfy the predicate condition.

* **Response:**
  * `deletedCount` - the number of items successfully deleted.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { DelPred } from "ahnlich-client-node/grpc/ai/query_pb";
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function deletePredicate() {
  const client = createAiClient("127.0.0.1:1370");

  // Create a predicate condition to match entries where "category" equals "outdated"
  const condition = new PredicateCondition({
    value: new Predicate({
      predicate: {
        case: "equals",
        value: new Equals({
          key: "category",
          value: new MetadataValue({
            value: {
              case: "rawString",
              value: "outdated"
            }
          })
        })
      }
    })
  });

  const response = await client.delPred(
    new DelPred({
      store: "my_store",
      condition: condition
    })
  );

  console.log(`Deleted ${response.deletedCount} items`);
}

deletePredicate();
```
</details>
