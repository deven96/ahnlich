use std::collections::{BTreeMap, BTreeSet};
use std::net::SocketAddr;
use std::sync::Arc;

use openraft::{Raft, RaftTypeConfig};
use tonic::{Request, Response, Status};

use crate::identity::{ClusterIdentity, ClusterIdentityProvider};
use crate::node::ReplicationNode;
use crate::proto::cluster_admin::cluster_admin_service_server::ClusterAdminService;
use crate::proto::cluster_admin::{
    AdmitLearnerRequest, AdmitLearnerResponse, CandidateNode, GetClusterIdentityRequest,
    GetClusterIdentityResponse, GetLeaderRequest, GetLeaderResponse, GetMetricsRequest,
    GetMetricsResponse, InitClusterRequest, InitClusterResponse, NodeInfo, PromoteLearnerRequest,
    PromoteLearnerResponse, RemoveNodeRequest, RemoveNodeResponse, TriggerSnapshotRequest,
    TriggerSnapshotResponse,
};

pub struct ClusterAdmin<C: RaftTypeConfig> {
    raft: Arc<Raft<C>>,
    identity_provider: Arc<dyn ClusterIdentityProvider>,
}

impl<C: RaftTypeConfig> ClusterAdmin<C> {
    pub fn new(raft: Arc<Raft<C>>, identity_provider: Arc<dyn ClusterIdentityProvider>) -> Self {
        Self {
            raft,
            identity_provider,
        }
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

    #[allow(clippy::result_large_err)]
    fn cluster_identity(&self) -> Result<ClusterIdentity, Status> {
        self.identity_provider
            .cluster_identity()
            .map_err(Status::failed_precondition)?
            .ok_or_else(|| Status::failed_precondition("cluster identity is not initialized"))
    }

    #[allow(clippy::result_large_err)]
    fn validate_candidate(&self, candidate: &CandidateNode) -> Result<ReplicationNode, Status>
    where
        C::NodeId: Into<u64> + Copy,
        C::Node: Into<ReplicationNode> + Clone,
    {
        let node = candidate
            .node
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("missing node in candidate"))?;

        if node.id == 0 {
            return Err(Status::invalid_argument(
                "candidate node id must not be zero",
            ));
        }

        let raft_addr: SocketAddr = node
            .raft_addr
            .parse()
            .map_err(|_| Status::invalid_argument("candidate raft_addr must be host:port"))?;
        let service_addr: SocketAddr = node
            .service_addr
            .parse()
            .map_err(|_| Status::invalid_argument("candidate service_addr must be host:port"))?;

        if raft_addr.ip().is_unspecified() {
            return Err(Status::invalid_argument(
                "candidate raft_addr must be an advertised, routable address",
            ));
        }

        if service_addr.ip().is_unspecified() {
            return Err(Status::invalid_argument(
                "candidate service_addr must be an advertised, routable address",
            ));
        }

        let identity = self.cluster_identity()?;
        if candidate.replication_protocol_version != identity.replication_protocol_version {
            return Err(Status::failed_precondition(format!(
                "replication protocol version mismatch: cluster={}, candidate={}",
                identity.replication_protocol_version, candidate.replication_protocol_version,
            )));
        }

        if candidate.state_machine_format_version != identity.state_machine_format_version {
            return Err(Status::failed_precondition(format!(
                "state machine format version mismatch: cluster={}, candidate={}",
                identity.state_machine_format_version, candidate.state_machine_format_version,
            )));
        }

        let metrics = self.raft.metrics().borrow().clone();
        for (existing_id, existing_node) in metrics.membership_config.nodes() {
            let existing: ReplicationNode = existing_node.clone().into();

            if (*existing_id).into() == node.id
                && (existing.raft_addr != node.raft_addr
                    || existing.service_addr != node.service_addr)
            {
                return Err(Status::failed_precondition(format!(
                    "candidate node id {} is already registered with different endpoints",
                    node.id,
                )));
            }

            if (*existing_id).into() != node.id
                && (existing.raft_addr == node.raft_addr
                    || existing.service_addr == node.service_addr)
            {
                return Err(Status::failed_precondition(format!(
                    "candidate endpoint is already registered to node {}",
                    (*existing_id).into(),
                )));
            }
        }

        Ok(ReplicationNode::from(node))
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

    async fn admit_learner(
        &self,
        request: Request<AdmitLearnerRequest>,
    ) -> Result<Response<AdmitLearnerResponse>, Status> {
        let candidate = request
            .into_inner()
            .candidate
            .ok_or_else(|| Status::invalid_argument("missing candidate"))?;
        let node_id = candidate
            .node
            .as_ref()
            .ok_or_else(|| Status::invalid_argument("missing node in candidate"))?
            .id;
        let node = self.validate_candidate(&candidate)?;

        self.raft
            .add_learner(node_id.into(), node.into(), true)
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        Ok(Response::new(AdmitLearnerResponse {}))
    }

    async fn promote_learner(
        &self,
        request: Request<PromoteLearnerRequest>,
    ) -> Result<Response<PromoteLearnerResponse>, Status> {
        let node_id = request.into_inner().node_id.into();
        let metrics = self.raft.metrics().borrow().clone();
        let membership = metrics.membership_config.membership();

        if membership.get_node(&node_id).is_none() {
            return Err(Status::invalid_argument("cannot promote an unknown node"));
        }

        let mut voters: BTreeSet<C::NodeId> = membership.voter_ids().collect();
        if !voters.insert(node_id) {
            return Err(Status::failed_precondition("node is already a voter"));
        }

        self.raft
            .change_membership(voters, false)
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        Ok(Response::new(PromoteLearnerResponse {}))
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

    async fn get_cluster_identity(
        &self,
        _request: Request<GetClusterIdentityRequest>,
    ) -> Result<Response<GetClusterIdentityResponse>, Status> {
        let identity = self.cluster_identity()?;

        Ok(Response::new(GetClusterIdentityResponse {
            cluster_id: identity.id,
            cluster_name: identity.name,
            replication_protocol_version: identity.replication_protocol_version,
            state_machine_format_version: identity.state_machine_format_version,
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
