//! SCRFD detection head: raw model outputs to faces.
//!
//! Anchor-point based, so every predicted number is a distance from a point, in units of
//! that level's stride: `box = [x-l*s, y-t*s, x+r*s, y+b*s]`, `kp = point + delta*s`. No
//! variances, no `exp()`: those belong to the older anchor-box RetinaFace.
//!
//! Wrong landmarks fail silently, since the counts come from the boxes but the recognition
//! crop comes from the landmarks. `test_buffalo_l_embeddings_discriminate_faces` catches it.

use super::face_align::FaceDetection;
use crate::error::AIProxyError;
use ndarray::{ArrayView, IxDyn};

/// Feature-pyramid strides, smallest faces first.
const FPN_STRIDES: [usize; 3] = [8, 16, 32];

const ANCHORS_PER_CELL: usize = 2;

/// The preprocessor letterboxes every image to this, so the anchor grid is fixed.
pub(crate) const INPUT_SIZE: usize = 640;

/// An anchor point, plus the stride whose units its predictions are in.
#[derive(Debug, Clone, Copy)]
pub(crate) struct Anchor {
    pub x: f32,
    pub y: f32,
    pub stride: f32,
}

/// Anchor points for every level, in the order the model emits its rows.
///
/// One point per cell at `(col * stride, row * stride)`, twice. The cell corner, not its
/// middle: SCRFD has no `+ 0.5`.
pub(crate) fn generate_anchors() -> Vec<Anchor> {
    let mut anchors = Vec::new();

    for stride in FPN_STRIDES {
        for row in 0..INPUT_SIZE / stride {
            for col in 0..INPUT_SIZE / stride {
                let anchor = Anchor {
                    x: (col * stride) as f32,
                    y: (row * stride) as f32,
                    stride: stride as f32,
                };
                anchors.extend(std::iter::repeat_n(anchor, ANCHORS_PER_CELL));
            }
        }
    }

    anchors
}

fn anchors_at(stride: usize) -> usize {
    let cells = INPUT_SIZE / stride;
    cells * cells * ANCHORS_PER_CELL
}

/// Decode one image's raw outputs into faces above `min_confidence`.
///
/// The 9 output tensors are matched by SHAPE, not position: the graph groups them (all
/// scores, then all boxes, then all keypoints) and the runtime promises no ordering.
pub(crate) fn decode(
    outputs: &[ArrayView<f32, IxDyn>],
    anchors: &[Anchor],
    min_confidence: f32,
) -> Result<Vec<FaceDetection>, AIProxyError> {
    let mut faces = Vec::new();
    let mut offset = 0;

    for stride in FPN_STRIDES {
        let count = anchors_at(stride);
        let tensor = |cols: usize| by_shape(outputs, count, cols, stride);

        let scores = tensor(1)?;
        let boxes = tensor(4)?;
        let points = tensor(10)?;

        for i in 0..count {
            // A NaN score fails `<` and would reach `apply_nms`, which sorts with
            // `partial_cmp().unwrap()` and panics.
            let confidence = scores[i];
            if !confidence.is_finite() || confidence < min_confidence {
                continue;
            }

            let Anchor { x, y, stride: s } = anchors[offset + i];

            // Distances to the box edges: left, top, right, bottom.
            let bbox = [
                x - boxes[i * 4] * s,
                y - boxes[i * 4 + 1] * s,
                x + boxes[i * 4 + 2] * s,
                y + boxes[i * 4 + 3] * s,
            ];

            // 5 (dx, dy) offsets: both eyes, nose, both mouth corners.
            let mut landmarks = [[0.0f32; 2]; 5];
            for (j, landmark) in landmarks.iter_mut().enumerate() {
                landmark[0] = x + points[i * 10 + j * 2] * s;
                landmark[1] = y + points[i * 10 + j * 2 + 1] * s;
            }

            faces.push(FaceDetection {
                bbox,
                landmarks,
                confidence,
            });
        }

        offset += count;
    }

    Ok(faces)
}

/// The `[rows, cols]` output tensor, as a flat slice.
fn by_shape<'a>(
    outputs: &'a [ArrayView<f32, IxDyn>],
    rows: usize,
    cols: usize,
    stride: usize,
) -> Result<&'a [f32], AIProxyError> {
    outputs
        .iter()
        .find(|t| t.shape() == [rows, cols])
        .and_then(|t| t.as_slice())
        .ok_or_else(|| {
            AIProxyError::ModelProviderPostprocessingError(format!(
                "SCRFD output [{rows}, {cols}] missing for stride {stride}"
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ndarray::{Array2, IxDyn};

    #[test]
    fn anchor_grid_matches_the_model_output_lengths() {
        let anchors = generate_anchors();

        // 80x80x2 + 40x40x2 + 20x20x2. These lengths are the model's output rows.
        assert_eq!(anchors.len(), 12800 + 3200 + 800);
        assert_eq!(anchors[0].stride, 8.0);

        // Cell corners, not centres: the second cell of row 0 sits at x = stride.
        assert_eq!((anchors[0].x, anchors[0].y), (0.0, 0.0));
        assert_eq!((anchors[2].x, anchors[2].y), (8.0, 0.0));
    }

    #[test]
    fn decodes_distances_from_the_anchor_point() {
        let anchors = generate_anchors();

        // One face on the stride-8 level, at anchor 0 (the origin).
        let mut scores = Array2::<f32>::zeros((12800, 1));
        let mut boxes = Array2::<f32>::zeros((12800, 4));
        let mut points = Array2::<f32>::zeros((12800, 10));
        scores[[0, 0]] = 0.9;
        boxes
            .row_mut(0)
            .assign(&ndarray::arr1(&[1.0, 2.0, 3.0, 4.0]));
        points[[0, 0]] = 1.0; // left eye dx
        points[[0, 1]] = 2.0; // left eye dy

        let empty = |r, c| Array2::<f32>::zeros((r, c)).into_dyn();
        let tensors = [
            scores.into_dyn(),
            boxes.into_dyn(),
            points.into_dyn(),
            empty(3200, 1),
            empty(3200, 4),
            empty(3200, 10),
            empty(800, 1),
            empty(800, 4),
            empty(800, 10),
        ];
        let views: Vec<ArrayView<f32, IxDyn>> = tensors.iter().map(|t| t.view()).collect();

        let faces = decode(&views, &anchors, 0.5).unwrap();

        assert_eq!(faces.len(), 1);
        // Anchor (0,0), stride 8: box edges are distance * stride away from the point.
        assert_eq!(faces[0].bbox, [-8.0, -16.0, 24.0, 32.0]);
        assert_eq!(faces[0].landmarks[0], [8.0, 16.0]);
    }

    #[test]
    fn a_nan_score_is_dropped() {
        let anchors = generate_anchors();
        let mut scores = Array2::<f32>::zeros((12800, 1));
        scores[[0, 0]] = f32::NAN;

        let empty = |r, c| Array2::<f32>::zeros((r, c)).into_dyn();
        let tensors = [
            scores.into_dyn(),
            empty(12800, 4),
            empty(12800, 10),
            empty(3200, 1),
            empty(3200, 4),
            empty(3200, 10),
            empty(800, 1),
            empty(800, 4),
            empty(800, 10),
        ];
        let views: Vec<ArrayView<f32, IxDyn>> = tensors.iter().map(|t| t.view()).collect();

        assert!(decode(&views, &anchors, 0.5).unwrap().is_empty());
    }

    #[test]
    fn a_missing_output_is_an_error_not_a_panic() {
        let anchors = generate_anchors();
        let scores = Array2::<f32>::zeros((12800, 1)).into_dyn();
        let views = vec![scores.view()];

        assert!(decode(&views, &anchors, 0.5).is_err());
    }
}
