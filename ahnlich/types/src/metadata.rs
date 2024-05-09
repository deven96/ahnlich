/// New types for store metadata key and values
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct MetadataKey(String);
impl MetadataKey {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct MetadataValue(String);
impl MetadataValue {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}