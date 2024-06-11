use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
/// Supported ahnlich similarity algorithms
pub enum Algorithm {
    /// Euclidean distance is defined as the L2-norm of the difference between two vectors or their
    /// straight line distance between them. It
    /// considers both magnitude and direction of vectors
    EuclideanDistance,
    /// Dot product similarity is calculated by adding the product of the vectors corresponding
    /// components. It is a product of the vectors and the cosine of the angle between them
    DotProductSimilarity,
    /// Cosine similarity is the measure of the angle between two vectors. It is computed by taking
    /// the dot product of the vectors and dividing it by the product of their magnitudes. This
    /// metric is not affected by the magnitude of the vectors but only the angle bbetween them
    CosineSimilarity,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Similarity(pub f32);

impl PartialEq for Similarity {
    fn eq(&self, other: &Self) -> bool {
        (self.0 - other.0).abs() < std::f32::EPSILON
    }
}

impl Eq for Similarity {}
