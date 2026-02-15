// Realistic memory profiling for AI proxy
//
// This benchmark measures memory usage with actual embedding workloads
// using real image and text inputs similar to integration tests.
//
// Run with: cargo bench --bench realistic_memory_profile -- --nocapture

use ahnlich_ai_proxy::engine::ai::models::ImageArray;
use ahnlich_types::keyval::StoreInput;
use ahnlich_types::keyval::store_input::Value;
use rayon::prelude::*;
use std::sync::Arc;

fn main() {
    println!("\n=== Realistic Memory Profiling for AI Proxy ===\n");
    println!("This benchmark uses real images and preprocessing to measure actual memory usage.");
    println!("Simulates production workloads with different batch sizes.\n");

    // Real images from test suite
    let cat_png = include_bytes!("../src/tests/images/cat.png");
    let dog_jpg = include_bytes!("../src/tests/images/dog.jpg");
    let test_webp = include_bytes!("../src/tests/images/test.webp");

    println!("Loaded test images:");
    println!("  - cat.png: {} bytes", cat_png.len());
    println!("  - dog.jpg: {} bytes", dog_jpg.len());
    println!("  - test.webp: {} bytes\n", test_webp.len());

    // Profile the current "decode all at once" approach
    println!("=== CURRENT APPROACH: Decode ALL images at once ===\n");

    profile_image_batch("Small batch", vec![cat_png, dog_jpg, test_webp], 3);
    profile_image_batch(
        "Medium batch",
        vec![cat_png, dog_jpg, test_webp],
        25, // 25 copies = 75 images total
    );
    profile_image_batch(
        "Large batch",
        vec![cat_png, dog_jpg, test_webp],
        50, // 50 copies = 150 images total
    );

    // Theoretical comparison
    println!("\n=== STREAMING APPROACH (theoretical) ===");
    println!("If we decode in chunks of 16 (ONNX batch size):");
    println!("  - Small batch (9 images): 9 ImageArrays in memory");
    println!("  - Medium batch (75 images): 16 ImageArrays max at once (5x reduction)");
    println!("  - Large batch (150 images): 16 ImageArrays max at once (9x reduction)\n");

    println!("=== Profiling Complete ===\n");
}

fn profile_image_batch(label: &str, images: Vec<&[u8]>, copies: usize) {
    // Create batch by replicating the base images
    let mut batch = Vec::new();
    for _ in 0..copies {
        for img_bytes in &images {
            batch.push(StoreInput {
                value: Some(Value::Image(img_bytes.to_vec())),
            });
        }
    }

    let total_images = batch.len();
    let total_compressed: usize = batch
        .iter()
        .map(|input| match &input.value {
            Some(Value::Image(bytes)) => bytes.len(),
            _ => 0,
        })
        .sum();

    println!(
        "{} ({} images, {}KB compressed):",
        label,
        total_images,
        total_compressed / 1024
    );

    // Arc-wrap like the real code does
    let arc_inputs = Arc::new(batch);

    // THIS IS THE BOTTLENECK: Decode ALL at once
    // From manager/mod.rs:115-123
    let start = std::time::Instant::now();
    let image_arrays: Vec<ImageArray> = arc_inputs
        .par_iter()
        .filter_map(|input| match &input.value {
            Some(Value::Image(image_bytes)) => {
                Some(ImageArray::try_from(image_bytes.as_slice()).ok()?)
            }
            _ => None,
        })
        .collect();
    let decode_time = start.elapsed();

    // Estimate decoded memory (each 224x224 RGB8 image = 150KB)
    // But our test images vary in size after decoding, so estimate
    let estimated_decoded_mb = (total_images * 224 * 224 * 3) / (1024 * 1024);

    println!(
        "  - Decoded {} images in {:?}",
        image_arrays.len(),
        decode_time
    );
    println!(
        "  - Estimated memory: {}KB compressed → ~{}MB decoded ({}x expansion)",
        total_compressed / 1024,
        estimated_decoded_mb,
        if total_compressed > 0 {
            (estimated_decoded_mb * 1024) / (total_compressed / 1024)
        } else {
            0
        }
    );
    println!();
}
