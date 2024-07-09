#[derive(Debug)]
pub enum Error {
    DimensionMisMatch { expected: usize, found: usize },
    ImpossibleDepth { maximum: usize, found: usize },
}
