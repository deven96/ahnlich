use std::hash::{Hash, Hasher};
use std::sync::Arc;

/// Shared-ownership embedding vector used as the canonical currency type
/// across the non-linear index pipeline. Cloning is a cheap pointer bump.
#[derive(Debug, Clone)]
pub struct EmbeddingKey(pub Arc<Vec<f32>>);

impl EmbeddingKey {
    pub fn new(v: Vec<f32>) -> Self {
        Self(Arc::new(v))
    }

    pub fn as_slice(&self) -> &[f32] {
        self.0.as_slice()
    }
}

impl PartialEq for EmbeddingKey {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(a, b)| (a - b).abs() < f32::EPSILON)
    }
}

impl Eq for EmbeddingKey {}

impl Hash for EmbeddingKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for &v in self.0.iter() {
            let truncated = (v / f32::EPSILON).trunc() as i32;
            truncated.hash(state);
        }
    }
}

impl From<Vec<f32>> for EmbeddingKey {
    fn from(v: Vec<f32>) -> Self {
        Self::new(v)
    }
}
