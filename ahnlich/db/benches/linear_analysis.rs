// Analyze why linear search is taking 6.8ms for 10k vectors
// Expected: ~1-2ms for pure distance calculations
// Actual: ~6.8ms - where is the overhead?

use ahnlich_db::engine::store::StoreHandler;
use ahnlich_types::keyval::{StoreKey, StoreName, StoreValue};
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::schema::Schema;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::time::Instant;

static DISTANCE_CALC_COUNT: AtomicUsize = AtomicUsize::new(0);

fn main() {
    println!("=== LINEAR SEARCH ANALYSIS ===");
    println!("Goal: Understand why 10k linear search takes 6.8ms\n");
    
    let handler = Arc::new(StoreHandler::new(Arc::new(AtomicBool::new(false))));
    
    test_raw_distance_performance();
    test_linear_search_components(&handler);
    test_data_layout_impact(&handler);
}

fn test_raw_distance_performance() {
    println!(">>> Test 1: Raw Distance Calculation Performance\n");
    
    // First test: Hash calculation cost
    use ahash::RandomState;
    use std::hash::{BuildHasher, Hasher};
    
    let dimension = 1024;
    let vec: Vec<f32> = (0..dimension).map(|i| i as f32).collect();
    
    let start = Instant::now();
    let iterations = 10000;
    for _ in 0..iterations {
        let mut hasher = RandomState::with_seeds(0, 0, 0, 0).build_hasher();
        for element in vec.iter() {
            hasher.write_u32(element.to_bits());
        }
        let _hash = hasher.finish();
    }
    println!("Hash calculation (10k hashes): {:?}", start.elapsed());
    println!("Per hash: {}ns", start.elapsed().as_nanos() / iterations);
    println!("** This hash is computed on EVERY similarity comparison! **\n");
    
    let dimension = 1024;
    let iterations = 10000;
    
    // Test 1: Naive implementation
    let v1: Vec<f32> = (0..dimension).map(|i| i as f32).collect();
    let v2: Vec<f32> = (0..dimension).map(|i| (i + 1) as f32).collect();
    
    let start = Instant::now();
    let mut sum = 0.0f32;
    for _ in 0..iterations {
        let dot: f32 = v1.iter().zip(&v2).map(|(a, b)| a * b).sum();
        let norm_a: f32 = v1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm_b: f32 = v2.iter().map(|x| x * x).sum::<f32>().sqrt();
        sum += dot / (norm_a * norm_b);
    }
    let elapsed = start.elapsed();
    println!("Naive cosine ({}k calcs): {:?}", iterations / 1000, elapsed);
    println!("Per calculation: {}ns", elapsed.as_nanos() / iterations);
    println!("Checksum (prevent optimization): {}\n", sum);
    
    // Test 2: Using pulp SIMD directly
    use ahnlich_similarity::LinearAlgorithm;
    use ahnlich_similarity::DistanceFn;
    
    let start = Instant::now();
    let mut sum = 0.0f32;
    for _ in 0..iterations {
        sum += LinearAlgorithm::CosineSimilarity.distance(&v1, &v2);
    }
    let elapsed = start.elapsed();
    println!("SIMD cosine ({}k calcs): {:?}", iterations / 1000, elapsed);
    println!("Per calculation: {}ns", elapsed.as_nanos() / iterations);
    println!("Speedup: {:.2}x over naive\n", 
        (elapsed.as_nanos() / iterations) as f64 / (elapsed.as_nanos() / iterations) as f64);
    
    // Test 3: Different algorithms
    for algo in [
        LinearAlgorithm::CosineSimilarity,
        LinearAlgorithm::EuclideanDistance,
        LinearAlgorithm::DotProductSimilarity,
    ] {
        let start = Instant::now();
        let mut sum = 0.0f32;
        for _ in 0..iterations {
            sum += algo.distance(&v1, &v2);
        }
        println!("{:?}: {}ns per calc", algo, start.elapsed().as_nanos() / iterations);
    }
    
    println!("\nConclusion:");
    println!("If each distance calc takes ~100ns:");
    println!("  10k calcs should take: ~1ms");
    println!("  Actual linear search: ~6.8ms");
    println!("  Overhead: ~5.8ms (or 580ns per comparison)");
    println!("  This suggests 5-6x overhead beyond distance calculation");
}

fn test_linear_search_components(handler: &Arc<StoreHandler>) {
    println!("\n\n>>> Test 2: Linear Search Components\n");
    
    let dimension = 1024;
    let size = 10000;
    
    let store = StoreName { value: "linear".to_string() };
    handler.create_store(
        store.clone(),
        &Schema::default(),
        NonZeroUsize::new(dimension).unwrap(),
        vec![],
        HashSet::new(),
        false, // NO HNSW
    ).unwrap();
    
    // Insert data
    println!("Inserting {} vectors...", size);
    let start = Instant::now();
    for _ in 0..10 {
        let batch: Vec<_> = (0..1000).map(|_| gen_vec(dimension)).collect();
        handler.set_in_store(&store, &Schema::default(), batch).unwrap();
    }
    println!("Insert time: {:?}\n", start.elapsed());
    
    // Measure search
    let query = StoreKey { key: vec![0.5; dimension] };
    
    println!("Timing linear search components:");
    
    // Full search
    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        handler.get_sim_in_store(
            &store,
            &Schema::default(),
            query.clone(),
            NonZeroUsize::new(10).unwrap(),
            Algorithm::CosineSimilarity,
            None,
        ).unwrap();
    }
    let total_time = start.elapsed();
    println!("Total search time: {:?} per query", total_time / iterations);
    
    // Break down by k value
    println!("\nSearch time by k:");
    for k in [1, 10, 100, 1000, 10000] {
        let start = Instant::now();
        let iters = 10;
        for _ in 0..iters {
            handler.get_sim_in_store(
                &store,
                &Schema::default(),
                query.clone(),
                NonZeroUsize::new(k).unwrap(),
                Algorithm::CosineSimilarity,
                None,
            ).unwrap();
        }
        println!("  k={:5}: {:?}", k, start.elapsed() / iters);
    }
    
    println!("\nObservations:");
    println!("- If k=1 and k=10000 have similar times:");
    println!("  → Heap operations are NOT the bottleneck");
    println!("- If all k values take ~5-7ms:");
    println!("  → Must be iterating ALL 10k vectors every time");
    println!("  → Overhead is in: iteration, locking, or data access");
}

fn test_data_layout_impact(handler: &Arc<StoreHandler>) {
    println!("\n\n>>> Test 3: Data Layout and Access Patterns\n");
    
    // Test different vector sizes to see cache effects
    let sizes = [100, 1000, 5000, 10000];
    let dimension = 1024;
    
    for size in sizes {
        let store = StoreName { value: format!("layout_{}", size) };
        handler.create_store(
            store.clone(),
            &Schema::default(),
            NonZeroUsize::new(dimension).unwrap(),
            vec![],
            HashSet::new(),
            false,
        ).unwrap();
        
        for _ in 0..(size / 1000).max(1) {
            let batch: Vec<_> = (0..1000.min(size)).map(|_| gen_vec(dimension)).collect();
            handler.set_in_store(&store, &Schema::default(), batch).unwrap();
        }
        
        let query = StoreKey { key: vec![0.5; dimension] };
        let start = Instant::now();
        let iters = 20;
        for _ in 0..iters {
            handler.get_sim_in_store(
                &store,
                &Schema::default(),
                query.clone(),
                NonZeroUsize::new(10).unwrap(),
                Algorithm::CosineSimilarity,
                None,
            ).unwrap();
        }
        let per_query = start.elapsed() / iters;
        let per_vector = per_query.as_nanos() / size as u128;
        println!("{:5} vectors: {:?} total, {}ns per vector", 
            size, per_query, per_vector);
    }
    
    println!("\nExpected:");
    println!("- Per-vector cost should be ~100-200ns (distance calc + overhead)");
    println!("- Should be linear with dataset size");
    println!("- Cache effects might cause slowdown at larger sizes");
}

fn gen_vec(dimension: usize) -> (StoreKey, StoreValue) {
    (
        StoreKey { key: (0..dimension).map(|_| rand::random()).collect() },
        StoreValue { value: HashMap::new() },
    )
}
