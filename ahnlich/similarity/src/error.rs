#[derive(Debug)]
pub enum Error {
    DimensionMisMatch { expected: usize, found: usize },
}
