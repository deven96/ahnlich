---
title: Type Meanings
sidebar_posiiton: 5
---

# Type Meanings

The following terms are fundamental to how **Ahnlich AI requests** are structured and processed.

## Store Key

A one-dimensional vector that uniquely identifies an item in the store.

* Functions like a **primary key** in a database.

* Ensures that every stored entry has a distinct handle for retrieval and indexing.

* Source Code Source Code Example: a numerical vector representing an embedding for a product image.

## Store Value

A dictionary containing texts or binary data associated with a `StoreKey`.

* Stores the **payload** of information that can be retrieved, searched, or filtered.

* May include metadata such as titles, descriptions, or binary content (like embeddings, files, or serialized objects).

* Think of it as the "body" of the data linked to the store key.

## Store Predicates (Predicate Indices)

Special indices built on top of `StoreValue` fields to make filtering more efficient.

* They **optimize lookups** by pre-indexing specific fields.

* Useful when you need fast filtering by metadata like "`job`" or "`rank`".

* Without them, searches would be slower since the system would need to scan every entry.

## Predicates

Operations that define how filtering is performed on data.

* Examples include:

  * `Equals` → match exact values.

  * `NotEquals` → exclude values.

  * `Contains` → substring or set membership checks.

* They are always tied to a **key** in a `StoreValue` and evaluated against a **metadata value**.

* Provide the basic building blocks for query logic.

## PredicateConditions

Conditions that **wrap predicates** and allow combining them logically.

* A `PredicateCondition` can represent:

  * A **single predicate** (just one filter condition).

  * A **compound condition** using `AND` or `OR`.

* This makes it possible to construct **complex filters**, e.g., “all sorcerers who are chunin rank.”

### Example – single predicate condition:

```py
condition = predicates.PredicateCondition(
    value=predicates.Predicate(
        equals=predicates.Equals(
            key="job", value=metadata.MetadataValue(raw_string="sorcerer")
        )
    )
)
```

### Example – binary metadata value:

```py
condition = predicates.PredicateCondition(
    value=predicates.Predicate(
        equals=predicates.Equals(
            key="rank", value=metadata.MetadataValue(image=[2,2,3,4,5,6,7])
        )
    )
)
```

### Example – compound condition with `AND`:

```py
condition = predicates.PredicateCondition(
    and_=predicates.AndCondition(
        left=predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="job",
                    value=metadata.MetadataValue(raw_string="sorcerer")
                )
            )
        ),
        right=predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="rank",
                    value=metadata.MetadataValue(raw_string="chunin")
                )
            )
        )
    )
)
```

## MetadataValue

The container used inside predicates to hold values.

* Supports both **raw strings** (like "`sorcerer`") and **binary vectors** (lists of bytes/integers).

* This makes it versatile enough to handle both **structured text metadata** and **embeddings or binary payloads**.

## Search Input

The query input sent to Ahnlich AI for processing.

* Can be a **string** (text input, e.g., "`What is AI?`") or a **binary file** (like an image or audio file).

* The type of input depends on the **AI model** and the **store configuration** (string vs. binary store).

## AIModels

The set of supported AI models within Ahnlich AI.

* Each model determines the **type of input** it can process (e.g., text-only, image, multimodal).

* Choosing the right model ensures that the search input is properly understood and processed.

## AIStoreType

Defines the type of store being created.

* **String Store** - optimized for textual inputs and queries.

* **Binary Store** - optimized for binary data like embeddings, images, or raw vectors.

* Must be chosen carefully depending on whether you are working with **text-based AI models** or **binary models**.


