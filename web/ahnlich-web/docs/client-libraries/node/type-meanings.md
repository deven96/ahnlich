---
title: Type Meanings
sidebar_position: 6
---

# Type Meanings

The following terms are fundamental to how **Ahnlich requests** are structured and processed in the Node.js SDK.

## StoreKey

A one-dimensional `float32` vector that uniquely identifies an item in a DB store.

* Functions like a **primary key** in a database
* Vector dimension must match the store's configured dimension
* Used in DB operations (Set, GetKey, DelKey, GetSimN)

```ts
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";

const key = new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] });
```

## StoreValue

A map of string keys to `MetadataValue` containing the payload associated with an entry.

* Stores metadata like titles, descriptions, categories
* Can contain both text and binary data

```ts
import { StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

const value = new StoreValue({
  value: {
    label: new MetadataValue({ value: { case: "rawString", value: "Product A" } }),
    category: new MetadataValue({ value: { case: "rawString", value: "Electronics" } }),
  },
});
```

## StoreInput

A raw string or binary blob accepted by the AI proxy for automatic embedding generation.

* Used in AI operations instead of StoreKey
* The AI server converts inputs to embeddings automatically

```ts
import { StoreInput } from "ahnlich-client-node/grpc/keyval_pb";

// Text input
const textInput = new StoreInput({
  value: { case: "rawString", value: "Hello world" },
});

// Binary input (e.g., image)
const imageInput = new StoreInput({
  value: { case: "image", value: new Uint8Array([...]) },
});
```

## Predicates

Operations that define how filtering is performed on metadata.

| Predicate | Description | Example |
|-----------|-------------|---------|
| `Equals` | Match exact value | `label = "A"` |
| `NotEquals` | Exclude exact value | `status != "archived"` |
| `In` | Match if value in set | `category IN ["A", "B"]` |
| `NotIn` | Match if value not in set | `status NOT IN ["deleted"]` |
| `Contains` | String contains substring | `name CONTAINS "shoe"` |
| `NotContains` | String doesn't contain | `name NOT CONTAINS "test"` |

```ts
import { Predicate, Equals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

const predicate = new Predicate({
  kind: {
    case: "equals",
    value: new Equals({
      key: "category",
      value: new MetadataValue({ value: { case: "rawString", value: "shoes" } }),
    }),
  },
});
```

## PredicateCondition

Conditions that wrap predicates and allow combining them logically.

* Can represent a single predicate
* Can combine predicates with `AND` or `OR`

### Single Predicate

```ts
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

const condition = new PredicateCondition({
  kind: {
    case: "value",
    value: new Predicate({
      kind: {
        case: "equals",
        value: new Equals({
          key: "brand",
          value: new MetadataValue({ value: { case: "rawString", value: "Nike" } }),
        }),
      },
    }),
  },
});
```

### AND Condition

```ts
import { PredicateCondition, Predicate, Equals, AndCondition } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

const condition = new PredicateCondition({
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
                key: "brand",
                value: new MetadataValue({ value: { case: "rawString", value: "Nike" } }),
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
                value: new MetadataValue({ value: { case: "rawString", value: "Running" } }),
              }),
            },
          }),
        },
      }),
    }),
  },
});
```

## MetadataValue

The container used inside predicates and store values to hold data.

| Type | Description |
|------|-------------|
| `rawString` | Text data |
| `image` | Binary image data |

```ts
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

// String value
const textValue = new MetadataValue({
  value: { case: "rawString", value: "Hello" },
});

// Binary value
const binaryValue = new MetadataValue({
  value: { case: "image", value: new Uint8Array([0x01, 0x02, 0x03]) },
});
```

## AIModel

The set of supported AI models within Ahnlich AI.

| Model | Type | Description |
|-------|------|-------------|
| `ALL_MINI_LM_L6_V2` | Text | Sentence transformer (384 dimensions) |
| `BGE_BASE_EN_V1_5` | Text | BAAI text embeddings |
| `BGE_LARGE_EN_V1_5` | Text | BAAI large text embeddings |
| `RESNET50` | Image | Image classification |
| `CLIP_VIT_B32` | Multimodal | Text and image embeddings |
| `BUFFALO_L` | Face | Face detection/recognition |
| `SFACE_YUNET` | Face | Face detection with YuNet |
| `CLAP` | Audio | Audio embeddings |

```ts
import { AIModel } from "ahnlich-client-node/grpc/ai/models_pb";

const model = AIModel.ALL_MINI_LM_L6_V2;
```

## Algorithm

Similarity algorithms for vector search.

| Algorithm | Description |
|-----------|-------------|
| `COSINE_SIMILARITY` | Cosine similarity (good for text) |
| `EUCLIDEAN_DISTANCE` | Euclidean distance (L2 norm) |
| `DOT_PRODUCT` | Dot product similarity |

```ts
import { Algorithm } from "ahnlich-client-node/grpc/algorithm/algorithm_pb";

const algo = Algorithm.COSINE_SIMILARITY;
```

## NonLinearAlgorithm

Non-linear indexing algorithms for faster similarity search.

| Algorithm | Description |
|-----------|-------------|
| `KDTree` | K-dimensional tree (lower dimensions) |
| `HNSW` | Hierarchical Navigable Small World (high dimensions) |

```ts
import { NonLinearAlgorithm, KDTreeConfig, HNSWConfig } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

const algo = NonLinearAlgorithm.HNSW;
```
