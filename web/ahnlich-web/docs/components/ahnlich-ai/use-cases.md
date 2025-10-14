---
title: Use Cases
---

# Use Cases

## 1. Semantic Search with Raw Input

A news website integrates Ahnlich AI with `all-minilm-l6-v2`. Instead of keyword search, users type:

```
SEARCH "climate change and food security" IN news_store WHERE (topic != sports)
```

- The text query is embedded automatically.

- Stored articles are embedded consistently.

- Ahnlich DB returns top semantic matches, filtering irrelevant categories.

This provides **conceptual search** rather than exact word matching.

## 2. Cross-Modal Search (Text ↔ Image)

A fashion platform configures:
- Index Model = `resnet-50` (for product images)

- Query Model = `all-minilm-l6-v2` (for user text queries)

When a user searches for a product:

```
GETSIMN 5 WITH [red summer dress] USING cosinesimilarity IN fashion_store
```

Ahnlich AI embeds the **text query** and compares it against **image embeddings** in the store. This allows retrieving visually similar dresses from the catalog — without the store owner needing to manually tag the images.

## 3. Personalized Recommendations
Ahnlich AI can also transform **user profiles** or **behaviors into embeddings** automatically.

Example: an e-commerce platform using `product_store`:

```
GETSIMN 5 WITH [eco-friendly home products] USING cosinesimilarity IN product_store WHERE (status = in_stock)
```

Here:
- The **user query** is embedded.

- It’s matched against **product embeddings**.

- The results are filtered using the **predicate** `(status = in_stock)`.

This enables **real-time, personalized product recommendations** tailored to availability.

## 4. Multimodal Applications in Healthcare
Ahnlich AI does not support mixing image and text embeddings in a single store. Each store is model-aware and tied to one input type (text or image).

For workflows such as CT scans and radiology reports, the recommended approach is to create **two separate stores** and link them using metadata fields like `patient_id` or `report_id`.

### Pattern A  Two Stores with Metadata Linking

<details>
  <summary>Click to expand</summary>

  ```
  create an image store
  CREATESTORE ct_image_store QUERYMODEL resnet-50 INDEXMODEL resnet-50 STOREORIGINAL;
  CREATEPREDINDEX (patient_id, report_id) IN ct_image_store;

  create a text report store
  CREATESTORE report_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2 STOREORIGINAL;
  CREATEPREDINDEX (patient_id, report_id) IN report_store;

  insert CT scan
  SET (([<image-vector>], {patient_id: "P123", report_id: "R789"})) IN ct_image_store;

  insert report
  SET ((["Findings: ground-glass opacities ..."], {patient_id: "P123", report_id: "R789"})) IN report_store;

  query CT scans
  GETSIMN 5 WITH [<image-vector>] USING cosinesimilarity IN ct_image_store;
  fetch linked report
  GETPRED (report_id = "R789") IN report_store;
  ```
</details>

Here, similarity search on `ct_image_store` finds related scans, and `report_id` links results to the associated report in `report_store`.

### Pattern B — Cross-Modal Matching with CLIP

<details>
  <summary>Click to expand</summary>
  ```
  -- create a cross-modal store (text ↔ image)
  CREATESTORE clip_image_store QUERYMODEL CLIP_VIT_B32_TEXT INDEXMODEL CLIP_VIT_B32_IMAGE STOREORIGINAL;
  CREATEPREDINDEX (patient_id, report_id) IN clip_image_store;

  insert CT scan
  SET (([<image-vector>], {patient_id: "P123", report_id: "R789"})) IN clip_image_store;

  query with text (embedded via CLIP text model)
  GETSIMN 5 WITH ["ground-glass opacity in left lung"] USING cosinesimilarity IN clip_image_store
  ```
</details>

## 5. Real-Time Assistance
A chatbot connected to Ahnlich AI can provide real-time recommendations by retrieving similar past support tickets.

Example flow:
1. The user submits a query:
 `"I need help with my billing issue"`

2. The query is converted into embeddings automatically.

3. Retrieve the top N similar past tickets using `GETSIMN`:


    ```
    GETSIMN 5 WITH ["I need help with my billing issue"] USING cosinesimilarity IN support_store
    ```

4. Optionally, filter results using metadata with `GETPRED`:

    ```
    GETPRED (resolved = true) IN support_store
    ```

This workflow allows the chatbot to return relevant historical solutions with low latency, using similarity search combined with predicate filtering.
