use serde::{Deserialize, Serialize};
use std::fmt;

/// A logical namespace for grouping stores within ahnlich.
///
/// Schemas provide multi-tenant isolation and organizational hierarchy
/// on top of the flat store namespace. Every store belongs to exactly
/// one schema. The schema `"public"` is the default used when none is
/// specified, preserving backward compatibility with existing stores.
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct Schema(String);

impl Schema {
    /// The default schema name used when no schema is specified.
    pub const DEFAULT_NAME: &'static str = "public";

    /// Creates a new `Schema` with the given name.
    ///
    /// # Panics
    ///
    /// Panics if `name` is empty.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        assert!(!name.is_empty(), "Schema name cannot be empty");
        Self(name)
    }

    /// Returns the schema name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the schema, returning the inner string.
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl Default for Schema {
    fn default() -> Self {
        Self(Self::DEFAULT_NAME.to_string())
    }
}

impl fmt::Display for Schema {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for Schema {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}

impl From<&str> for Schema {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}

impl AsRef<str> for Schema {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
