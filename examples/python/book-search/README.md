## Book search

An example on how to use the python sdk that shows the process of splitting and 
inserting an epub ebook into the db and querying it via a search phrase either directly or contextually

### What this Example Does

To install dependencies (ensure you have poetry installed)  
```poetry install```

To insert run  
```poetry run insertbook```

- The book _(Animal Farm by George Orwell)_ is processed and indexed
  * `epub` file is split up into paragraph and cleaned a bit
  * Embeddings are generated by `ahnlich-ai` using the `BGEBaseEnV15`
    * Generated embeddings are stored within `ahnlich-db` vector datastore
  * ![insertion gif](insertbook.gif)

To search run  
```poetry run searchbook```

- Query text is provided
  * `BGEBaseEnV15` is used to generated embeddings for the query text
  * Embedding from the query text is compared using `CosineAlgorithm` against every embedding in the vector datastore
  * Closest 5 embeddings are identified and are the corresponding paragraphs are printed out in order of similarity
  * ![insertion gif](searchphrase.gif)