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

-  `efConstruction`:  controls the breadth of the search during the insertion phase.
When a new element is added to the index, HNSW performs a multi-layer search.
At layer 0, instead of performing a greedy search (which can easily get stuck), HNSW executes an extended search that maintains:
    - a candidate priority queue (closest unexplored nodes)
    - a top-K priority queue (best found neighbors)

    The search explores nodes until the candidate queue reaches size efConstruction.
    The top results of this search are used to select up to M neighbors for the new node.
    Increasing efConstruction improves graph quality and recall but increases build time and memory usage.
    
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





## Algorithms

 ### Algorithm 1

```
INSERT(hnsw, q, M, Mmax, efConstruction, mL)
Input: multilayer graph hnsw, new element q, number of established
connections M, maximum number of connections for each element
per layer Mmax, size of the dynamic candidate list efConstruction, normalization factor for level generation mL
Output: update hnsw inserting element q
1 W ← ∅ // list for the currently found nearest elements
2 ep ← get enter point for hnsw
3 L ← level of ep // top layer for hnsw
4 l ← ⌊-ln(unif(0..1))∙mL⌋ // new element’s level
5 for lc ← L … l+1
6   W ← SEARCH-LAYER(q, ep, ef=1, lc)
7   ep ← get the nearest element from W to q

8 for lc ← min(L, l) … 0
9   W ← SEARCH-LAYER(q, ep, efConstruction, lc)
10  neighbors ← SELECT-NEIGHBORS(q, W, M, lc) // alg. 3 or alg. 4
11  add bidirectionall connections from neighbors to q at layer lc

12  for each e ∈ neighbors // shrink connections if needed
13    eConn ← neighbourhood(e) at layer lc
14    if │eConn│ > Mmax // shrink connections of e if lc = 0 then Mmax = Mmax0
15      eNewConn ← SELECT-NEIGHBORS(e, eConn, Mmax, lc) // alg. 3 or alg. 4
16      set neighbourhood(e) at layer lc to eNewConn
17  ep ← W
18 if l > L
19  set enter point for hnsw to q

```
### Breakdown

1. **`W`**: initial empty queue of nearest elements to the q
2. **`ep`**: the current entry point of the HNSW graph.
3. **`L`**: the layer of the entry point, i.e., the highest layer in the graph.
4. **`l`**: randomly selected level for the new element using exponential distribution.
5. Loop from upper layers (`L`) down to `l+1`(level above the new elements level) where the new element does not exist yet
6. At each such layer, run `SEARCH-LAYER(q, ep, ef=1)` to perform greedy search. This gets all closest nodes at the current layer (lc) with ef=1 and store them in W.
7. Update `ep` to the closest node found in `W`; this becomes the entry point for the next layer down.

8. Loop from `min(L, l)` down to layer `0` to perform full neighbor discovery.

9. At each such layer, run `SEARCH-LAYER(q, ep, efConstruction)`, performing a thorough search for high-quality neighbors.

10. Run `SELECT-NEIGHBORS` to choose M neighbors for q.
11. Insert bidirectional links between q and selected neighbors.


12. for each neighbour `e`

13. Retrieve e’s adjacency list at this layer(`eConn`).
14.  If degree exceeds allowed limit (`Mmax` or `Mmax0 for layer 0`), prune.

15. Use `SELECT-NEIGHBORS(e, eConn, Mmax)` to prune excess neighbors.

16. Replace e’s adjacency list with pruned version.

17. Update `ep` to the *closest* element in W (not the entire W).

18. If the new node’s level `l` is greater than the max level in the HNSW structure `L`
19. update the global entry point so that future searches begin at q.


### Algorithm 2


### Breakdown

---

## **Questions & Answers/Assumptions Made**

### **Q0. How deterministic is the HNSW given it's approximate nature? With Ahnlich looking at distributed strategies, if a replica instance is spun up and a same query is sent to each replicas as independent results, how similar would the response be?**
pending...

### **Q1. Why is l chosen randomly? Why exponential distribution?**

To mimic skip lists: higher layers become exponentially rarer, giving logarithmic search complexity.

---

### **Q2. What does efConstruction do?**

Controls search breadth during construction:

* Larger efConstruction → better-quality neighbors → better recall
* But slower indexing

---

### **Q3. Does SEARCH-LAYER override W?**

Yes. W always becomes the new candidate queue.

---

### **Q4. Why is ep sometimes written as ep ← W?**

Sloppy notation. Actual meaning:

> ep = closest element in W to q
> (not the entire W)

---

### **Q5. What is difference between Algorithm 3 and Algorithm 4?**

* Algorithm 3: simple neighbor selection
* Algorithm 4: heuristic with diversity (recommended, default)

Use Algorithm 4 unless you need speed and accept lower recall.

---

### **Q6. Why do we shrink e’s connections after adding q?**

To enforce the maximum degree constraint required for navigability and performance.

---

### **Q7. Why does layer 0 use Mmax0 (larger)?**

Layer 0 holds **most** elements → needs more neighbors → increases recall.

---

### **Q8: During insertion, ep seems to change at each layer. Is there a separate ep per layer? Or how does this work?**

No, there is only one global entry point stored in the HNSW structure, which always points to the topmost node.
Inside the insert algorithm, ep is just a temporary variable (like ep_temp) used to navigate layer by layer while inserting the new element.

At each layer, ep is updated to the closest node in the current search (W), and this helps the algorithm descend efficiently.

After insertion, the permanent global entry point (global_ep) is only updated if the new node has a higher level than the current top node (line 18–19).

Otherwise, the global entry point remains unchanged, pointing to the highest-level node.

Analogy:

global_ep = elevator at the top of the building

ep (temporary) = walking down the stairs floor by floor

---


## References:

- https://arxiv.org/pdf/1603.09320
- https://keyurramoliya.com/posts/Understading-HNSW-Hierarchical-Navigable-Small-World/
