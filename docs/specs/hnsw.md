# Thoughts and potential Spec on HNSW algorithm

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

  * [Algorithm 6 — Delete](#algorithm-6)

    * [Breakdown](#breakdown-5)

  * [Algorithm 7 — Filtered Search](#algorithm-7)

    * [Breakdown](#breakdown-6)

* [Data Model & API Interface](#data-model--api-interfaces)

* [Needs Further Research / Open Questions](#needs-further-research--open-questions)

  
* [Testing Strategy](#testing-strategy)
    * [Correctness Testing (Core Validation)](#1-correctness-testing-core-validation)
        * [Linear Scan Baseline (Required, V1)](#11-linear-scan-baseline-required-v1)
        * [FAISS HNSW Comparison (Required, Optional Output Check)](#12-faiss-hnsw-comparison-required-optional-output-check)
        * [Filtered Recall (Required for alg. 7)](#13-filtered-recall-required-for-alg-7)
        * [Optional / Advanced Correctness Checks](#14-optional--advanced-correctness-checks)
    * [Determinism in a Replicated System](#2-determinism-in-a-replicated-system)
    * [Performance Testing](#3-performance-testing)
        * [Speed Benchmarks](#31-speed-benchmarks)
        * [Memory Usage](#32-memory-usage)
        * [Testing Flow Summary](#testing-flow-summary)


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
how many neighbors each element can maintain in layers above level 0.


- `mL`: a normalization param that controls the expected number of layers in the hierarchical structure. It plays a crucial role in balancing the tradeoff between search
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
  - `mL`: `1/ln(M)` 




## Others:
  ### Memory and Construction Complexity
  - Memory Complexity: Memory consumption for hnsw is directly proportional to the dataset size. Meaning it scales
  linearly. Total memory requirement can be expressed approximately as: 
    `(Mmax0 + mL * Mmax) * bytes_per_link per element` where:
      
    Mmax: the number of connections allowed at layers above level 0.

    **NOTE: for typical paremeter settings, this is roughly 60-450bytes per element for the graph structure alone, excluding the storage
    required for the original vector data**
  

  - Construction Complexity: Theoretically, it is `O(N log N)` for construction on relatively low-dimensional data. Where N represents the number of elements.







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
11  add bidirectional connections from neighbors to q at layer lc

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

### Algorithm 6

```
DELETE(hnsw, nodeId)
Input: multilayer graph hnsw, NodeId of element to remove
Output: update hnsw by removing the node and cleaning neighbor references

1 n ← hnsw.nodes.get(nodeId) // fetch the node to delete
2 for each r ∈ n.back_links
3    rNode ← hnsw.nodes.get(r) // fetch referring node
4    for each lc ∈ rNode.neighbours.keys()
5        if nodeId ∈ rNode.neighbours[lc]
6            remove nodeId from rNode.neighbours[lc] // remove reference
7 for each lc ∈ n.neighbours.keys()
8    remove nodeId from hnsw.graph[lc] // remove from layer set
9 remove nodeId from hnsw.nodes // remove the node itself
```

----

> **Note on Insertion for Back-link Deletions:**
To support efficient back-link-based deletions, the **INSERT** procedure would include an extra step: for each neighbor e that q connects to, add q to e’s back-links.
----

### Breakdown

1. **Fetch node**: Retrieve the Node struct corresponding to `nodeId` from `hnsw.nodes`. This is the node we want to delete.

2. **Iterate over back-links/referrals (`r`)**:

   * back-links represent nodes that consider the deleted node as a neighbor.
   * For each referring node, fetch its Node struct.

3. **Remove deleted node from neighbor sets**:

   * For each layer `lc` of the referring node, check if the nodeId exists in its `neighbours[lc]`.
   * If it does, remove it.
   * This ensures no stale references remain in the graph.

4. **Remove node from graph layers**:

   * Iterate over the layers where the deleted node exists (`n.neighbours.keys()`).
   * Remove `nodeId` from the corresponding `hnsw.graph[lc]` sets.
   * This ensures the layer-level index remains consistent.

5. **Remove node from nodes map**:

   * Finally, remove `nodeId` from `hnsw.nodes`, fully purging the node from the HNSW structure.

---

### **Rationale**:

* Deletion is **directed**, not global: we only touch nodes that reference the deleted node, avoiding a full traversal of the graph.
* Layer information is naturally captured via `neighbours.keys()`, so no extra layer tracking in referrals is needed.
* Complexity is O(M × R) where M = max neighbors per layer, R = number of referrals/back-links, making deletion practical.
* After deletion, the `graph` remains consistent: no node ID appears in any layer sets for the deleted node.

---


### Algorithm 7

> **This algorithm is ours, not the paper's.** The paper does not cover filtered/predicate-constrained
> search at all — algs. 1-6 assume every node in the graph is a valid answer. We need it because a
> `GETSIMN` can arrive with a predicate condition attached, so only the subset of nodes matching that
> predicate may be returned. Everything below is an extension on top of alg. 5 and alg. 2.

The input is an `acceptList`: the set of NodeIds whose vectors satisfy the predicate. It is computed by
the predicate indices *before* we ever touch the graph, so by this point the filter is just a set membership test.

```
FILTERED-SEARCH(hnsw, q, K, ef, acceptList)
Input: multilayer graph hnsw, query element q, number of nearest neighbors to
return K, size of the dynamic candidate list ef, set of allowed NodeIds acceptList
Output: K nearest elements to q that are members of acceptList

1 if acceptList = ∅
2   return ∅                          // predicate matched nothing, no work to do
3 if │acceptList│ ≤ bruteForceThreshold
4   return K nearest elements to q from acceptList   // exact scan, skip the graph entirely
5 return K-NN-SEARCH-FILTERED(hnsw, q, K, ef, acceptList)   // alg. 5, with alg. 2 filtered
```

`K-NN-SEARCH-FILTERED` is **alg. 5 unchanged**, except that the layer-0 call passes `acceptList` down
into SEARCH-LAYER. The upper layers (lc ← L … 1) are called with **no** acceptList.

`SEARCH-LAYER` is **alg. 2 unchanged**, except for two lines:

```
SEARCH-LAYER-FILTERED(q, ep, ef, lc, acceptList)
  ... identical to alg. 2 ...
7   if distance(c, q) > distance(f, q) and │W│ = ef      // <- CHANGED: the │W│ = ef guard
8     break
  ...
13  if distance(e, q) < distance(f, q) or │W│ < ef
14    C ← C ⋃ e                                          // e always joins the candidates
15    if e ∈ acceptList                                  // <- CHANGED: but only joins W if allowed
16      W ← W ⋃ e
17      if │W│ > ef
18        remove furthest element from W to q
19 return W
```

### Breakdown

1. **Empty acceptList** → return immediately. The predicate matched nothing, so no vector can be an answer.

2. **Small acceptList (line 3)**: if the predicate is selective enough, we do not touch the graph at all.
We just compute the distance from `q` to each of the accepted vectors and take the K nearest. This is
**exact**, not approximate, and at this size it is cheaper than a graph walk. See Q3 for why the graph walk
is *especially* bad in exactly this case.

3. **Otherwise (line 5)**: run alg. 5 as normal, and filter inside alg. 2. This is the important part.

4. **Line 14 — `e` always joins `C`.** A node that fails the predicate is still a legitimate *stepping stone*.
The graph's navigability does not know or care about our predicate; the small-world property lives in the
edges. If we refused to traverse *through* rejected nodes, we would be walking a different (and disconnected)
graph than the one we built, and the search would strand itself. So rejected nodes are traversed through.

5. **Line 15-16 — `e` only joins `W` if it is in the acceptList.** `W` is the *result* set, so this is the
only place the predicate is allowed to have an opinion. This is the whole trick, and it is what the literature
calls **in-filtering** (filter during traversal) as opposed to **post-filtering** (search, then discard).

6. **Line 7 — the `│W│ = ef` guard.** We only stop early once `W` is *full*. This matters here in a way it
never did for unfiltered search: `W` now only accepts matching nodes, so under a selective predicate it fills
*slowly*. Without the guard, `f` (the furthest in `W`) is a very tight bound while `W` is still nearly empty,
the stopping condition fires almost immediately, and the search gives up having found only a handful of
results. Unfiltered, `W` fills on the first expansion, so this was never observable.

7. **Upper layers are never filtered.** lc ← L … 1 is pure navigation with ef=1; its only job is to hand a
good entry point to layer 0. Filtering there would mean the descent could fail to find *any* acceptable node
at some upper layer and strand itself, when all it was ever supposed to do was get us to the right
neighbourhood. Only layer 0 collects results, so only layer 0 filters.

---

## **Questions & Answers/Assumptions Made**

### **Q1. Why not just search normally and drop the non-matching results afterwards?**

That is post-filtering, and it is what we did originally. It does not work:

* If the accepted vectors are all far from `q`, the top-K are all rejects and you return **nothing** —
  even though plenty of vectors satisfied the predicate. The results are silently wrong, not just badly ordered.
* The only way to defend against that with post-filtering is to over-search. But there is no over-search
  factor that is *safe*: in the worst case the accepted vectors are the K **furthest** from `q`, so you would
  have to retrieve the entire store to be sure of finding them.

Post-filtering cannot give a guarantee at any price. In-filtering can, because `W` never fills with rejects
in the first place.

---

### **Q2. Should `ef` be scaled up to compensate for the filter?**

**No, and this is the trap.** `ef` is a quality/latency knob (paper §4, "the quality of the search is controlled
by ... ef"). It is a *configured constant*, not a function of the data. Tying `ef` to `│acceptList│` means the
breadth of a search is decided by how many rows the WHERE clause happened to match — a loose predicate
(half the store) would drive `ef` into the tens of thousands and traverse essentially the whole graph to return
10 results, which is slower than the linear scan HNSW exists to beat.

`K` is what the caller asked for. `ef` is what we configured. Neither is derived from the predicate.

---

### **Q3. Then what happens when the predicate is very selective?**

In-filtering degrades here, and honestly so: `W` can only fill with accepted nodes, so if very few nodes are
acceptable, `W` never reaches `ef`, the stop condition (line 7) never fires, and the search keeps widening —
approaching a full traversal to return a handful of results. The visited count grows roughly as `ef / selectivity`.

That is exactly why line 3 exists. Below the threshold we abandon the graph and scan the accepted set directly:
it is O(│acceptList│) distance computations, it is exact, and it is bounded. The graph is the wrong tool when
the candidate set is already small.

---

### **Q4. How is the bruteForceThreshold chosen?**

Currently a constant (4096). This is a **weak point and an open question** — see the open questions below.
It should probably be relative to the size of the store, not absolute, since "selective" is a ratio, not a count.

---

### **Q5. Does this stay deterministic across replicas?**

It must, per the determinism requirements in the testing strategy. Two places to be careful:

* The brute-force path sorts by distance. Equal distances must **not** tie-break on hash-set iteration order.
  We break ties on NodeId, which is a content hash and therefore stable across processes and restarts.
* The acceptList itself is a set of NodeIds, so it is identical on every replica given the same data and the
  same predicate.

---

### **Notes / Observations**

* The acceptList is keyed by NodeId (u64), not by the vector, so membership is a single integer hash lookup on
  the hot path and we never re-hash a 1024-dim vector during traversal.
* Recall under a filter is *not* the same quantity as recall without one. The ground truth for a filtered query
  is the true K nearest **among the accepted set**, i.e. brute force restricted to the acceptList. Any recall
  test for this path must compare against that, not against the unfiltered brute force.
* This algorithm has a hard requirement that alg. 2's `W` only ever contains accepted nodes — the stopping
  condition reads `W`, so a single reject leaking into `W` corrupts the bound and silently truncates the search.

---




## Data Model & API Interfaces:

WIP: A simple HNSW structure
```rust
use std::collections::{HashSet, btree_map::BTreeMap};

/// LayerIndex is just a wrapper around u8 to represent a layer in HNSW.
pub struct LayerIndex(u8);

/// NodeId wraps u64(hash of node embeddings) to uniquely identify a node across all layers.
pub struct NodeId(u64);


/// HNSW represents a Hierarchical Navigable Small World graph.
///
/// The graph is organized into multiple layers. Each layer contains a set of node IDs,
/// and each node holds its neighbors per layer along with its embedding vector.
/// This separation allows efficient lookups, prevents duplicate nodes per layer,
/// and supports deletion operations.
///
/// Design rationale:
/// 1. `nodes` is the single source of truth: all Node structs live here, keyed by NodeId.
/// 2. `graph` maps each layer to a `HashSet` of NodeIds, ensuring uniqueness per layer
///    and fast removal when deleting nodes.
/// 3. Deletion is fully supported:
///    - Remove the node ID from the `graph` for all layers where it exists.
///    - Remove the node from `nodes`.
///    - Remove the node ID from all neighbors of other nodes (using back-links/referrals).
///      This ensures no stale references remain in the graph.
///
/// Example of usage:
/// ```text
/// Layer 0: {42, 10, 55}
/// Layer 1: {42, 11, 9}
/// Layer 2: {42, 88}
/// Layer 3: {42, 200, 201}
///
/// Node 42 participates in layers 0–3, with neighbors stored per layer and
/// back-links automatically updated upon deletion.
/// ```
pub struct HNSW {
    /// Breadth of search during insertion (efConstruction)
    pub ef_construction: Option<u8>,

    /// Top-most layer index in the graph (L)
    pub top_most_layer: u8,

    /// Maximum number of connections per node (M)
    pub maximum_connections: u8,

    /// Precomputed value 1 / ln(M) used in level generation
    pub inv_log_m: f64,

    /// Nodes in each layer
    ///
    /// Each layer index maps to a set of NodeIds.
    /// This ensures uniqueness per layer and allows easy removal during deletion.
    graph: BTreeMap<LayerIndex, HashSet<NodeId>>,

    /// All nodes in the HNSW
    ///
    /// The single source of truth for all node data.
    /// Keys are NodeId, values are the Node structs containing embeddings and neighbors.
    nodes: HashMap<NodeId, Node>,
}

/// Node represents a single element in the HNSW graph.
///
/// Each node stores:
/// - `id`: unique identifier
/// - `value`: embedding vector
/// - `neighbours`: map from layer to set of NodeIds of neighbors in that layer
/// - `back_links`: set of NodeIds of nodes that consider us a neighbor.
///   Used to efficiently update the graph when deleting this node.
///
/// Example of a node:
/// ```text
/// Node {
///     id: 42,
///     value: [0.12, 0.55, 0.77],
///     neighbours: {
///         0: [10, 55, 71],
///         1: [9, 11],
///         2: [88],
///         3: [200, 201]
///     },
///     back_links: [9, 88]
/// }
/// ```
/// This shows that Node 42 participates in layers 0 through 3.
pub struct Node {
    id: NodeId,
    value: Vec<f64>,
    neighbours: BTreeMap<LayerIndex, HashSet<NodeId>>,
    back_links: HashSet<NodeId>,
}

```

### API Interface

#### HNSW
```rust
impl HNSW {
    /// Insert a new element into the HNSW graph
    /// Corresponds to Algorithm 1 (INSERT)
    pub fn insert(&mut self, q: Vec<f64>) -> NodeId {
        // internally uses SEARCH-LAYER, SELECT-NEIGHBORS
        todo!()
    }

    /// Search for ef nearest neighbors in a specific layer
    /// Corresponds to Algorithm 2 (SEARCH-LAYER)
    pub fn search_layer(
        &self,
        query: &Vec<f64>,
        entry_points: &[NodeId],
        ef: usize,
        layer: LayerIndex,
    ) -> Vec<NodeId> {
        todo!()
    }

    /// Select M neighbors simply based on distance
    /// Corresponds to Algorithm 3 (SELECT-NEIGHBORS-SIMPLE)
    pub fn select_neighbors_simple(
        &self,
        base: NodeId,
        candidates: &[NodeId],
        m: usize,
    ) -> Vec<NodeId> {
        todo!()
    }

    /// Select M neighbors using heuristic for diversity and pruning
    /// Corresponds to Algorithm 4 (SELECT-NEIGHBORS-HEURISTIC)
    pub fn select_neighbors_heuristic(
        &self,
        base: NodeId,
        candidates: &[NodeId],
        m: usize,
        layer: LayerIndex,
        extend_candidates: bool,
        keep_pruned_connections: bool,
    ) -> Vec<NodeId> {
        todo!()
    }

    /// K-Nearest Neighbor Search
    /// Corresponds to Algorithm 5 (K-NN-SEARCH)
    pub fn knn_search(
        &self,
        query: &Vec<f64>,
        k: usize,
        ef: Option<usize>,
    ) -> Vec<NodeId> {
        todo!()
    }

    /// Optional helper to get a node by NodeId efficiently
    pub fn get_node(&self, id: NodeId) -> Option<&Node> {
        todo!()
    }
}
```


#### Node
*Just helper methods*
```rust
impl Node {
    /// get neighbors at a specific layer
    pub fn neighbors_at(&self, layer: LayerIndex) -> Option<&HashSet<NodeId>> {
        self.neighbours.get(&layer)
    }

    /// add a neighbor at a specific layer
    pub fn add_neighbor(&mut self, layer: LayerIndex, neighbor: NodeId) {
        self.neighbours.entry(layer).or_insert(neighbor);
    }

    /// remove a neighbor at a specific layer
    pub fn remove_neighbor(&mut self, layer: LayerIndex, neighbor: NodeId) {
        if let Some(set) = self.neighbours.get_mut(&layer) {
            set.remove(&neighbor);
        }
    }

    // perhaps useful??
    pub fn closest_neighbor_to(&self, query: &Vec<f64>) -> NodeId {
      todo!()
    }
}

```


## Needs Further Research / Open Questions

- Construction complexity for high-dimensional data.

- Determinism in replicated environments: how similar would results be across multiple replicas for the same query?

- Default ef in K-NN-SEARCH: should it default to efConstruction if not provided?

- How do we define a default for this value if not provided by the user? If a wrong default is chosen, do we recreate after we understand the dimensionality of said data??
  > @deven96's comment: Yeah we already know the dimensionality of the data so what i propose we do is surface some options as Option but if the values are not provided then we compute our defaults

- **Filtered search (alg. 7): what should `bruteForceThreshold` actually be?** It is currently an absolute
  constant (4096), which is crude — "selective" is a ratio, not a count. An acceptList of 5,000 is a loose
  predicate on a 10k store and an extremely tight one on a 10M store, and only the second case wants the
  graph. Options: make it relative (`max(4096, N/100)`), derive it from a cost model (estimated distance
  computations for the scan vs. the expected `ef / selectivity` visits for the walk), or surface it as config.
  Needs measurement across selectivities before we pick.

- **Filtered search: should we cap the visited-node count?** At low selectivity but above the threshold,
  in-filtering can still approach a full traversal. A visited cap would bound the latency at the cost of recall.
  We have no data yet on where that trade sits, and a silent recall cliff is worse than a slow query, so it is
  deliberately not implemented.

- **Filtered search: is `ef` still the right knob under a filter?** `ef` accepted nodes is a different amount of
  work than `ef` nodes. It may be that filtered queries want their own `ef` default. Open.



## **Testing Strategy**


To properly validate our HNSW implementation, we’ll structure testing into three main layers:

1. **Correctness** – ensures our ANN returns neighbors close to the true nearest neighbors.
2. **Determinism** – ensures replicated systems produce consistent results.
3. **Performance** – measures efficiency and guides optimization.

Each layer targets a different class of issues, together ensuring a **robust and production-ready implementation**.

---

### **1. Correctness Testing (Core Validation)**

The primary goal is to ensure our HNSW implementation returns neighbors as close as possible to the **true nearest neighbors**, measured using **Recall@K**.

**Recall@K** is defined as:
```r
Recall@K = (# of true neighbors returned in top K results) / K
```

* **K** = number of neighbors requested per query (should reflect typical application queries)
* **True neighbors** = nearest neighbors obtained via a brute-force linear scan
* **Range:** 0–1 (or 0%–100%)

High Recall@K indicates the approximate HNSW search is capturing most of the actual nearest neighbors.

Testing correctness is done in **two phases**:

---

#### **1.1. Linear Scan Baseline (Required, V1)**

Compare our HNSW implementation against a **brute-force KNN (linear scan)**, which serves as the ground truth. ([https://github.com/nmslib/hnswlib/blob/master/TESTING_RECALL.md](https://github.com/nmslib/hnswlib/blob/master/TESTING_RECALL.md))

**Procedure:**

1. Generate a dataset (synthetic or real, e.g., SIFT1M).
2. Build our HNSW index and a brute-force index.
3. Query each vector for `k` nearest neighbors.
4. Compute **Recall@K**:

```r
Recall@K = (# correct neighbors returned by HNSW) / K
```

**Validates:**

* Layer traversal
* Neighbor selection & pruning logic
* Candidate queue behavior
* Distance metric correctness

💡 This is the **industry-standard recall validation**.

---

#### **1.2. FAISS HNSW Comparison (Required, Optional Output Check)**

Compare our HNSW implementation against **FAISS’s HNSW** on the same dataset.

* **Purpose:** Not to copy FAISS’s implementation, but to obtain a **production-level baseline for recall**.
* FAISS outputs are **not ground truth**, but they serve as a widely-used reference for recall performance.
* Helps identify potential blind spots and performance differences.

**Procedure:**

1. Build a FAISS HNSW index on the same dataset.
2. Query for `k` nearest neighbors.
3. Compare neighbor overlap and Recall@K with our implementation.

---

#### **1.3. Filtered Recall (Required for alg. 7)**

Filtered search needs its own recall test, because **the ground truth is a different quantity**. For a filtered
query the true answer is the K nearest **among the accepted set** — i.e. brute force restricted to the
acceptList — *not* the K nearest overall. Comparing a filtered result against the unfiltered brute force will
look like a recall collapse even when the implementation is perfect.

```r
FilteredRecall@K = (# of true accepted-neighbors returned in top K) / K
    where true accepted-neighbors = brute-force KNN over acceptList only
```

**Procedure:**

1. Build an index over a dataset.
2. Pick an acceptList (a predicate) and sweep its **selectivity**: e.g. 100%, 50%, 10%, 1%, 0.1%.
3. Ground truth: brute-force KNN restricted to the acceptList.
4. Compare, and record both recall **and** latency at each selectivity.

**The two cases that must be covered, because both were live bugs:**

* **Adversarial placement (correctness).** Construct the acceptList so the accepted vectors are *far* from the
  query and the rejected ones are *near* it (e.g. two separated clusters, query sitting in the rejected cluster,
  predicate matching only the far cluster). A post-filtering implementation returns **zero** results here while
  hundreds of vectors satisfy the predicate. This is the regression test for the original defect.
* **Loose predicate (latency).** acceptList = ~half the store, K = 10. This must be *fast*: nearly everything
  near the query is acceptable, so it should cost about what an unfiltered search costs. An implementation that
  scales `ef` with `│acceptList│` blows up here — traversing the whole graph to return 10 rows, i.e. worse than
  the linear scan HNSW is supposed to beat.

> Latency must be asserted alongside recall for this path. A filtered search that is correct but traverses the
> entire graph is still a failure — it silently turns the ANN index back into a linear scan.

---

#### **1.4. Optional / Advanced Correctness Checks**

* **Sanity tests:** simple vectors, identical vectors, widely separated points.
* **Structural integrity tests:** no duplicate neighbors, bidirectional edges, valid levels.
* **Dense cluster stress tests:** test candidate pruning and heuristics under tightly clustered vectors.

> These tests are optional but provide additional confidence in the implementation. They can be run periodically or on stress-test datasets.

---

### **2. Determinism in a Replicated System**

For distributed or replicated deployments, **determinism is critical**.

**Procedure:**

1. Insert **exact same items in the exact same order** across two nodes.
2. Compare:

   * Complete adjacency lists
   * Layer assignments
   * Enter point
   * Number of neighbors per node

**Checks for:**

* Floating-point nondeterminism
* Unstable sorting
* Concurrency issues
* Randomness not properly seeded

> Ensures queries produce consistent results across replicas.

---

### **3. Performance Testing**

Once correctness and a recall baseline are established, performance testing guides **optimization and efficiency improvements**.

#### **3.1. Speed Benchmarks**

Measure **insertion and search speed** at varying dataset sizes:

* 10k, 100k, 1M vectors
* Track impact of HNSW parameters: `M`, `Mmax0`, `efConstruction`, `ef`

  **Note on ef vs efConstruction:**

  **efConstruction** → controls candidate exploration during insertion, affecting graph quality and build time.

  **ef** → controls candidate exploration during search/query, affecting recall and query speed.

  Increasing either parameter improves recall but slows down the respective operation.

* utilize SIMD acceleration for distance calculations (Euclidean, dot product, cosine similarity).


#### **3.2. Memory Usage**


Track **memory per node and per layer**:

* Total memory vs dataset size
* Effect of larger `M` values
* Impact of data layout optimizations (SoA vs AoS):  
Using a Structure of Arrays (SoA) can improve cache locality and SIMD efficiency for bulk distance computations, whereas Array of Structures (AoS) is simpler but less cache-friendly for vector-heavy operations

> Provides insight for optimizing both speed and memory footprint.

---

#### **Testing Flow Summary**

1. **Correctness Phase**: Compare our HNSW vs brute-force → establishes Recall@K baseline.
2. **Production Baseline Phase**: Compare our HNSW vs FAISS HNSW → confirms recall relative to a widely-used implementation.
3. **Determinism & Integrity Checks**: ensure replication-safe behavior.
4. **Performance Phase**: establish baseline speed/memory, then explore optimizations.
5. Test recall by potentially replicating the deletion experiment from the paper: randomly remove a percentage of nodes and reinsert them over multiple cycles, then observe whether search performance (recall) remains stable. 




## References:

- https://arxiv.org/pdf/1603.09320
- https://keyurramoliya.com/posts/Understading-HNSW-Hierarchical-Navigable-Small-World/
