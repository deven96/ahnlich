// Speed comparison: All-at-once vs Streaming decoding
// Run with: cargo bench --bench speed_comparison

use ahnlich_ai_proxy::engine::ai::models::ImageArray;
use ahnlich_types::keyval::StoreInput;
use ahnlich_types::keyval::store_input::Value;
use rayon::prelude::*;
use std::sync::Arc;
use std::time::Instant;

fn main() {
    println!("\n=== Speed Comparison: All-at-once vs Streaming ===\n");

    let cat_png = include_bytes!("../src/tests/images/cat.png");
    let dog_jpg = include_bytes!("../src/tests/images/dog.jpg");
    let test_webp = include_bytes!("../src/tests/images/test.webp");

    // Test with 150 images (50 copies of 3 images)
    let mut batch = Vec::new();
    for _ in 0..50 {
        batch.push(StoreInput {
            value: Some(Value::Image(cat_png.to_vec())),
        });
        batch.push(StoreInput {
            value: Some(Value::Image(dog_jpg.to_vec())),
        });
        batch.push(StoreInput {
            value: Some(Value::Image(test_webp.to_vec())),
        });
    }

    let arc_inputs = Arc::new(batch);
    let total_images = arc_inputs.len();

    println!("Testing with {} images\n", total_images);
    println!("Running 10 iterations each...\n");

    // Warm up
    for _ in 0..2 {
        let _: Vec<ImageArray> = arc_inputs
            .par_iter()
            .filter_map(|input| match &input.value {
                Some(Value::Image(image_bytes)) => {
                    Some(ImageArray::try_from(image_bytes.as_slice()).ok()?)
                }
                _ => None,
            })
            .collect();
    }

    // Benchmark: All at once
    let mut all_at_once_times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        let _: Vec<ImageArray> = arc_inputs
            .par_iter()
            .filter_map(|input| match &input.value {
                Some(Value::Image(image_bytes)) => {
                    Some(ImageArray::try_from(image_bytes.as_slice()).ok()?)
                }
                _ => None,
            })
            .collect();
        all_at_once_times.push(start.elapsed());
    }

    // Benchmark: Streaming (chunks of 16)
    let batch_size = 16;
    let mut streaming_times = Vec::new();
    for _ in 0..10 {
        let start = Instant::now();
        for chunk_start in (0..total_images).step_by(batch_size) {
            let chunk_end = (chunk_start + batch_size).min(total_images);
            let chunk = &arc_inputs[chunk_start..chunk_end];

            let _: Vec<ImageArray> = chunk
                .par_iter()
                .filter_map(|input| match &input.value {
                    Some(Value::Image(image_bytes)) => {
                        Some(ImageArray::try_from(image_bytes.as_slice()).ok()?)
                    }
                    _ => None,
                })
                .collect();
        }
        streaming_times.push(start.elapsed());
    }

    // Calculate averages
    let avg_all_at_once: f64 = all_at_once_times
        .iter()
        .map(|d| d.as_secs_f64())
        .sum::<f64>()
        / all_at_once_times.len() as f64;
    let avg_streaming: f64 =
        streaming_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / streaming_times.len() as f64;

    println!("All-at-once approach:");
    println!("  Average: {:.2}ms", avg_all_at_once * 1000.0);
    println!(
        "  Min: {:.2}ms",
        all_at_once_times.iter().min().unwrap().as_secs_f64() * 1000.0
    );
    println!(
        "  Max: {:.2}ms",
        all_at_once_times.iter().max().unwrap().as_secs_f64() * 1000.0
    );

    println!("\nStreaming approach (chunks of {}):", batch_size);
    println!("  Average: {:.2}ms", avg_streaming * 1000.0);
    println!(
        "  Min: {:.2}ms",
        streaming_times.iter().min().unwrap().as_secs_f64() * 1000.0
    );
    println!(
        "  Max: {:.2}ms",
        streaming_times.iter().max().unwrap().as_secs_f64() * 1000.0
    );

    let diff_pct = ((avg_streaming - avg_all_at_once) / avg_all_at_once) * 100.0;
    println!("\nDifference: {:.1}%", diff_pct);

    if diff_pct.abs() < 5.0 {
        println!("✓ Performance impact is negligible (<5%)");
    } else if diff_pct > 0.0 {
        println!("⚠ Streaming is {:.1}% slower", diff_pct);
    } else {
        println!("✓ Streaming is {:.1}% faster!", diff_pct.abs());
    }

    println!("\n=== Comparison Complete ===\n");
}
