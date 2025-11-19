# WIP: Thoughts and potential Spec on HNSW algorithm

> This would serve rough sheet where i dump summaries on hnsw, until it becomes a spec document. More reading is required to understand certain questions, omissions and outright wrong assumtions


## HNSW 

HNSW is an approximation algorithm built upon the idea of NSW and Skip-lists. It creates a multi-layer
structure where the number of elements increases as you move down the structure(layers)
Like the skip-list, the top most layer contains fewer elements and at each layer we have an NSW which is basically
a graph

## Parameters and importance

- `l`: The maximum layer level of an element. Defined by `l = floor(-ln(uniform(0,1)) * mL)` (Used during insert)

- `M`: Defines the maximum number of connections between each elements/node. It controls the tradeoff between memory usage and search quality by
how manay neighbors each element can maintain in layers above level 0.


- `mL`: a normalization param that controls the expected number of layers in the hierachical structure. It plays a crucial role in balancing the tradeoff between search
efficiency and construction complexity as its probabilistic assignment ensures that higher layers contain fewer elements while maintaining
proper connectivity.

-  `efConstruction`: 
- `Mmax0`: Maximum connection at layer 0

  ### Parameter Optimizations

  - `M`: `5-48`, depending on dimensionality and clustering characteristics of the data. Lower dimension datasets would perform well
  with a smaller number `5-12`, while high-dimensional data `16-48`

  *Questions for Ahnlich...??*
  > - How do we define a default for this value if not provided by the user? If a wrong default is chosen, do we recreate after
    we understand the dimensionality of said data??




## Others:
  ### Memory and Contruction Complexity
  - Memory Complexity: Memory consumption for hnsw is directly proportional to the dataset size. Meaning it scales
  linearly. Total memory requirement can be expressed approximately as: 
    `(Mmax0 + mL * Mmax) * bytes_per_link per element` where:
      
    Mmax: the number of connections allowed at layers above level 0.

    **NOTE: for typical paremeter settings, this is roughly 60-450bytes per element for the graph structure alone, excluding the storage
    required for the original vector data**
  

  - Construction Complexity: Theoretically, it is `O(N log N)` for construction on relatively low-dimensional data. Where N represents the number of elements.
    *Questions for Ahnlich...??*
    > - What is the construction complexity for high-dimensional data??



## More Questions:
- How deterministic is the HNSW given it's approximate nature? With Ahnlich looking at distributed strategies, if a replica instance is spun up and
a same query is sent to each replicas as independent results, how similar would the response be?


## References:

- https://arxiv.org/pdf/1603.09320
- https://keyurramoliya.com/posts/Understading-HNSW-Hierarchical-Navigable-Small-World/
