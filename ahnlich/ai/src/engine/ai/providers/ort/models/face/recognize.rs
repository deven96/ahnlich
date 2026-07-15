//! ArcFace: aligned face crops in, one 512-d embedding each out.

use crate::error::AIProxyError;
use ndarray::{Array, Axis, Ix2, Ix4};
use ort::Session;

/// Embed every aligned crop in one forward pass.
pub(crate) fn embed(
    faces: Array<f32, Ix4>,
    session: &Session,
) -> Result<Array<f32, Ix2>, AIProxyError> {
    let inputs = ort::inputs!["input.1" => faces.view()]
        .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

    let outputs = session
        .run(inputs)
        .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;

    let embeddings = outputs[0]
        .try_extract_tensor::<f32>()
        .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

    let shape = embeddings.shape();
    let mut embeddings = embeddings
        .to_owned()
        .into_shape_with_order((shape[0], shape[1]))
        .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

    l2_normalize(&mut embeddings);
    Ok(embeddings)
}

/// Cosine does not care, but an ArcFace vector's magnitude tracks image quality, so raw
/// vectors would let a dot-product or euclidean index rank by quality over identity.
fn l2_normalize(embeddings: &mut Array<f32, Ix2>) {
    for mut embedding in embeddings.axis_iter_mut(Axis(0)) {
        let norm = embedding.dot(&embedding).sqrt();
        if norm > 1e-12 {
            embedding /= norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::arr2;

    #[test]
    fn normalizing_puts_every_row_on_the_unit_sphere() {
        let mut embeddings = arr2(&[[3.0f32, 4.0], [0.0, 0.0], [-8.0, 6.0]]);
        l2_normalize(&mut embeddings);

        assert_eq!(embeddings.row(0).to_vec(), vec![0.6, 0.8]);
        assert_eq!(embeddings.row(1).to_vec(), vec![0.0, 0.0]); // zero vector
        assert_eq!(embeddings.row(2).to_vec(), vec![-0.8, 0.6]);
    }
}
