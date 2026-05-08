use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::sync::Arc;

use openraft::{Raft, RaftTypeConfig};
use tonic::{Request, Response, Status};

use crate::node::ReplicationNode;
use crate::proto::cluster_admin::cluster_admin_service_server::ClusterAdminService;
use crate::proto::cluster_admin::{
    AddLearnerRequest, AddLearnerResponse, ChangeMembershipRequest, ChangeMembershipResponse,
    GetLeaderRequest, GetLeaderResponse, GetMetricsRequest, GetMetricsResponse, InitClusterRequest,
    InitClusterResponse, NodeInfo, RemoveNodeRequest, RemoveNodeResponse, TriggerSnapshotRequest,
    TriggerSnapshotResponse,
};

pub struct ClusterAdmin<C: RaftTypeConfig> {
    raft: Arc<Raft<C>>,
}

impl<C: RaftTypeConfig> ClusterAdmin<C> {
    pub fn new(raft: Arc<Raft<C>>) -> Self {
        Self { raft }
    }

    fn build_nodes(nodes: &[NodeInfo]) -> BTreeMap<C::NodeId, C::Node>
    where
        C::NodeId: From<u64>,
        C::Node: From<ReplicationNode>,
    {
        nodes
            .iter()
            .map(|node| (node.id.into(), ReplicationNode::from(node).into()))
            .collect()
    }
}

#[tonic::async_trait]
impl<C: RaftTypeConfig> ClusterAdminService for ClusterAdmin<C>
where
    C::NodeId: From<u64> + Into<u64> + Copy,
    C::Node: From<ReplicationNode> + Into<ReplicationNode> + Clone,
    C: RaftTypeConfig<Responder = openraft::impls::OneshotResponder<C>>,
{
    async fn init_cluster(
        &self,
        request: Request<InitClusterRequest>,
    ) -> Result<Response<InitClusterResponse>, Status> {
        let nodes = request.into_inner().nodes;
        self.raft
            .initialize(Self::build_nodes(&nodes))
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        Ok(Response::new(InitClusterResponse {}))
    }

    async fn add_learner(
        &self,
        request: Request<AddLearnerRequest>,
    ) -> Result<Response<AddLearnerResponse>, Status> {
        let node = request
            .into_inner()
            .node
            .ok_or_else(|| Status::invalid_argument("missing node in AddLearnerRequest"))?;

        self.raft
            .add_learner(node.id.into(), ReplicationNode::from(node).into(), true)
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        Ok(Response::new(AddLearnerResponse {}))
    }

    async fn change_membership(
        &self,
        request: Request<ChangeMembershipRequest>,
    ) -> Result<Response<ChangeMembershipResponse>, Status> {
        let node_ids = request.into_inner().node_ids;
        let metrics = self.raft.metrics().borrow().clone();
        let known_nodes: BTreeSet<u64> = metrics
            .membership_config
            .nodes()
            .map(|(node_id, _)| (*node_id).into())
            .collect();

        let missing: Vec<u64> = node_ids
            .iter()
            .copied()
            .filter(|node_id| !known_nodes.contains(node_id))
            .collect();
        if !missing.is_empty() {
            return Err(Status::invalid_argument(format!(
                "cannot change membership with unknown node ids: {missing:?}"
            )));
        }

        let set: BTreeSet<C::NodeId> = node_ids.into_iter().map(Into::into).collect();
        self.raft
            .change_membership(set, false)
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        Ok(Response::new(ChangeMembershipResponse {}))
    }

    async fn remove_node(
        &self,
        request: Request<RemoveNodeRequest>,
    ) -> Result<Response<RemoveNodeResponse>, Status> {
        let node_id = request.into_inner().node_id;
        let metrics = self.raft.metrics().borrow().clone();
        let mut voters: BTreeSet<C::NodeId> =
            metrics.membership_config.membership().voter_ids().collect();
        voters.remove(&node_id.into());
        self.raft
            .change_membership(voters, false)
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        Ok(Response::new(RemoveNodeResponse {}))
    }

    async fn get_metrics(
        &self,
        _request: Request<GetMetricsRequest>,
    ) -> Result<Response<GetMetricsResponse>, Status> {
        let metrics = self.raft.metrics().borrow().clone();
        let leader_id = metrics.current_leader.map(Into::into);
        let last_applied_index = metrics.last_applied.map(|log_id| log_id.index);
        let voter_ids = metrics
            .membership_config
            .membership()
            .voter_ids()
            .map(Into::into)
            .collect();
        let nodes = metrics
            .membership_config
            .nodes()
            .map(|(node_id, node)| {
                let node: ReplicationNode = node.clone().into();
                NodeInfo {
                    id: (*node_id).into(),
                    raft_addr: node.raft_addr,
                    service_addr: node.service_addr,
                }
            })
            .collect();

        Ok(Response::new(GetMetricsResponse {
            leader_id,
            current_term: metrics.current_term,
            last_applied_index,
            voter_ids,
            nodes,
        }))
    }

    async fn get_leader(
        &self,
        _request: Request<GetLeaderRequest>,
    ) -> Result<Response<GetLeaderResponse>, Status> {
        let metrics = self.raft.metrics().borrow().clone();
        let leader_id = metrics
            .current_leader
            .ok_or_else(|| Status::failed_precondition("no elected leader"))?;
        let leader_node: ReplicationNode = metrics
            .membership_config
            .membership()
            .get_node(&leader_id)
            .cloned()
            .ok_or_else(|| Status::failed_precondition("leader missing from membership metadata"))?
            .into();

        Ok(Response::new(GetLeaderResponse {
            leader_id: leader_id.into(),
            leader_addr: leader_node.raft_addr,
        }))
    }

    async fn trigger_snapshot(
        &self,
        _request: Request<TriggerSnapshotRequest>,
    ) -> Result<Response<TriggerSnapshotResponse>, Status> {
        self.raft
            .trigger()
            .snapshot()
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        Ok(Response::new(TriggerSnapshotResponse {}))
    }
}
