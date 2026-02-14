---
slug: simd-optimization-vector-database
title: "Making Ahnlich Faster with SIMD: A 4.7x Speedup Story"
authors: [diretnan, davidonuh]
tags: [ahnlich, performance, simd, rust]
image: /img/blog/simd-hero.svg
---

Vector databases power modern search, from finding similar images to semantic document retrieval. But there's a hidden performance bottleneck: calculating distances between millions of high-dimensional vectors requires billions of operations per query. In this post, I'll show you how we used SIMD (Single Instruction, Multiple Data) to make [Ahnlich](https://github.com/deven96/ahnlich) **4.7x faster** at these calculations.

![SIMD Optimization Hero](/img/blog/simd-hero.svg)

<!-- truncate -->

## What's a Vector Database Anyway?

Most developers are familiar with traditional databases where you filter by exact criteria. A friend recently showed me his financial market data queries that looked like this:

```python
results = db.query()
    .with_text_search("tech stocks")
    .with_date_filter(start="2024-01-01", end="2024-12-31")
    .with_creator("Goldman Sachs")
    .with_soft_deletion(False)
    .limit(100)
```

He was chaining together filters trying to find relevant reports. The problem? He already knew what he wanted but still had to manually specify every constraint. What if he could just search "recent tech stock analysis from major banks" and get exactly what he needed?

That's where vector databases shine.

### Traditional Search vs Semantic Search

**Traditional databases** match exact criteria. You tell them exactly what to look for: specific dates, specific creators, specific keywords. It's like searching for your friend at a concert by checking "wearing a red shirt AND standing near the stage AND arrived at 7pm."

**Vector databases** work with embeddings. First, your data passes through a model (like BERT, OpenAI, or others) that converts text/images into vectors (lists of numbers representing semantic meaning). The vector database then stores these embeddings and finds items that are *similar* to your query based on mathematical distance, even if the words are completely different. Some databases like Ahnlich even let you specify the embedding model as an argument, handling the conversion automatically.

Let's visualize this with a simple example. Say we have three sentences converted to 2D vectors (real embeddings are 100s or 1000s of dimensions, but 2D is easier to see):

```
"cat sleeps"   → [0.77, 0.23]
"dog sleeping" → [0.7, 0.3]  
"I ate pizza"  → [0.1, 0.9]
```

Plotting these in 2D space:

![Vector 2D Plot](/img/blog/vector-2d-plot.svg)

When you query **"kitten napping" → [0.75, 0.25]**, the vector database calculates the distance to each stored vector and finds the closest matches. Even though "kitten napping" doesn't use the exact words "cat" or "dog," the semantic meaning places it close to those vectors.

### How Distance Calculations Work

Vector databases use mathematical distance metrics to measure similarity:

1. **Euclidean Distance** - Straight-line distance in n-dimensional space (lower = more similar)
2. **Cosine Similarity** - Angle between vectors, ignoring magnitude (1 = identical direction)
3. **Dot Product** - Measures alignment between vectors (higher = more similar)

Let's calculate Euclidean distance for our query:

```
Query: "kitten napping" [0.75, 0.25]

Distance to "cat sleeps" [0.77, 0.23]:
  = sqrt((0.75-0.77)² + (0.25-0.23)²) 
  = sqrt(0.0004 + 0.0004) 
  = 0.028 ✓ Closest match!

Distance to "dog sleeping" [0.7, 0.3]:
  = sqrt((0.75-0.7)² + (0.25-0.3)²) 
  = sqrt(0.0025 + 0.0025)
  = 0.071 ✓ Also close!

Distance to "I ate pizza" [0.1, 0.9]:
  = sqrt((0.75-0.1)² + (0.25-0.9)²) 
  = sqrt(0.4225 + 0.4225)
  = 0.92 ✗ Far away!
```

The database returns the top results: "cat sleeps" (closest) and "dog sleeping" (second closest). Perfect semantic search!

### The Performance Challenge

Here's the catch: this toy example has 3 vectors in 2 dimensions. Real-world applications have:
- **Millions of vectors** (documents, images, products)
- **1024+ dimensions** (typical embedding size)
- **Thousands of queries per second**

For each query, you perform:
- **100 million vectors × 1024 dimensions = 102 billion operations**

A single search without optimization would take seconds or minutes. This is why vector databases obsess over performance, and why we need SIMD.

## The Similarity Search Problem: Optimizing Distance Calculations

At the heart of every vector database query are distance calculations. Let's look at dot product similarity as our example:

```rust
// Scalar implementation
fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b)
        .map(|(&x, &y)| x * y)
        .sum()
}
```

For a 1024-dimensional vector, this performs 1024 multiplications and 1023 additions. When you're doing this for 100,000 vectors, that's over 100 million operations.

## Enter SIMD: Parallel Processing on Steroids

SIMD stands for Single Instruction, Multiple Data. Instead of processing one number at a time, SIMD lets your CPU process multiple numbers in a single instruction.

Think of it like this:
- **Scalar**: One cashier checking out one customer at a time
- **SIMD**: Four cashiers working in perfect synchronization, all scanning items at the exact same moment

Modern CPUs have SIMD instruction sets like SSE, AVX, and NEON (on ARM). On an M1 Mac, NEON can process 4 x f32 values simultaneously. On Intel/AMD with AVX-512, you can process 16 x f32 values at once!

Here's a visual representation:

```
Scalar Processing:
[1.0] * [2.0] = [2.0]   ← One multiplication
[1.5] * [2.5] = [3.75]  ← Another multiplication  
[2.0] * [3.0] = [6.0]   ← Yet another...
[2.5] * [3.5] = [8.75]  ← You get the idea

SIMD Processing (4-wide):
[1.0, 1.5, 2.0, 2.5] * [2.0, 2.5, 3.0, 3.5] = [2.0, 3.75, 6.0, 8.75]
         ↑ All done in ONE instruction! ↑
```

## Implementing SIMD in Ahnlich

For Ahnlich, we used the [`pulp`](https://crates.io/crates/pulp) crate, which provides portable SIMD abstractions across different CPU architectures. Here's how we implemented SIMD dot product ([see full implementation on GitHub](https://github.com/deven96/ahnlich/blob/main/ahnlich/db/src/algorithm/similarity.rs#L134-L162)):

```rust
use pulp::{Arch, Simd, WithSimd};

struct DotProduct<'a> {
    first: &'a [f32],
    second: &'a [f32],
}

impl WithSimd for DotProduct<'_> {
    type Output = f32;

    #[inline(always)]
    fn with_simd<S: Simd>(self, simd: S) -> Self::Output {
        // Split arrays into SIMD-aligned chunks and remainder
        let (first_head, first_tail) = S::as_simd_f32s(self.first);
        let (second_head, second_tail) = S::as_simd_f32s(self.second);

        // SIMD accumulator starting at zero
        let mut sum_of_points = simd.splat_f32s(0.0);

        // Process SIMD chunks - 4 multiplies + 4 adds per iteration!
        for (&chunk_first, &chunk_second) in first_head.iter().zip(second_head) {
            sum_of_points = simd.mul_add_f32s(chunk_first, chunk_second, sum_of_points);
        }

        // Reduce SIMD register to single value
        let mut dot_product = simd.reduce_sum_f32s(sum_of_points);

        // Handle remaining elements with scalar code
        dot_product += first_tail
            .iter()
            .zip(second_tail)
            .map(|(&x, &y)| x * y)
            .sum::<f32>();
            
        dot_product
    }
}

pub fn dot_product(first: &[f32], second: &[f32]) -> f32 {
    let arch = Arch::new();
    arch.dispatch(DotProduct {
        first,
        second,
    })
}
```

Let's break down what's happening:

1. **Split into chunks**: `as_simd_f32s` splits our arrays into SIMD-aligned chunks (groups of 4 for NEON) plus a remainder tail
2. **SIMD accumulation**: `mul_add_f32s` performs fused multiply-add on 4 values simultaneously
3. **Reduction**: `reduce_sum_f32s` adds all values in the SIMD register together
4. **Handle remainder**: Process any leftover elements with scalar code

The same SIMD pattern extends to the other distance metrics Ahnlich supports. Each one benefits from parallel processing in different ways:

### Cosine Similarity

Cosine similarity measures the angle between two vectors, making it perfect for comparing semantic meaning in text embeddings. The formula is: **dot(A, B) / (magnitude(A) × magnitude(B))**

The computation involves two SIMD operations: calculating the dot product (which we already saw) and computing the magnitudes. Here's how it looks ([source](https://github.com/deven96/ahnlich/blob/main/ahnlich/db/src/algorithm/similarity.rs#L109-L125)):

```rust
fn cosine_similarity(first: &[f32], second: &[f32]) -> f32 {
    let dot_product = dot_product(first, second);
    
    // Calculate magnitudes using SIMD
    let arch = Arch::new();
    let magnitude = arch.dispatch(Magnitude {
        first,
        second,
    });
    
    dot_product / magnitude
}
```

The magnitude calculation also uses SIMD to compute `sqrt(Σ(x²))` for both vectors simultaneously, giving us another layer of parallelization.

### Euclidean Distance  

Euclidean distance measures the straight-line distance between two points in high-dimensional space. It's commonly used for image embeddings and continuous numerical features. The formula is: **sqrt(Σ(a[i] - b[i])²)**

This one's interesting because we can use SIMD for both the subtraction and the squaring ([source](https://github.com/deven96/ahnlich/blob/main/ahnlich/db/src/algorithm/similarity.rs#L198-L231)):

```rust
impl WithSimd for EuclideanDistance<'_> {
    type Output = f32;

    #[inline(always)]
    fn with_simd<S: Simd>(self, simd: S) -> Self::Output {
        let (first_head, first_tail) = S::as_simd_f32s(self.first);
        let (second_head, second_tail) = S::as_simd_f32s(self.second);

        let mut sum_of_squares = simd.splat_f32s(0.0);

        for (&cord_first, &cord_second) in first_head.iter().zip(second_head) {
            let diff = simd.sub_f32s(cord_first, cord_second);  // 4 subtractions
            sum_of_squares = simd.mul_add_f32s(diff, diff, sum_of_squares);  // 4 squares + 4 adds
        }

        let mut total = simd.reduce_sum_f32s(sum_of_squares);

        // Scalar remainder
        total += first_tail
            .iter()
            .zip(second_tail)
            .map(|(&x, &y)| {
                let diff = x - y;
                diff * diff
            })
            .sum::<f32>();

        total.sqrt()
    }
}
```

Notice how we're doing 8 SIMD operations per iteration (4 subtractions + 4 multiply-adds), making this particularly efficient.

## The Benchmark Results

Let's see how our SIMD implementation performs. Testing with 100,000 vectors of 1024 dimensions each on Apple M1:

```rust
fn main() {
    let dimension = 1024;
    let size = 100_000;
    
    let query: Vec<f32> = (0..dimension).map(|i| i as f32 * 0.1).collect();
    let vectors: Vec<Vec<f32>> = (0..size)
        .map(|_| (0..dimension).map(|i| i as f32 * 0.01).collect())
        .collect();
    
    // Benchmark scalar
    let start = Instant::now();
    for v in &vectors {
        let _ = dot_product_scalar(&query, v);
    }
    let scalar_duration = start.elapsed();
    
    // Benchmark SIMD
    let start = Instant::now();
    for v in &vectors {
        let _ = dot_product_simd(&query, v);
    }
    let simd_duration = start.elapsed();
    
    println!("Scalar: {:?}", scalar_duration);
    println!("SIMD:   {:?}", simd_duration);
    println!("Speedup: {:.2}x", scalar_duration.as_secs_f64() / simd_duration.as_secs_f64());
}
```

**Results (Apple M1 with ARM NEON)**:
```
Scalar duration: 111.22ms
SIMD duration:   23.49ms

Speedup: 4.73x
```

4.7x faster with SIMD. For a vector database handling millions of similarity calculations per second, this translates to significant real-world performance improvements.

## Peeking Under the Hood: The Assembly

Let's verify what's actually happening at the assembly level. Using `cargo-show-asm`, we can inspect the generated machine code:

```bash
cargo install cargo-show-asm
cargo asm --rust simd_bench::dot_product_simd
```

Key SIMD instruction from the output:
```asm
LBB0_4:
    ldr q1, [x10], #16       ; Load 4 x f32 from first array into vector register
    ldr q2, [x11], #16       ; Load 4 x f32 from second array into vector register
    fmla.4s v0, v2, v1       ; Fused multiply-add: v0 += v2 * v1 (4 operations!)
    subs x9, x9, #1          ; Decrement counter
    b.ne LBB0_4              ; Loop if not done
```

The magic line is `fmla.4s v0, v2, v1`. This single ARM NEON instruction performs **4 multiply-adds simultaneously**. Instead of:
```
result += a[0] * b[0]
result += a[1] * b[1]
result += a[2] * b[2]
result += a[3] * b[3]
```

We get all four operations in **one instruction cycle**. That's the power of SIMD.

## The Auto-Vectorization Question

Modern compilers like LLVM are surprisingly good at auto-vectorization. You might wonder, "Does the compiler already vectorize my scalar code?"

Sometimes, but not always.

Let's examine what LLVM does with our scalar implementation. The scalar version uses iterator combinators:

```rust
fn dot_product_scalar(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(&x, &y)| x * y).sum()
}
```

When compiled with `-C opt-level=3` (release mode), LLVM's auto-vectorizer kicks in and **does** generate SIMD instructions for this simple pattern. This is why our benchmark shows good scalar performance, it's actually using SIMD under the hood!

### Proving Auto-Vectorization

We can verify this by examining the assembly or by disabling SIMD features entirely. On ARM, we can disable NEON:

```bash
RUSTFLAGS="-C target-feature=-neon" cargo run --release
```

**Results without NEON**:
```
Scalar duration: 762.05ms  (7x slower)
SIMD duration:   131.84ms  (explicit SIMD fallback path)
```

The scalar version tanks without auto-vectorization, taking **762ms** instead of **111ms**.

### Why Explicit SIMD Still Matters

If compilers auto-vectorize, why write explicit SIMD code?

1. **Compiler Limitations**: Auto-vectorization works well for simple loops but fails on complex patterns. Vector databases have intricate algorithms (graph traversal, tree searches) where manual SIMD wins.

2. **Portability Guarantees**: Auto-vectorization quality varies across:
   - Different compilers (GCC vs Clang vs MSVC)
   - Compiler versions (what works in LLVM 15 may not in LLVM 12)
   - Target architectures (x86 vs ARM vs RISC-V)
   
   Explicit SIMD using `pulp` ensures consistent performance everywhere.

3. **Control Over Optimization**: Sometimes you need fine-grained control:
   - Custom reduction operations
   - Specific instruction selection (FMA over separate mul+add)
   - Memory alignment requirements
   
4. **Performance Predictability**: Auto-vectorization is "best effort." A small code change can break vectorization without warning. Explicit SIMD gives predictable performance.

5. **Documentation**: Explicit SIMD clearly communicates intent. Future maintainers know this code *must* be fast.

### When Auto-Vectorization Fails

Here's a real example where auto-vectorization struggles:

```rust
// Compiler probably vectorizes this ✅
fn simple_sum(arr: &[f32]) -> f32 {
    arr.iter().sum()
}

// Compiler struggles to vectorize this ❌
fn conditional_sum(arr: &[f32], threshold: f32) -> f32 {
    let mut sum = 0.0;
    for &val in arr {
        if val > threshold {
            sum += val * val;
        } else {
            sum += val;
        }
    }
    sum
}
```

Branches inside hot loops confuse auto-vectorizers. SIMD techniques like masking or blend instructions can handle this, but you need explicit SIMD to access them.

### Visualizing Auto-Vectorization

Here's what happens at compile time:

![Auto-vectorization Flow](/img/blog/auto-vectorization-flow.svg)

With explicit SIMD using `pulp`, we bypass these heuristics and tell the compiler exactly what to do.

## The Real-World Impact

While our simple dot product didn't show dramatic speedups on M1 due to auto-vectorization, the benefits become clear in:

- **Cross-platform consistency**: The SIMD code performs predictably across ARM, x86, and other architectures
- **Complex algorithms**: KD-tree and HNSW implementations benefit from hand-tuned SIMD
- **Future-proofing**: As we add more sophisticated distance metrics, explicit SIMD control helps maintain performance

Here's our full suite of similarity algorithms using SIMD:

| Algorithm | Use Case | SIMD Benefit |
|-----------|----------|--------------|
| Cosine Similarity | Text embeddings, semantic search | Magnitude + dot product SIMD |
| Euclidean Distance | Image vectors, continuous features | Fused multiply-add for differences |
| Dot Product | Recommendation systems | Direct SIMD multiplication |

## Key Takeaways

1. **SIMD provides real speedups**: We achieved 4.7x faster dot product calculations on ARM NEON
2. **Modern compilers auto-vectorize simple patterns**: LLVM turned our scalar code into SIMD instructions automatically
3. **Auto-vectorization has limits**: Complex algorithms, branches, and intricate patterns need explicit SIMD
4. **Explicit SIMD guarantees performance**: No surprises across compilers, versions, or architectures
5. **The `pulp` crate rocks**: Portable SIMD in Rust without drowning in architecture-specific intrinsics
6. **Always verify with assembly**: Use `cargo-show-asm` to see what's really happening
7. **Benchmark on real hardware**: Measure performance on your actual target architecture

## Epilogue: What About Linux/x86?

The benchmarks shown here are on Apple M1 (ARM NEON). On Intel/AMD with AVX2 or AVX-512, the SIMD width is larger (8 to 16 x f32 vs 4 x f32), which means:
- More parallelism per instruction
- Different auto-vectorization behavior  
- Potentially larger SIMD vs scalar gaps

If you're running Ahnlich on x86, I'd love to see your benchmark results! Open an issue or PR on [GitHub](https://github.com/deven96/ahnlich) with your numbers.

## References

- [Ahnlich GitHub Repository](https://github.com/deven96/ahnlich)
- [Pulp: Portable SIMD for Rust](https://crates.io/crates/pulp)
- [LLVM Auto-Vectorization Documentation](https://llvm.org/docs/Vectorizers.html)
- [Understanding LLVM's Loop Vectorizer](https://llvm.org/devmtg/2013-04/achalmers-slides.pdf)
- [ARM NEON Intrinsics Guide](https://developer.arm.com/architectures/instruction-sets/simd-isas/neon)
- [Intel AVX Intrinsics Reference](https://www.intel.com/content/www/us/en/docs/intrinsics-guide/index.html)
- [Cargo Show ASM Tool](https://github.com/pacak/cargo-show-asm)
- [Rust Performance Book - SIMD](https://nnethercote.github.io/perf-book/simd.html)

---

_Want to try Ahnlich? Check out our [documentation](https://ahnlich.rs) and join our community. Contributions welcome!_
