use ndarray::Array1;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Debug, Clone)]
pub struct Array1F32Ordered(pub Array1<f32>);

impl PartialEq for Array1F32Ordered {
    fn eq(&self, other: &Self) -> bool {
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(a, b)| (a - b).abs() < f32::EPSILON)
    }
}

impl Eq for Array1F32Ordered {}

impl Hash for Array1F32Ordered {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for &value in self.0.iter() {
            let truncated = (value / f32::EPSILON).trunc() as i32;
            truncated.hash(state);
        }
    }
}
