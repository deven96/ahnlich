use ahnlich_replication::proto::cluster_admin::cluster_admin_service_client::ClusterAdminServiceClient;
use ahnlich_replication::proto::cluster_admin::{
    AddLearnerRequest, ChangeMembershipRequest, GetLeaderRequest, GetMetricsRequest,
    InitClusterRequest, NodeInfo, RemoveNodeRequest, TriggerSnapshotRequest,
};
use tonic::Request;

use crate::config::cli::{
    ClusterAddLearner, ClusterChangeMembership, ClusterInit, ClusterJoin, ClusterLeader,
    ClusterMetrics, ClusterNodeInfo, ClusterRemove, ClusterSnapshot,
};

fn format_addr(addr: &str) -> String {
    if addr.starts_with("http://") || addr.starts_with("https://") {
        addr.to_string()
    } else {
        format!("http://{addr}")
    }
}

fn node_from_cli(node: &ClusterNodeInfo) -> NodeInfo {
    NodeInfo {
        id: node.node_id,
        raft_addr: node.raft_addr.clone(),
        admin_addr: node.admin_addr.clone(),
        service_addr: node.service_addr.clone(),
    }
}

fn node_from_string(input: &str) -> Result<NodeInfo, String> {
    let parts: Vec<&str> = input.split(',').collect();
    if parts.len() != 4 {
        return Err("node format must be id,raft_addr,admin_addr,service_addr".to_string());
    }
    let id = parts[0]
        .parse::<u64>()
        .map_err(|_| "invalid node id".to_string())?;
    Ok(NodeInfo {
        id,
        raft_addr: parts[1].to_string(),
        admin_addr: parts[2].to_string(),
        service_addr: parts[3].to_string(),
    })
}

pub async fn init_cluster(cmd: ClusterInit) -> Result<(), String> {
    let mut client = ClusterAdminServiceClient::connect(format_addr(&cmd.admin_addr))
        .await
        .map_err(|e| e.to_string())?;
    let mut nodes = Vec::new();
    for n in cmd.nodes {
        nodes.push(node_from_string(&n)?);
    }
    client
        .init_cluster(Request::new(InitClusterRequest { nodes }))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn join_cluster(cmd: ClusterJoin) -> Result<(), String> {
    let mut client = ClusterAdminServiceClient::connect(format_addr(&cmd.join))
        .await
        .map_err(|e| e.to_string())?;
    let node = node_from_cli(&cmd.node);
    client
        .add_learner(Request::new(AddLearnerRequest { node: Some(node) }))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn add_learner(cmd: ClusterAddLearner) -> Result<(), String> {
    let mut client = ClusterAdminServiceClient::connect(format_addr(&cmd.admin_addr))
        .await
        .map_err(|e| e.to_string())?;
    let node = node_from_cli(&cmd.node);
    client
        .add_learner(Request::new(AddLearnerRequest { node: Some(node) }))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn change_membership(cmd: ClusterChangeMembership) -> Result<(), String> {
    let mut client = ClusterAdminServiceClient::connect(format_addr(&cmd.admin_addr))
        .await
        .map_err(|e| e.to_string())?;
    client
        .change_membership(Request::new(ChangeMembershipRequest {
            node_ids: cmd.node_ids,
        }))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn remove_node(cmd: ClusterRemove) -> Result<(), String> {
    let mut client = ClusterAdminServiceClient::connect(format_addr(&cmd.admin_addr))
        .await
        .map_err(|e| e.to_string())?;
    client
        .remove_node(Request::new(RemoveNodeRequest {
            node_id: cmd.node_id,
        }))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn metrics(cmd: ClusterMetrics) -> Result<Vec<u8>, String> {
    let mut client = ClusterAdminServiceClient::connect(format_addr(&cmd.admin_addr))
        .await
        .map_err(|e| e.to_string())?;
    let resp = client
        .get_metrics(Request::new(GetMetricsRequest {}))
        .await
        .map_err(|e| e.to_string())?;
    Ok(resp.into_inner().metrics)
}

pub async fn leader(cmd: ClusterLeader) -> Result<(u64, String), String> {
    let mut client = ClusterAdminServiceClient::connect(format_addr(&cmd.admin_addr))
        .await
        .map_err(|e| e.to_string())?;
    let resp = client
        .get_leader(Request::new(GetLeaderRequest {}))
        .await
        .map_err(|e| e.to_string())?;
    let inner = resp.into_inner();
    Ok((inner.leader_id, inner.leader_addr))
}

pub async fn snapshot(cmd: ClusterSnapshot) -> Result<(), String> {
    let mut client = ClusterAdminServiceClient::connect(format_addr(&cmd.admin_addr))
        .await
        .map_err(|e| e.to_string())?;
    client
        .trigger_snapshot(Request::new(TriggerSnapshotRequest {}))
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}
