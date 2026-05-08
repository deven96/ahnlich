use openraft::BasicNode;
use serde::{Deserialize, Serialize};

use crate::proto::cluster_admin::NodeInfo;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplicationNode {
    pub raft_addr: String,
    pub service_addr: String,
}

impl From<NodeInfo> for ReplicationNode {
    fn from(value: NodeInfo) -> Self {
        Self {
            raft_addr: value.raft_addr,
            service_addr: value.service_addr,
        }
    }
}

impl From<&NodeInfo> for ReplicationNode {
    fn from(value: &NodeInfo) -> Self {
        Self {
            raft_addr: value.raft_addr.clone(),
            service_addr: value.service_addr.clone(),
        }
    }
}

impl From<ReplicationNode> for BasicNode {
    fn from(value: ReplicationNode) -> Self {
        BasicNode::new(value.raft_addr)
    }
}
