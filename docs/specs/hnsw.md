# WIP: Thoughts and potential Spec on HNSW algorithm

> This would serve rough sheet where i dump summaries on hnsw, until it becomes a spec document. More reading is required to understand certain questions, omissions and outright wrong assumtions

---

# **Table of Contents**

* [HNSW](#hnsw)

* [Parameters and Importance](#parameters-and-importance)

  * [Parameter Optimizations](#parameter-optimizations)

* [Others](#others)

  * [Memory and Construction Complexity](#memory-and-construction-complexity)

* [More Questions](#more-questions)

* [Algorithms](#algorithms)

  * [Algorithm 1 — INSERT](#algorithm-1)

    * [Breakdown](#breakdown)

  * [Algorithm 2 — SEARCH-LAYER](#algorithm-2)

    * [Breakdown](#breakdown-1)

  * [Algorithm 3 — SELECT-NEIGHBORS-SIMPLE](#algorithm-3)

    * [Breakdown](#breakdown-2)

  * [Algorithm 4 — SELECT-NEIGHBORS-HEURISTICS](#algorithm-4)

    * [Breakdown](#breakdown-3)

  * [Algorithm 5 — KNN-SEARCH](#algorithm-5)

    * [Breakdown](#breakdown-4)

* [References](#references)


---




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
  ### Memory and Construction Complexity
  - Memory Complexity: Memory consumption for hnsw is directly proportional to the dataset size. Meaning it scales
  linearly. Total memory requirement can be expressed approximately as: 
    `(Mmax0 + mL * Mmax) * bytes_per_link per element` where:
      
    Mmax: the number of connections allowed at layers above level 0.

    **NOTE: for typical paremeter settings, this is roughly 60-450bytes per element for the graph structure alone, excluding the storage
    required for the original vector data**
  

  - Construction Complexity: Theoretically, it is `O(N log N)` for construction on relatively low-dimensional data. Where N represents the number of elements.
    *Questions for Ahnlich...??*
    > - What is the construction complexity for high-dimensional data??



<!-- Move-->
## **More Questions**

### **Q0. How deterministic is the HNSW given it's approximate nature? With Ahnlich looking at distributed strategies, if a replica instance is spun up and a same query is sent to each replicas as independent results, how similar would the response be?**
pending...



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


---

## **Questions & Answers/Assumptions Made**

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


### Algorithm 2

```
SEARCH-LAYER(q, ep, ef, lc)
Input: query element q, enter points ep, number of nearest to q elements to return ef, layer number lc
Output: ef closest neighbors to q
1 v ← ep // set of visited elements
2 C ← ep // set of candidates
3 W ← ep // dynamic list of found nearest neighbors
4 while │C│ > 0
5   c ← extract nearest element from C to q
6   f ← get furthest element from W to q
7   if distance(c, q) > distance(f, q)
8     break // all elements in W are evaluated
9   for each e ∈ neighbourhood(c) at layer lc // update C and W
10    if e ∉ v
11      v ← v ⋃ e
12      f ← get furthest element from W to q
13      if distance(e, q) < distance(f, q) or │W│ < ef
14        C ← C ⋃ e
15        W ← W ⋃ e
16        if │W│ > ef
17          remove furthest element from W to q
18 return W

```

---

### **Breakdown**

1. **`v`**: Set of visited elements to avoid cycles. Initialized with `ep`.
2. **`C`**: Candidate set (priority queue) for nodes whose neighbors need exploration. Initialized with `ep`.
3. **`W`**: Dynamic list of current closest neighbors. Maintains top `ef` nodes (queue ordered by distance to `q`). Initialized with `ep`.
4. **`while |C| > 0`**: Keep exploring candidates as long as there are nodes to evaluate.
5. **`c ← extract nearest from C`**: Pick the closest node to `q` from candidate set `C`.
6. **`f ← get furthest element from W`**: Find the current “worst” nearest neighbor.
7. **`if distance(c, q) > distance(f, q)` → break**: Stop search if remaining candidates are worse than current nearest neighbors.
9. **Explore neighbors of `c`** (nodes directly connected to `c` in layer `lc`):
   * For each neighbor `e`, if not visited (`e ∉ v`), mark as visited (`v ∪ e`).
   * Update `f` after adding `e` to `v`.
   * If `e` is closer than `f` **or `W` isn’t full yet** (`|W| < ef`):
     * Add `e` to candidate set `C` and to `W`.
     * Update W: if `|W| > ef`, remove the furthest element.
. Repeat until `C` is empty.
18. Return `W` — the `ef` closest elements found in this layer.

---

## **Questions & Answers / Assumptions**


### **Q1. What is `ep` here? Is it a single node or a list?**

* Conceptually, `ep` can be multiple entry points, but in practice, it’s usually **one node**.
* Initialized to start the search from the topmost or current entry point.

---

### **Q2. How do we get the furthest element (`f`) from `W`?**

* It’s the node in `W` with the **largest distance to `q`**.
* Often implemented as a **max-heap** for efficiency.

---

### **Q3. Why break if `distance(c,q) > distance(f,q)`?**

* Because **all remaining candidates in `C` are farther than the worst in `W`**.
* No need to continue; we already have the best `ef` neighbors.

---

### **Q4. Why add `e` to `C` and `W` only if closer than `f` or `|W| < ef`?**

* Ensures **W always holds the top `ef` closest nodes**.
* If W isn’t full yet, we add candidates even if they’re not closer than current worst.

---

### **Q5. How does this relate to the INSERT algorithm?**

* INSERT calls `SEARCH-LAYER` at each layer to **find the closest neighbors** before establishing connections.
* The output `W` is used to select neighbors for the new node.
* `ep` in INSERT is updated per layer with the closest node from `W`.

---



### Algorithm 3:

```
SELECT-NEIGHBORS-SIMPLE(q, C, M)

Input: base element q, candidate elements C, number of neighbors to return M

Output: M nearest elements to q
return M nearest elements from C to q

```

### Breakdown:

Algorithm 3 takes a base node `q` and a set of candidate nodes `C`, then simply selects the `M` closest nodes to `q` based on distance. It does not perform any additional checks or loops beyond evaluating distances, making it a straightforward nearest-neighbor selection.




### Algorithm 4

```
SELECT-NEIGHBORS-HEURISTIC(q, C, M, lc, extendCandidates, keepPrunedConnections)
Input: base element q, candidate elements C, number of neighbors to
return M, layer number lc, flag indicating whether or not to extend
candidate list extendCandidates, flag indicating whether or not to add
discarded elements keepPrunedConnections
Output: M elements selected by the heuristic
1 R ← ∅
2 W ← C // working queue for the candidates
3 if extendCandidates // extend candidates by their neighbors
4   for each e ∈ C
5     for each eadj ∈ neighbourhood(e) at layer lc
6       if eadj ∉ W
7         W ← W ⋃ eadj
8 Wd ← ∅ // queue for the discarded candidates
9 while │W│ > 0 and │R│< M
10  e ← extract nearest element from W to q
11  if e is closer to q compared to any element from R
12    R ← R ⋃ e
13  else
14    Wd ← Wd ⋃ e
15 if keepPrunedConnections // add some of the discarded connections from Wd
16  while │Wd│> 0 and │R│< M
17    R ← R ⋃ extract nearest element from Wd to q
18 return R

```
---

### **Breakdown**

1. **`R ← ∅`**: Initialize the result set. This will eventually hold the `M` neighbors selected by the heuristic.
2. **`W ← C`**: Initialize a working queue `W` with the candidate elements `C`. This queue is ordered by distance to `q` (nearest first).
3. **`if extendCandidates`**: Optional step to expand the candidate pool by including neighbors of the candidates.

   * Loop through each candidate `e` in `C`.
   * For each neighbor `eadj` of `e` in layer `lc`:

     * If `eadj` is not already in `W`, add it to `W`.
   * This step helps capture nearby clustered nodes that may otherwise be missed.
4. **`Wd ← ∅`**: Initialize a discard queue to temporarily hold candidates that don’t meet the heuristic at first.
5. **`while |W| > 0 and |R| < M`**: Continue selecting neighbors until either `W` is empty or `R` reaches size `M`.

   * Extract the closest element `e` from `W` (nearest to `q`).
   * **Line 11 check**:

     * If `e` is closer to `q` than **any element currently in `R`**, add `e` to `R`.
     * Otherwise, move `e` to the discard queue `Wd`.
   * Note: For the first iteration, `R` is empty, so the first element always enters `R`.
6. **`if keepPrunedConnections`**: Optional step to recover some discarded neighbors.

   * While `Wd` is not empty and `R` has fewer than `M` elements:

     * Extract the nearest element from `Wd` and add it to `R`.
   * This ensures that `R` still reaches the required number of neighbors while preserving some pruned candidates for connectivity.
7. **`return R`**: Output the final set of `M` neighbors selected according to the heuristic.

---

### **Notes / Observations**

* `R` ensures the selected neighbors are **diverse** and not overly clustered, while still prioritizing proximity to `q`.
* The **heuristic pruning** balances **proximity** (distance to `q`) and **diversity** (avoiding connections that are too similar to already selected neighbors).
* `Wd` acts as a **backup queue**: nodes discarded during pruning can be added later if `keepPrunedConnections` is enabled, ensuring each node still reaches the desired number of neighbors.
* `extendCandidates` is rarely needed, only for **extremely clustered data**, as it expands the candidate pool to avoid local optima.
* `keepPrunedConnections` guarantees that `R` reaches the required size `M` even if some candidates were initially pruned.
* On the first iteration, `R` is empty, so the **closest candidate always enters**. Subsequent selections enforce diversity among neighbors.

---


### Algorithm 5

```
K-NN-SEARCH(hnsw, q, K, ef)
Input: multilayer graph hnsw, query element q, number of nearest
neighbors to return K, size of the dynamic candidate list ef
Output: K nearest elements to q
1 W ← ∅ // set for the current nearest elements
2 ep ← get enter point for hnsw
3 L ← level of ep // top layer for hnsw
4 for lc ← L … 1
5   W ← SEARCH-LAYER(q, ep, ef=1, lc)
6   ep ← get nearest element from W to q
7 W ← SEARCH-LAYER(q, ep, ef, lc =0)
8 return K nearest elements from W to q

```

---

### **Breakdown**

1. **`W ← ∅`**: Initialize an empty list/queue `W` that will hold the current nearest neighbors found during the search.

2. **`ep ← get enter point for hnsw`**: Start the search from the global entry point of the HNSW graph (the topmost node).

3. **`L ← level of ep`**: Record the top layer number; this is the starting layer for the search.

4. **Loop from `L` down to 1 (`for lc ← L … 1`)**: Perform a **greedy search** at each upper layer to quickly get close to the query `q`.

   * **Line 5**: `W ← SEARCH-LAYER(q, ep, ef=1, lc)`

     * Run a greedy search in the current layer using `ef=1` to find candidate neighbors near `q`.
     * Only the closest element really matters at this stage (speed over quality).
   * **Line 6**: `ep ← get nearest element from W to q`

     * Update `ep` to the closest candidate found; this becomes the entry point for the next lower layer.

5. **`W ← SEARCH-LAYER(q, ep, ef, lc=0)`**: At the **bottom layer** (ground layer), perform a **full search** with the user-specified `ef`.

   * This is where quality matters: we use `ef` to ensure a good approximation of the true K nearest neighbors.

6. **Return K nearest elements from W**: Extract the top `K` closest elements from `W` as the final search result.

---

### **Notes / Observations**

* The **upper layers (L … 1)** are used for **coarse navigation**, quickly moving toward the region of interest in the graph.
* The **bottom layer (0)** is where **high-quality neighbor selection** happens. This is why we use the full `ef` parameter here.
* `ep` is **temporarily updated** at each layer to point to the best candidate from the previous layer; the global entry point remains unchanged.
* The algorithm mimics **INSERT’s first phase** but without actually inserting a new element. You’re just finding nearest neighbors.
* It seems like ef in K-NN-SEARCH could be treated as optional, defaulting to efConstruction if the user doesn’t provide a value. This way, the search would automatically match the recall used during insertion, while still allowing users to override it for custom search precision or speed.

---


## References:

- https://arxiv.org/pdf/1603.09320
- https://keyurramoliya.com/posts/Understading-HNSW-Hierarchical-Navigable-Small-World/
