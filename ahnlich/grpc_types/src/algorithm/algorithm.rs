// This file is @generated by prost-build.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum Algorithm {
    EuclideanDistance = 0,
    DotProductSimilarity = 1,
    CosineSimilarity = 2,
    KdTree = 3,
}
impl Algorithm {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            Self::EuclideanDistance => "EuclideanDistance",
            Self::DotProductSimilarity => "DotProductSimilarity",
            Self::CosineSimilarity => "CosineSimilarity",
            Self::KdTree => "KDTree",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "EuclideanDistance" => Some(Self::EuclideanDistance),
            "DotProductSimilarity" => Some(Self::DotProductSimilarity),
            "CosineSimilarity" => Some(Self::CosineSimilarity),
            "KDTree" => Some(Self::KdTree),
            _ => None,
        }
    }
}
