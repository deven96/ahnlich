/// Utility functions for bounding box calculations and normalization

/// Normalized bounding box coordinates (0-1 range)
pub struct NormalizedBBox {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

/// Applies letterbox correction to a bounding box and normalizes coordinates to 0-1 range.
///
/// When an image is letterboxed (resized with black bars to preserve aspect ratio),
/// the bounding box coordinates from the model need to be:
/// 1. Adjusted to remove the letterbox offset
/// 2. Normalized to the original image dimensions (0-1 range)
///
/// # Arguments
/// * `bbox` - Raw bounding box [x1, y1, x2, y2] from model output
/// * `orig_width` - Original image width
/// * `orig_height` - Original image height
/// * `img_size` - Model input size (e.g., 640.0 for 640x640)
///
/// # Returns
/// Normalized bounding box with coordinates in 0-1 range, clamped to valid values
pub fn apply_letterbox_correction(
    bbox: &[f32; 4],
    orig_width: f32,
    orig_height: f32,
    img_size: f32,
) -> NormalizedBBox {
    // Calculate letterbox parameters
    // The image was resized to img_size x img_size with letterboxing (black bars) to preserve aspect ratio
    // IMPORTANT: Use integer math to match the preprocessing code exactly
    let scale = (img_size / orig_width).min(img_size / orig_height);
    let scaled_width_int = (orig_width * scale) as u32;
    let scaled_height_int = (orig_height * scale) as u32;
    let scaled_width = scaled_width_int as f32;
    let scaled_height = scaled_height_int as f32;
    let offset_x = ((img_size as u32 - scaled_width_int) / 2) as f32;
    let offset_y = ((img_size as u32 - scaled_height_int) / 2) as f32;

    // Adjust bounding boxes to account for letterboxing
    // 1. Subtract letterbox offset to get coords in scaled image space
    // 2. Divide by scaled dimensions to normalize to 0-1
    let x1_adjusted = (bbox[0] - offset_x) / scaled_width;
    let y1_adjusted = (bbox[1] - offset_y) / scaled_height;
    let x2_adjusted = (bbox[2] - offset_x) / scaled_width;
    let y2_adjusted = (bbox[3] - offset_y) / scaled_height;

    NormalizedBBox {
        x1: x1_adjusted.clamp(0.0, 1.0),
        y1: y1_adjusted.clamp(0.0, 1.0),
        x2: x2_adjusted.clamp(0.0, 1.0),
        y2: y2_adjusted.clamp(0.0, 1.0),
    }
}
