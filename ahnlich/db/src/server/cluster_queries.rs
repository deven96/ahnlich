use crate::engine::operations;
use crate::engine::store::StoreHandler;
use crate::errors::ServerError;
use crate::server::cluster::ClusterRuntime;
use ahnlich_replication::cluster_info;
use ahnlich_replication::node::ReplicationNode;
use ahnlich_types::db::server;
use ahnlich_types::shared::cluster::{
    ClusterInfoResponse, ClusterNode, NodeHealthStatus as PublicNodeHealthStatus,
    NodeRole as PublicNodeRole,
};
use openraft::error::RaftError;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::Arc;

fn map_cluster_role(role: cluster_info::NodeRole) -> i32 {
    match role {
        cluster_info::NodeRole::Leader => PublicNodeRole::Leader as i32,
        cluster_info::NodeRole::Follower => PublicNodeRole::Follower as i32,
        cluster_info::NodeRole::Learner => PublicNodeRole::Learner as i32,
    }
}

fn map_cluster_health(health: cluster_info::NodeHealthStatus) -> i32 {
    match health {
        cluster_info::NodeHealthStatus::Healthy => PublicNodeHealthStatus::Healthy as i32,
        cluster_info::NodeHealthStatus::Unreachable => PublicNodeHealthStatus::Unreachable as i32,
    }
}

pub(crate) fn map_storage_error(context: &str, err: openraft::StorageError<u64>) -> tonic::Status {
    tonic::Status::internal(format!("{context}: {err}"))
}

pub(crate) fn map_linearizable_error(
    err: RaftError<u64, openraft::error::CheckIsLeaderError<u64, ReplicationNode>>,
) -> tonic::Status {
    if let Some(forward) = err.forward_to_leader() {
        let leader = forward
            .leader_node
            .as_ref()
            .map(|node| node.service_addr.as_str())
            .unwrap_or("unknown leader");
        tonic::Status::failed_precondition(format!(
            "ListStores must be served by the leader in cluster mode; retry against {leader}",
        ))
    } else {
        tonic::Status::internal(format!("failed to linearize ListStores: {err}"))
    }
}

#[allow(clippy::result_large_err)]
pub(crate) fn read_store_handler<R>(
    cluster: Option<&ClusterRuntime>,
    store_handler: &Arc<StoreHandler>,
    f: impl FnOnce(&StoreHandler) -> Result<R, ServerError>,
) -> Result<R, tonic::Status> {
    if let Some(cluster) = cluster {
        cluster
            .state_machine
            .with_handler(|handler| f(handler.store_handler()))
            .map_err(|err| map_storage_error("failed to access clustered state machine", err))?
            .map_err(Into::into)
    } else {
        f(store_handler).map_err(Into::into)
    }
}

pub(crate) async fn list_stores_response(
    cluster: Option<&ClusterRuntime>,
    store_handler: &Arc<StoreHandler>,
) -> Result<server::StoreList, tonic::Status> {
    if let Some(cluster) = cluster {
        cluster
            .raft
            .ensure_linearizable()
            .await
            .map_err(map_linearizable_error)?;
    }

    read_store_handler(cluster, store_handler, |store_handler| {
        Ok(operations::list_stores(store_handler))
    })
}

pub(crate) async fn cluster_info_response(
    listener_addr: IoResult<SocketAddr>,
    cluster: Option<&ClusterRuntime>,
) -> Result<ClusterInfoResponse, tonic::Status> {
    if let Some(cluster) = cluster {
        let nodes = cluster_info::cluster_topology(cluster.raft.as_ref())
            .await
            .into_iter()
            .map(|node| ClusterNode {
                node_id: node.node_id,
                addr: node.service_addr,
                role: map_cluster_role(node.role),
                health: map_cluster_health(node.health),
                term: node.term,
                commit_index: node.commit_index,
            })
            .collect();

        Ok(ClusterInfoResponse { nodes })
    } else {
        Ok(ClusterInfoResponse {
            nodes: vec![ClusterNode {
                node_id: 1,
                addr: listener_addr?.to_string(),
                role: PublicNodeRole::Leader as i32,
                health: PublicNodeHealthStatus::Healthy as i32,
                term: None,
                commit_index: None,
            }],
        })
    }
}
