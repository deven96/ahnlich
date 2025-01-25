use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug, Clone)]
pub struct VecF32Ordered(pub Vec<f32>);

impl PartialEq for VecF32Ordered {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(a, b)| (a - b).abs() < f32::EPSILON)
    }
}

impl Eq for VecF32Ordered {}

impl Hash for VecF32Ordered {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for &value in self.0.iter() {
            let truncated = (value / f32::EPSILON).trunc() as i32;
            truncated.hash(state);
        }
    }
}
