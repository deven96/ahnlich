---
slug: choosing-an-ordered-index-for-ahnlich
title: "Choosing an ordered index for Ahnlich: when BTreeSet beats SkipSet"
authors: [osobotu, diretnan]
tags: [ahnlich, performance, rust]
image: /img/blog/ahnlich-vectordbz-demo.gif
---

We recently added `ListStoreEntries`, an API that lists every entry in an Ahnlich store. Stable pagination requires a consistent order, but Ahnlich stores entries in a hash map optimized for direct lookup. We therefore needed a separate ordered index to track the entries of a store. We compared Rust's `BTreeSet`, protected by a read-write lock, with the concurrent `SkipSet` from `crossbeam-skiplist`. For Ahnlich's current workload, `BTreeSet` made page reads 6 to 31 times faster and used about one-third as much memory. `SkipSet` performed better when four writers competed for the same index.

![Browsing an Ahnlich store through VectorDBZ](/img/blog/ahnlich-vectordbz-demo.gif)

*Browsing and managing an Ahnlich store through VectorDBZ.*

<!-- truncate -->

## Pagination needs order

[Ahnlich](https://ahnlich.dev/) is an open-source vector database that stores vector embeddings and metadata. A vector embedding is a list of numbers that represents an item and allows you to perform similarity search. You can create a store of vectors in Ahnlich and use the `ListStoreEntries` API to list all the entries in the store with cursor pagination. If a page ends at entry ID `9`, the next request asks for the entry IDs after `9`.

Suppose a store contains these entry IDs:

```text
18, 4, 27, 9, 12
```

A hash map can return these entry IDs in a different order after an insertion or deletion. As a result, a client can receive one entry twice or miss another entry. Using numeric offsets still requires a stable order because "skip the first 100 entries" only makes sense when the first 100 have a stable order.

By contrast, sorting the entry IDs gives the cursor a precise meaning:

```text
4, 9, 12, 18, 27
```

The two entry IDs after `9` are always `12` and `18`, so the client can use `18` as its next cursor.

For that reason, Ahnlich keeps the hash map and adds an ordered index that contains only entry IDs. The ordered index selects the IDs for a page, and the hash map returns the corresponding vectors and metadata.

## Two ways to keep IDs sorted

Rust's standard-library [`BTreeSet`](https://doc.rust-lang.org/std/collections/struct.BTreeSet.html) stores sorted keys in tree nodes. Ahnlich wraps the set in `parking_lot::RwLock`, which allows multiple readers or one writer to access it at a time. [`SkipSet`](https://docs.rs/crossbeam-skiplist/latest/crossbeam_skiplist/struct.SkipSet.html) is a concurrent skip list. Its keys live in linked, sorted nodes at several levels, allowing readers and writers to access the collection without a single lock around the entire set.

At first, `SkipSet` appeared better suited for a database because it supports concurrent changes. However, that concurrency requires more allocations and pointers. `BTreeSet` is simpler and more compact, but writers can wait when readers keep the lock busy.

Both structures provide `O(log n)` search, insertion, and deletion. However, the Big-O complexity does not show the cost of page traversal, memory use, or lock contention. Therefore, we benchmarked the operations that Ahnlich would perform.

We tested pseudorandom entry IDs generated from a fixed seed in stores of up to one million entries. We read pages of 100 and 1,000 IDs from the start, middle, and end of each index. In addition, we measured bulk changes, allocated memory, and three-second contention runs with up to 32 readers and four writers.

We ran the benchmarks on a 13th Gen Intel Core i7-1355U with 10 cores, 12 threads, and 15 GiB of RAM. The machine used Ubuntu 24.04 and Rust 1.96.0. Because these are microbenchmarks, the absolute times will change on other machines. The comparisons show the relative cost of each structure for this workload.

## BTreeSet was faster for page reads

At one million IDs, `BTreeSet` returned pages faster at every cursor position we tested. The chart shows the speedup at the middle cursor.

![At the middle cursor, BTreeSet was 6.2 times faster for pages of 100 IDs and 10.2 times faster for pages of 1,000 IDs.](/img/blog/ordered-index-read-latency.svg)

For a page of 100 IDs, `BTreeSet` took 0.95 microseconds while `SkipSet` took 5.86 microseconds. When the page size increased to 1,000, the medians were 5.67 and 58.0 microseconds. Across the start, middle, and late cursor positions, the tree was 6 to 31 times faster.

Memory layout might explain part of the difference. A B-tree stores several nearby keys in each node, which can improve cache locality during sequential reads. A skip list follows pointers between separately allocated nodes and stores additional metadata to support concurrent updates. We did not measure cache misses, so these benchmarks do not establish the cause.

The same pattern appeared in bulk writes. Inserting 100,000 IDs took 17.06 milliseconds with `BTreeSet` and 54.31 milliseconds with `SkipSet`. Deleting the same number took 14.30 and 49.69 milliseconds.

`BTreeSet` also used less memory. At one million IDs, the tree used 10.18 bytes per ID while the skip list used 31.99 bytes per ID.

The ordered index repeats entry IDs that already exist in the hash map. At this scale, the difference adds roughly 22 MB for each million-entry store. Therefore, memory use mattered as much as latency in this decision.

## SkipSet won under write pressure

Our initial contention test had one writer, so `SkipSet` had little opportunity to use its concurrency model. To test its intended workload, we made several writers repeatedly insert and remove individual IDs while readers requested pages of 100.

With one reader and one writer, `BTreeSet` still led both reader and writer throughput. However, the results changed as the number of writers increased. To compare the slower operations, we measured p99 writer latency. In each run, 99% of writer operations completed at or below this value. The chart uses a logarithmic scale, where each vertical step is ten times larger.

![Writer p99 latency with four writers and increasing reader counts. SkipSet keeps writer latency lower under contention.](/img/blog/ordered-index-writer-p99.svg)

At 32 readers and four writers, `SkipSet` completed 569,339 writer operations per second. `BTreeSet` completed 60,137. The `SkipSet` writer p99 was 81.84 microseconds, while `BTreeSet` reached 3,725.18 microseconds.

`BTreeSet`, however, remained faster for readers. In the same run, it completed 2,066,498 reader operations per second while `SkipSet` completed 319,374. A steady stream of readers could keep using the shared lock, which left writers waiting for exclusive access. The skip list traded some read speed and memory for steadier writer progress.

These results show where `SkipSet` earns its overhead. It is a better fit when several independent writers change the same ordered collection and writer latency matters more than page-read speed.

## Filter selectivity changed the fastest strategy

An ordered index solves unfiltered pagination. However, metadata filters create another choice. Ahnlich calls these filters predicates, and selectivity is the percentage of store entries that match a filter.

For example, imagine a million-entry store where a filter matches 100 entries. Ahnlich can collect those 100 entry IDs, sort them, and return the requested page. Scanning a million-entry index to find the same sparse matches would do far more work.

Conversely, if the filter matches nearly every entry, Ahnlich can walk the ordered index and stop when the page is full. Sorting hundreds of thousands of matches would waste time.

We measured both strategies on a one-million-entry store. The chart uses a page size of 100 and logarithmic axes.

![Latency of sorting filter matches versus scanning a BTreeSet as filter selectivity increases. Sorting wins for sparse matches and scanning wins for broad matches.](/img/blog/list-store-entries-selectivity.svg)

At 0.01% selectivity, sorting the 100 matches took 0.66 microseconds, while scanning the tree took 9.58 milliseconds. By contrast, at 100% selectivity, sorting took 23.633 milliseconds while the tree filled the page in 2.540 microseconds.

The two strategies were close around 1% for a page of 100: 142.99 microseconds to sort and 167.25 microseconds to scan. However, a page of 1,000 moved the crossover. Therefore, a fixed selectivity percentage cannot choose the strategy by itself. The decision must consider both the number of matches and the requested page size.

## Why Ahnlich uses BTreeSet

Ahnlich maintains the ordered index as part of the same store operations that update the primary hash map. In clustered deployments, Raft applies committed mutations in the same order on every node. Therefore, the ordered index does not need a lock-free structure designed primarily for many independent writers.

Standalone deployments can still process concurrent writes, so the multi-writer benchmark remains a useful stress test. However, for the read-heavy pagination workload performed by `ListStoreEntries`, `BTreeSet` gave us faster page reads, lower memory use, and simpler standard-library code. We therefore chose it for the ordered entry ID index.

This decision can change. If Ahnlich later allows many independent writers to update the same store concurrently, the lock can become a source of writer latency. The contention benchmark gives us a specific reason to measure again if that architecture changes.

We also kept the filtered pagination strategy separate from the index implementation. Sparse filters can sort their small match set. By contrast, broad filters can scan the ordered index and stop at the page limit. This separation lets us replace either data structure without rewriting the pagination API.

## Run the benchmarks

The benchmark sources are in [`ahnlich/db/benches/experiments`](https://github.com/deven96/ahnlich/tree/main/ahnlich/db/benches/experiments). From the repository's `ahnlich` directory, run:

```bash
cargo bench -p db --bench ordered_store_key_index
cargo bench -p db --bench ordered_store_key_index_memory
cargo bench -p db --bench ordered_store_key_index_contention
cargo bench -p db --bench list_store_entries_strategy
```

Criterion writes the timing reports to `target/criterion/report/index.html`. Change the store size, page size, and reader-to-writer ratio to match your application. If your results differ from ours, use yours. The right ordered index depends on the traffic that will reach it.
