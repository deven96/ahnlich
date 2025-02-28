use bincode::config::DefaultOptions;
use bincode::config::Options;
use once_cell::sync::Lazy;
use serde::Deserialize;
use serde::Serialize;

pub static VERSION: Lazy<Version> = Lazy::new(|| {
    let version_string: &str = env!("CARGO_PKG_VERSION");
    match version_string.split('.').collect::<Vec<_>>()[..] {
        [major, minor, patch] => Some(Version {
            major: major
                .parse()
                .expect("Could not parse major portion of version"),
            minor: minor
                .parse()
                .expect("Could not parse minor portion of version"),
            patch: patch
                .parse()
                .expect("Could not parse patch portion of version"),
        }),
        _ => None,
    }
    .unwrap_or_else(|| panic!("Could not parse CARGO_PKG_VERSION into Version"))
});

#[derive(Debug, Serialize, Deserialize, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct Version {
    pub major: u8,
    pub minor: u16,
    pub patch: u16,
}

impl Version {
    pub fn deserialize_magic_bytes(bytes: &[u8]) -> Result<Self, bincode::Error> {
        let config = DefaultOptions::new()
            .with_fixint_encoding()
            .with_little_endian();
        config.deserialize(bytes)
    }

    /// what versions are compatible. For now we assume that the versions should always be exact
    /// but ultimately we want major versions being the same to be enough
    pub fn is_compatible(&self, other: &Self) -> bool {
        self.major == other.major
    }
}
