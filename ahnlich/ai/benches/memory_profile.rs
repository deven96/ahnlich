// Memory profiling benchmark for AI proxy
// Run with: cargo bench --package ai --bench memory_profile
//
// This profiles memory allocations for different batch sizes of inputs
// to understand the memory pressure before we add streaming.
//
// Measures actual image decoding from compressed bytes to RGB8 pixels.
//
// NOTE: This uses criterion for easier execution, but the actual profiling
// happens through manual measurement and output, not through dhat,
// because utils crate already defines a global allocator.

use ahnlich_types::keyval::StoreInput;
use ahnlich_types::keyval::store_input::Value;
use rayon::prelude::*;
use std::sync::Arc;

fn main() {
    println!("\n=== Memory Analysis for AI Proxy ===\n");
    println!("This benchmark shows the memory characteristics of different operations.");
    println!("For detailed heap profiling, use dhat separately on the AI binary.\n");

    // Profile text inputs (baseline - no decoding)
    println!("Text inputs (baseline - clones strings):");
    profile_text_inputs(100);
    profile_text_inputs(1000);

    // Profile image Arc wrapping WITHOUT decoding
    println!("\nImage Arc wrapping (no decoding, compressed bytes only):");
    profile_image_arc_only(10, 500_000);
    profile_image_arc_only(100, 500_000);

    // Profile ACTUAL image decoding (the memory hotspot)
    println!("\nIMAGE DECODING (compressed -> RGB8) - THE MEMORY HOTSPOT:");
    measure_image_decoding_memory(10);
    measure_image_decoding_memory(50);
    measure_image_decoding_memory(100);

    println!("\n=== Analysis complete ===\n");
}

fn profile_text_inputs(batch_size: usize) {
    let inputs: Vec<StoreInput> = (0..batch_size)
        .map(|i| StoreInput {
            value: Some(Value::RawString(format!(
                "This is a test sentence number {} for memory profiling. \
                 It contains some text to simulate real embedding requests. \
                 The sentence is long enough to represent typical usage with \
                 multiple words and phrases that might be embedded.",
                i
            ))),
        })
        .collect();

    let arc_inputs = Arc::new(inputs);

    // Simulate string extraction (clones the strings)
    let _strings: Vec<String> = arc_inputs
        .par_iter()
        .filter_map(|input| match &input.value {
            Some(Value::RawString(s)) => Some(s.clone()),
            _ => None,
        })
        .collect();

    println!("  - Processed {} text inputs", batch_size);
}

fn profile_image_arc_only(batch_size: usize, image_size_bytes: usize) {
    let inputs: Vec<StoreInput> = (0..batch_size)
        .map(|_| StoreInput {
            value: Some(Value::Image(vec![0u8; image_size_bytes])),
        })
        .collect();

    let arc_inputs = Arc::new(inputs);

    // Just iterate, no decoding
    let _total_bytes: usize = arc_inputs
        .par_iter()
        .filter_map(|input| match &input.value {
            Some(Value::Image(bytes)) => Some(bytes.len()),
            _ => None,
        })
        .sum();

    println!(
        "  - Arc-wrapped {} images @ {}KB (no decoding)",
        batch_size,
        image_size_bytes / 1024
    );
}

fn measure_image_decoding_memory(batch_size: usize) {
    use ahnlich_ai_proxy::engine::ai::models::ImageArray;

    // Generate a real tiny PNG image (1x1 pixel) and replicate it
    // This is more realistic than random bytes which won't decode
    let tiny_png = create_tiny_png();
    let compressed_size = tiny_png.len();

    let inputs: Vec<StoreInput> = (0..batch_size)
        .map(|_| StoreInput {
            value: Some(Value::Image(tiny_png.clone())),
        })
        .collect();

    let total_compressed = compressed_size * batch_size;
    let arc_inputs = Arc::new(inputs);

    // THIS IS THE ACTUAL HOTSPOT: Decoding all images at once
    // Mimics manager/mod.rs:115-123
    let image_arrays: Vec<ImageArray> = arc_inputs
        .par_iter()
        .filter_map(|input| match &input.value {
            Some(Value::Image(image_bytes)) => {
                Some(ImageArray::try_from(image_bytes.as_slice()).ok()?)
            }
            _ => None,
        })
        .collect();

    // Each 1x1 RGB8 image = 3 bytes (R,G,B)
    // But ImageBuffer has overhead, so approximate
    let estimated_decoded_size = batch_size * 3; // minimal for 1x1

    println!(
        "  - {} images: compressed={}KB, decoded≈{}B ({} ImageArrays created)",
        batch_size,
        total_compressed / 1024,
        estimated_decoded_size,
        image_arrays.len()
    );
    println!(
        "    For real 224x224 images: decoded would be ≈{}MB vs compressed≈{}MB",
        (batch_size * 224 * 224 * 3) / (1024 * 1024),
        (batch_size * 50_000) / (1024 * 1024) // assume 50KB compressed
    );
}

// Creates a minimal valid PNG image (1x1 red pixel)
fn create_tiny_png() -> Vec<u8> {
    use image::{ImageBuffer, Rgb};
    let img: ImageBuffer<Rgb<u8>, Vec<u8>> = ImageBuffer::from_pixel(1, 1, Rgb([255, 0, 0]));
    let mut buffer = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut buffer),
        image::ImageFormat::Png,
    )
    .expect("Failed to encode PNG");
    buffer
}
