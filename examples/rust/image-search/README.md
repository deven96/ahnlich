## Image search

An example on how to use the rust sdk that shows the process of indexing a couple of images and 
into the db and querying those via text

### What This Example Does

To install dependencies (ensure you have cargo installed)  
```cargo build```

Place the images into the images folder and run
```cargo run index```

- Each image within the `images` folder is indexed 
  * One of the models supported by `ahnlich-ai` i.e `ClipVitB32Image` is used to generate embeddings for the images
    * Embeddings are then stored within `ahnlich-db` vector datastore
  * ![insertion gif](index-image.gif)

To search run  
```cargo run query```

- Query text is provided
  * `ClipVitB32Text` is used to generate embeddings for the query text.
  * Embedding from the query text is compared using `CosineAlgorithm` against every embedding in the vector datastore
  * Closest embedding is identified and the corresponding image pixels are rendered to screen
  * ![query gif](query-image.gif)

