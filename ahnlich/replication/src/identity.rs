use serde::{Deserialize, Serialize};

// Bump when binaries using different values must not participate in the same
// cluster because their cluster-facing protocol or membership semantics are
// incompatible. Do not bump for internal refactors or routine dependency updates.
pub const REPLICATION_PROTOCOL_VERSION: u32 = 1;

// Bump when a binary cannot safely restore a snapshot or replay Raft entries
// produced by the previous format. Backward-compatible decoders do not require
// a bump; add a fixture-based compatibility test instead.
pub const STATE_MACHINE_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClusterIdentity {
    pub id: String,
    pub name: Option<String>,
    pub replication_protocol_version: u32,
    pub state_machine_format_version: u32,
}

impl ClusterIdentity {
    pub fn new(id: String, name: Option<String>) -> Self {
        Self {
            id,
            name,
            replication_protocol_version: REPLICATION_PROTOCOL_VERSION,
            state_machine_format_version: STATE_MACHINE_FORMAT_VERSION,
        }
    }

    pub fn ensure_supported_by_current_binary(&self) -> Result<(), String> {
        if self.replication_protocol_version != REPLICATION_PROTOCOL_VERSION {
            return Err(format!(
                "unsupported replication protocol version: cluster={}, binary={}",
                self.replication_protocol_version, REPLICATION_PROTOCOL_VERSION,
            ));
        }

        if self.state_machine_format_version != STATE_MACHINE_FORMAT_VERSION {
            return Err(format!(
                "unsupported state machine format version: cluster={}, binary={}",
                self.state_machine_format_version, STATE_MACHINE_FORMAT_VERSION,
            ));
        }

        Ok(())
    }
}

pub trait ClusterIdentityProvider: Send + Sync + 'static {
    fn cluster_identity(&self) -> Result<Option<ClusterIdentity>, String>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_unsupported_replication_protocol_version() {
        let mut identity = ClusterIdentity::new("cluster-1".to_owned(), None);
        identity.replication_protocol_version += 1;

        assert!(identity.ensure_supported_by_current_binary().is_err());
    }

    #[test]
    fn rejects_an_unsupported_state_machine_format_version() {
        let mut identity = ClusterIdentity::new("cluster-1".to_owned(), None);
        identity.state_machine_format_version += 1;

        assert!(identity.ensure_supported_by_current_binary().is_err());
    }
}
