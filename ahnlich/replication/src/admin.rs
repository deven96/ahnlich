use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::sync::{Arc, Mutex};

use bincode::serialize;
use openraft::{Raft, RaftTypeConfig};
use tonic::{Request, Response, Status};

use crate::proto::cluster_admin::cluster_admin_service_server::ClusterAdminService;
use crate::proto::cluster_admin::{
    AddLearnerRequest, AddLearnerResponse, ChangeMembershipRequest, ChangeMembershipResponse,
    GetLeaderRequest, GetLeaderResponse, GetMetricsRequest, GetMetricsResponse, InitClusterRequest,
    InitClusterResponse, NodeInfo, RemoveNodeRequest, RemoveNodeResponse, TriggerSnapshotRequest,
    TriggerSnapshotResponse,
};

#[derive(Debug, Clone)]
pub struct NodeRegistry {
    inner: Arc<Mutex<HashMap<u64, NodeInfo>>>,
}

impl NodeRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn insert_all(&self, nodes: &[NodeInfo]) {
        let mut inner = self.inner.lock().expect("node registry lock poisoned");
        for n in nodes {
            inner.insert(n.id, n.clone());
        }
    }

    pub fn insert(&self, node: &NodeInfo) {
        let mut inner = self.inner.lock().expect("node registry lock poisoned");
        inner.insert(node.id, node.clone());
    }

    pub fn remove(&self, node_id: u64) {
        let mut inner = self.inner.lock().expect("node registry lock poisoned");
        inner.remove(&node_id);
    }

    pub fn get(&self, node_id: u64) -> Option<NodeInfo> {
        let inner = self.inner.lock().expect("node registry lock poisoned");
        inner.get(&node_id).cloned()
    }

    pub fn ids(&self) -> BTreeSet<u64> {
        let inner = self.inner.lock().expect("node registry lock poisoned");
        inner.keys().copied().collect()
    }
}

pub struct ClusterAdmin<C: RaftTypeConfig> {
    raft: Arc<Raft<C>>,
    registry: NodeRegistry,
}

impl<C: RaftTypeConfig> ClusterAdmin<C> {
    pub fn new(raft: Arc<Raft<C>>, registry: NodeRegistry) -> Self {
        Self { raft, registry }
    }

    fn build_nodes(nodes: &[NodeInfo]) -> BTreeMap<C::NodeId, C::Node>
    where
        C::NodeId: From<u64>,
        C::Node: From<openraft::BasicNode>,
    {
        nodes
            .iter()
            .map(|n| {
                let node = openraft::BasicNode {
                    addr: n.raft_addr.clone(),
                };
                (n.id.into(), node.into())
            })
            .collect()
    }
}

#[tonic::async_trait]
impl<C: RaftTypeConfig> ClusterAdminService for ClusterAdmin<C>
where
    C::NodeId: From<u64> + Into<u64> + Copy,
    C::Node: From<openraft::BasicNode>,
    C: RaftTypeConfig<Responder = openraft::impls::OneshotResponder<C>>,
{
    async fn init_cluster(
        &self,
        request: Request<InitClusterRequest>,
    ) -> Result<Response<InitClusterResponse>, Status> {
        let nodes = request.into_inner().nodes;
        self.registry.insert_all(&nodes);
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

        self.registry.insert(&node);
        let basic = openraft::BasicNode {
            addr: node.raft_addr.clone(),
        };
        self.raft
            .add_learner(node.id.into(), basic.into(), true)
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        Ok(Response::new(AddLearnerResponse {}))
    }

    async fn change_membership(
        &self,
        request: Request<ChangeMembershipRequest>,
    ) -> Result<Response<ChangeMembershipResponse>, Status> {
        let set: BTreeSet<C::NodeId> = request
            .into_inner()
            .node_ids
            .into_iter()
            .map(Into::into)
            .collect();
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
        let mut ids = self.registry.ids();
        ids.remove(&node_id);
        let set: BTreeSet<C::NodeId> = ids.into_iter().map(Into::into).collect();
        self.raft
            .change_membership(set, false)
            .await
            .map_err(|e| Status::failed_precondition(e.to_string()))?;
        self.registry.remove(node_id);
        Ok(Response::new(RemoveNodeResponse {}))
    }

    async fn get_metrics(
        &self,
        _request: Request<GetMetricsRequest>,
    ) -> Result<Response<GetMetricsResponse>, Status> {
        let metrics = self.raft.metrics().borrow().clone();
        let payload = serialize(&metrics).map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(GetMetricsResponse { metrics: payload }))
    }

    async fn get_leader(
        &self,
        _request: Request<GetLeaderRequest>,
    ) -> Result<Response<GetLeaderResponse>, Status> {
        let leader = self.raft.current_leader().await;
        if let Some(id) = leader {
            let leader_id: u64 = id.into();
            if let Some(node) = self.registry.get(leader_id) {
                return Ok(Response::new(GetLeaderResponse {
                    leader_id,
                    leader_addr: node.admin_addr,
                }));
            }
        }

        Ok(Response::new(GetLeaderResponse {
            leader_id: 0,
            leader_addr: String::new(),
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
