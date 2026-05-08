//! Cluster topology helpers shared between DB and AI servers.
//!
//! `cluster_topology` queries an openraft handle and returns a
//! `Vec<ClusterNodeInfo>` describing every node in the current effective
//! membership, its role, and per-node health metrics. The DB and AI public
//! services map this into the `ahnlich_types::shared::cluster::ClusterNode`
//! proto type.
//!
//! Standalone (non-clustered) nodes synthesize a single-entry response so
//! callers can use the same `ClusterInfo` query shape regardless of deployment
//! mode. Because standalone mode has no Raft cluster or election state, the
//! server crates map that single node onto [`NodeRole::Leader`] as an API
//! convenience rather than as a literal Raft role.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use openraft::{Raft, RaftTypeConfig};
use serde::{Deserialize, Serialize};
use tonic::Request;

use crate::node::ReplicationNode;
use crate::proto::cluster_admin::cluster_admin_service_client::ClusterAdminServiceClient;
use crate::proto::cluster_admin::{GetMetricsRequest, GetMetricsResponse};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRole {
    Leader,
    Follower,
    Learner,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeHealthStatus {
    Healthy,
    Unreachable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterNodeInfo {
    pub node_id: u64,
    /// Cluster-traffic address (Raft RPCs and admin).
    pub raft_addr: String,
    /// Public client-facing address (the value of `--port`).
    pub service_addr: String,
    pub role: NodeRole,
    pub health: NodeHealthStatus,
    pub term: Option<u64>,
    pub commit_index: Option<u64>,
}

#[derive(Debug, Clone)]
struct ClusterMetricsSnapshot {
    leader_id: Option<u64>,
    current_term: u64,
    last_applied_index: Option<u64>,
    voter_ids: BTreeSet<u64>,
    nodes: BTreeMap<u64, ReplicationNode>,
}

/// Build the full topology view for a clustered node.
///
/// Returns one entry per node in the current effective membership. Roles and
/// node metadata are derived from an authoritative membership view: we prefer
/// the current leader's metrics when reachable, falling back to the local
/// node's metrics otherwise. Per-node term and commit index are probed from
/// each node's own admin `GetMetrics` RPC. Failed probes mark the node
/// [`NodeHealthStatus::Unreachable`] and leave those fields unset.
pub async fn cluster_topology<C: RaftTypeConfig>(raft: &Raft<C>) -> Vec<ClusterNodeInfo>
where
    C::NodeId: Into<u64> + Copy,
    C::Node: Into<ReplicationNode> + Clone,
{
    let local_metrics = snapshot_local_metrics(raft);
    let authority = authoritative_metrics(&local_metrics).await.ok();
    let baseline = choose_membership_baseline(&local_metrics, authority.as_ref());

    let mut probes = BTreeMap::new();
    for (node_id, node) in &baseline.nodes {
        probes.insert(*node_id, fetch_node_metrics(node).await.ok());
    }

    build_topology(&baseline, probes)
}

fn sorted_nodes(nodes: BTreeMap<u64, ReplicationNode>) -> Vec<(u64, ReplicationNode)> {
    nodes.into_iter().collect()
}

fn normalize_target(addr: &str) -> String {
    if addr.starts_with("http://") || addr.starts_with("https://") {
        addr.to_owned()
    } else {
        format!("http://{addr}")
    }
}

fn role_for_node(node_id: u64, leader_id: Option<u64>, voters: &BTreeSet<u64>) -> NodeRole {
    if Some(node_id) == leader_id {
        NodeRole::Leader
    } else if voters.contains(&node_id) {
        NodeRole::Follower
    } else {
        NodeRole::Learner
    }
}

fn choose_membership_baseline(
    local: &ClusterMetricsSnapshot,
    authority: Option<&ClusterMetricsSnapshot>,
) -> ClusterMetricsSnapshot {
    match authority {
        Some(authority) if !authority.nodes.is_empty() => authority.clone(),
        _ => local.clone(),
    }
}

fn build_topology(
    baseline: &ClusterMetricsSnapshot,
    probes: BTreeMap<u64, Option<ClusterMetricsSnapshot>>,
) -> Vec<ClusterNodeInfo> {
    let leader_id = baseline.leader_id;
    let voters = &baseline.voter_ids;
    let nodes = sorted_nodes(baseline.nodes.clone());

    let mut topology = Vec::with_capacity(nodes.len());
    for (node_id, node) in nodes {
        let role = role_for_node(node_id, leader_id, voters);
        let (health, term, commit_index) =
            match probes.get(&node_id).and_then(|probe| probe.as_ref()) {
                Some(metrics) => (
                    NodeHealthStatus::Healthy,
                    Some(metrics.current_term),
                    metrics.last_applied_index,
                ),
                None => (NodeHealthStatus::Unreachable, None, None),
            };

        topology.push(ClusterNodeInfo {
            node_id,
            raft_addr: node.raft_addr,
            service_addr: node.service_addr,
            role,
            health,
            term,
            commit_index,
        });
    }

    topology
}

fn snapshot_local_metrics<C: RaftTypeConfig>(raft: &Raft<C>) -> ClusterMetricsSnapshot
where
    C::NodeId: Into<u64> + Copy,
    C::Node: Into<ReplicationNode> + Clone,
{
    let metrics = raft.metrics().borrow().clone();
    ClusterMetricsSnapshot {
        leader_id: metrics.current_leader.map(Into::into),
        current_term: metrics.current_term,
        last_applied_index: metrics.last_applied.map(|log_id| log_id.index),
        voter_ids: metrics
            .membership_config
            .membership()
            .voter_ids()
            .map(Into::into)
            .collect(),
        nodes: metrics
            .membership_config
            .nodes()
            .map(|(node_id, node)| ((*node_id).into(), node.clone().into()))
            .collect(),
    }
}

async fn authoritative_metrics(
    local_metrics: &ClusterMetricsSnapshot,
) -> Result<ClusterMetricsSnapshot, tonic::Status> {
    let Some(leader_id) = local_metrics.leader_id else {
        return Err(tonic::Status::failed_precondition("current leader unknown"));
    };

    let Some(leader) = local_metrics.nodes.get(&leader_id) else {
        return Err(tonic::Status::failed_precondition(
            "leader missing from membership metadata",
        ));
    };

    fetch_node_metrics(leader).await
}

fn snapshot_remote_metrics(response: GetMetricsResponse) -> ClusterMetricsSnapshot {
    ClusterMetricsSnapshot {
        leader_id: response.leader_id,
        current_term: response.current_term,
        last_applied_index: response.last_applied_index,
        voter_ids: response.voter_ids.into_iter().collect(),
        nodes: response
            .nodes
            .into_iter()
            .map(|node| (node.id, ReplicationNode::from(node)))
            .collect(),
    }
}

async fn fetch_node_metrics(
    node: &ReplicationNode,
) -> Result<ClusterMetricsSnapshot, tonic::Status> {
    let mut client = ClusterAdminServiceClient::connect(normalize_target(&node.raft_addr))
        .await
        .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
    let response = client
        .get_metrics(Request::new(GetMetricsRequest {}))
        .await
        .map_err(|e| tonic::Status::unavailable(e.to_string()))?;
    Ok(snapshot_remote_metrics(response.into_inner()))
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::{
        ClusterMetricsSnapshot, NodeHealthStatus, NodeRole, build_topology,
        choose_membership_baseline,
    };
    use crate::node::ReplicationNode;

    fn node(id: u64) -> ReplicationNode {
        ReplicationNode {
            raft_addr: format!("127.0.0.1:{}", 9_000 + id),
            service_addr: format!("127.0.0.1:{}", 8_000 + id),
        }
    }

    #[test]
    fn choose_membership_baseline_falls_back_to_local_when_authority_has_no_nodes() {
        let local = ClusterMetricsSnapshot {
            leader_id: Some(2),
            current_term: 5,
            last_applied_index: Some(10),
            voter_ids: BTreeSet::from([2, 3]),
            nodes: BTreeMap::from([(2, node(2)), (3, node(3))]),
        };
        let authority = ClusterMetricsSnapshot {
            leader_id: Some(2),
            current_term: 5,
            last_applied_index: Some(10),
            voter_ids: BTreeSet::from([2, 3]),
            nodes: BTreeMap::new(),
        };

        let baseline = choose_membership_baseline(&local, Some(&authority));
        assert_eq!(baseline.nodes, local.nodes);
        assert_eq!(baseline.voter_ids, local.voter_ids);
    }

    #[test]
    fn build_topology_keeps_unreachable_nodes_in_membership() {
        let baseline = ClusterMetricsSnapshot {
            leader_id: Some(2),
            current_term: 7,
            last_applied_index: Some(20),
            voter_ids: BTreeSet::from([2, 3]),
            nodes: BTreeMap::from([(2, node(2)), (3, node(3)), (9, node(9))]),
        };
        let probes = BTreeMap::from([
            (
                2,
                Some(ClusterMetricsSnapshot {
                    leader_id: Some(2),
                    current_term: 7,
                    last_applied_index: Some(20),
                    voter_ids: BTreeSet::from([2, 3]),
                    nodes: BTreeMap::new(),
                }),
            ),
            (
                3,
                Some(ClusterMetricsSnapshot {
                    leader_id: Some(2),
                    current_term: 7,
                    last_applied_index: Some(19),
                    voter_ids: BTreeSet::from([2, 3]),
                    nodes: BTreeMap::new(),
                }),
            ),
            (9, None),
        ]);

        let topology = build_topology(&baseline, probes);
        assert_eq!(topology.len(), 3);

        let leader = topology
            .iter()
            .find(|node| node.node_id == 2)
            .expect("leader node");
        assert_eq!(leader.role, NodeRole::Leader);
        assert_eq!(leader.health, NodeHealthStatus::Healthy);
        assert_eq!(leader.term, Some(7));
        assert_eq!(leader.commit_index, Some(20));

        let follower = topology
            .iter()
            .find(|node| node.node_id == 3)
            .expect("follower node");
        assert_eq!(follower.role, NodeRole::Follower);
        assert_eq!(follower.health, NodeHealthStatus::Healthy);
        assert_eq!(follower.term, Some(7));
        assert_eq!(follower.commit_index, Some(19));

        let learner = topology
            .iter()
            .find(|node| node.node_id == 9)
            .expect("learner node");
        assert_eq!(learner.role, NodeRole::Learner);
        assert_eq!(learner.health, NodeHealthStatus::Unreachable);
        assert_eq!(learner.term, None);
        assert_eq!(learner.commit_index, None);
    }
}
