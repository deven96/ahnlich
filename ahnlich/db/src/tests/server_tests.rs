use crate::server::handler::Server;
use crate::{cli::ServerConfig, errors::ServerError};
use ahnlich_replication::config::RaftStorageEngine;
use ahnlich_replication::proto::cluster_admin::cluster_admin_service_client::ClusterAdminServiceClient;
use ahnlich_replication::proto::cluster_admin::{
    AddLearnerRequest, ChangeMembershipRequest, GetMetricsRequest, InitClusterRequest, NodeInfo,
};
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::metadata::metadata_value::Value as MetadataValueEnum;
use ahnlich_types::predicates::{
    self, Predicate, PredicateCondition, predicate::Kind as PredicateKind,
    predicate_condition::Kind as PredicateConditionKind,
};
use ahnlich_types::server_types::ServerType;
use ahnlich_types::shared::info::StoreUpsert;
use ahnlich_types::similarity::Similarity;
use once_cell::sync::Lazy;
use openraft::{BasicNode, RaftMetrics, ServerState};
use pretty_assertions::assert_eq;
use std::collections::{HashMap, HashSet};
use std::net::{SocketAddr, TcpListener};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use utils::server::AhnlichServerUtils;

use ahnlich_types::{
    db::{
        pipeline::{self as db_pipeline, db_query::Query},
        query as db_query_types, server as db_response_types,
    },
    keyval::StoreName,
    services::db_service::db_service_client::DbServiceClient,
};
use bincode::deserialize;
use tonic::transport::Channel;

static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());

static CONFIG_WITH_MAX_CLIENTS: Lazy<ServerConfig> =
    Lazy::new(|| ServerConfig::default().os_select_port().maximum_clients(2));

static PERSISTENCE_FILE: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ahnlich.dat"));

static CONFIG_WITH_PERSISTENCE: Lazy<ServerConfig> = Lazy::new(|| {
    ServerConfig::default()
        .os_select_port()
        .persistence_interval(200)
        .persist_location((*PERSISTENCE_FILE).clone())
});

fn reserve_ephemeral_addr() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to reserve local port");
    listener
        .local_addr()
        .expect("failed to read reserved local addr")
}

struct ClusteredDbNode {
    node_id: u64,
    raft_addr: SocketAddr,
    admin_addr: SocketAddr,
    service_addr: SocketAddr,
    shutdown: CancellationToken,
    handle: JoinHandle<std::io::Result<()>>,
}

async fn cluster_admin_client(
    admin_addr: SocketAddr,
) -> ClusterAdminServiceClient<tonic::transport::Channel> {
    ClusterAdminServiceClient::connect(format!("http://{admin_addr}"))
        .await
        .expect("failed to connect cluster admin")
}

async fn db_client(service_addr: SocketAddr) -> DbServiceClient<tonic::transport::Channel> {
    DbServiceClient::connect(
        Channel::from_shared(format!("http://{service_addr}")).expect("Failed to get channel"),
    )
    .await
    .expect("Failure")
}

async fn wait_for_node_ready(admin_addr: SocketAddr, node_id: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let mut metrics_ok = false;
        if let Ok(mut admin) =
            ClusterAdminServiceClient::connect(format!("http://{admin_addr}")).await
        {
            metrics_ok = admin
                .get_metrics(tonic::Request::new(GetMetricsRequest {}))
                .await
                .is_ok();
        }

        if metrics_ok {
            return;
        }

        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for node readiness: node_id={node_id} admin={admin_addr}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn spawn_clustered_db_node(node_id: u64) -> ClusteredDbNode {
    let mut config = ServerConfig::default().os_select_port();
    config.cluster_enabled = true;
    config.raft_storage = RaftStorageEngine::Memory;
    config.raft_node_id = node_id;
    config.raft_join = None;
    let raft_addr = reserve_ephemeral_addr();
    let admin_addr = reserve_ephemeral_addr();
    config.raft_addr = raft_addr.to_string();
    config.admin_addr = admin_addr.to_string();

    let server = Server::new(&config)
        .await
        .expect("Could not initialize clustered db server");
    let service_addr = server.local_addr().expect("Could not get local addr");
    let shutdown = server.cancellation_token();
    let handle = tokio::spawn(async move { server.start().await });

    wait_for_node_ready(admin_addr, node_id).await;

    ClusteredDbNode {
        node_id,
        raft_addr,
        admin_addr,
        service_addr,
        shutdown,
        handle,
    }
}

async fn init_single_node_cluster(node: &ClusteredDbNode) {
    let mut admin = cluster_admin_client(node.admin_addr).await;
    admin
        .init_cluster(tonic::Request::new(InitClusterRequest {
            nodes: vec![NodeInfo {
                id: node.node_id,
                raft_addr: node.raft_addr.to_string(),
                admin_addr: node.admin_addr.to_string(),
                service_addr: node.service_addr.to_string(),
            }],
        }))
        .await
        .expect("init single-node cluster");
}

async fn init_three_node_cluster(nodes: &[ClusteredDbNode]) {
    let n1 = &nodes[0];
    let mut admin = cluster_admin_client(n1.admin_addr).await;
    admin
        .init_cluster(tonic::Request::new(InitClusterRequest {
            nodes: vec![NodeInfo {
                id: n1.node_id,
                raft_addr: n1.raft_addr.to_string(),
                admin_addr: n1.admin_addr.to_string(),
                service_addr: n1.service_addr.to_string(),
            }],
        }))
        .await
        .expect("init cluster");

    for n in &nodes[1..] {
        admin
            .add_learner(tonic::Request::new(AddLearnerRequest {
                node: Some(NodeInfo {
                    id: n.node_id,
                    raft_addr: n.raft_addr.to_string(),
                    admin_addr: n.admin_addr.to_string(),
                    service_addr: n.service_addr.to_string(),
                }),
            }))
            .await
            .expect("add learner");
    }

    admin
        .change_membership(tonic::Request::new(ChangeMembershipRequest {
            node_ids: nodes.iter().map(|n| n.node_id).collect(),
        }))
        .await
        .expect("change membership");
}

async fn wait_for_exact_voter_set(
    nodes: &[ClusteredDbNode],
    expected_voters: &HashSet<u64>,
    phase: &str,
) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let mut matching = 0usize;
        for node in nodes {
            let mut admin = cluster_admin_client(node.admin_addr).await;
            let metrics = admin
                .get_metrics(tonic::Request::new(GetMetricsRequest {}))
                .await
                .expect("get_metrics should succeed")
                .into_inner();
            let m: RaftMetrics<u64, BasicNode> =
                deserialize(&metrics.metrics).expect("metrics decode should succeed");
            let voters: HashSet<u64> = m.membership_config.membership().voter_ids().collect();
            if voters == *expected_voters {
                matching += 1;
            }
        }
        if matching == nodes.len() {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for exact voter set during phase={phase}"
        );
        tokio::time::sleep(Duration::from_millis(120)).await;
    }
}

async fn wait_for_leader_from_any(
    nodes: &[ClusteredDbNode],
    allowed: &HashSet<u64>,
    excluded: &HashSet<u64>,
    phase: &str,
) -> (u64, u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        for node in nodes {
            if !allowed.contains(&node.node_id) || excluded.contains(&node.node_id) {
                continue;
            }
            let Ok(mut admin_client) =
                ClusterAdminServiceClient::connect(format!("http://{}", node.admin_addr)).await
            else {
                continue;
            };
            let Ok(resp) = admin_client
                .get_metrics(tonic::Request::new(GetMetricsRequest {}))
                .await
            else {
                continue;
            };
            let m: RaftMetrics<u64, BasicNode> =
                deserialize(&resp.into_inner().metrics).expect("metrics decode should succeed");
            if m.state == ServerState::Leader
                && m.current_leader == Some(node.node_id)
                && allowed.contains(&node.node_id)
                && !excluded.contains(&node.node_id)
            {
                if db_client(node.service_addr)
                    .await
                    .ping(tonic::Request::new(db_query_types::Ping {}))
                    .await
                    .is_ok()
                {
                    return (node.node_id, m.current_term);
                }
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for leader during phase={phase}"
        );
        tokio::time::sleep(Duration::from_millis(120)).await;
    }
}

async fn wait_for_leader_replication_targets(
    leader_admin_addr: SocketAddr,
    expected_followers: &HashSet<u64>,
    phase: &str,
) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        let mut admin = cluster_admin_client(leader_admin_addr).await;
        let metrics = admin
            .get_metrics(tonic::Request::new(GetMetricsRequest {}))
            .await
            .expect("get_metrics should succeed")
            .into_inner();
        let m: RaftMetrics<u64, BasicNode> =
            deserialize(&metrics.metrics).expect("metrics decode should succeed");
        if m.state == ServerState::Leader {
            let replication_targets: HashSet<u64> = m
                .replication
                .as_ref()
                .map(|r| r.keys().copied().collect())
                .unwrap_or_default();
            if expected_followers.is_subset(&replication_targets) {
                return;
            }
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for replication targets during phase={phase}"
        );
        tokio::time::sleep(Duration::from_millis(120)).await;
    }
}

async fn wait_for_store_presence(service_addr: SocketAddr, store_name: &str, should_exist: bool) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let mut client = db_client(service_addr).await;
        let listed = match client
            .list_stores(tonic::Request::new(db_query_types::ListStores {}))
            .await
        {
            Ok(v) => v.into_inner(),
            Err(_) => {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "timed out waiting for store presence check: store={store_name} should_exist={should_exist}"
                );
                tokio::time::sleep(Duration::from_millis(120)).await;
                continue;
            }
        };
        let exists = listed.stores.iter().any(|s| s.name == store_name);
        if exists == should_exist {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for store presence check: store={store_name} should_exist={should_exist}"
        );
        tokio::time::sleep(Duration::from_millis(120)).await;
    }
}

fn assert_unavailable(err: tonic::Status, context: &str) {
    assert_eq!(
        err.code(),
        tonic::Code::Unavailable,
        "{context}: expected UNAVAILABLE, got {:?}: {}",
        err.code(),
        err.message()
    );
}

async fn wait_for_consistent_survivor_view(
    nodes: &[ClusteredDbNode],
    excluded: &HashSet<u64>,
    expected_voters: &HashSet<u64>,
    expected_leader: u64,
    phase: &str,
) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let mut all_match = true;
        let mut common_term: Option<u64> = None;
        for node in nodes {
            if excluded.contains(&node.node_id) {
                continue;
            }
            let mut admin = cluster_admin_client(node.admin_addr).await;
            let metrics = admin
                .get_metrics(tonic::Request::new(GetMetricsRequest {}))
                .await
                .expect("get_metrics should succeed")
                .into_inner();
            let m: RaftMetrics<u64, BasicNode> =
                deserialize(&metrics.metrics).expect("metrics decode should succeed");
            let voters: HashSet<u64> = m.membership_config.membership().voter_ids().collect();
            if voters != *expected_voters || m.current_leader != Some(expected_leader) {
                all_match = false;
                break;
            }
            match common_term {
                Some(term) if term != m.current_term => {
                    all_match = false;
                    break;
                }
                None => common_term = Some(m.current_term),
                _ => {}
            }
        }
        if all_match {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for consistent survivor view during phase={phase}"
        );
        tokio::time::sleep(Duration::from_millis(120)).await;
    }
}

async fn restart_clustered_db_node(node: &mut ClusteredDbNode, service_port: Option<u16>) {
    node.shutdown.cancel();
    node.handle.abort();
    tokio::time::sleep(Duration::from_millis(300)).await;

    let mut config = ServerConfig::default().os_select_port();
    config.cluster_enabled = true;
    config.raft_storage = RaftStorageEngine::Memory;
    config.raft_node_id = node.node_id;
    config.raft_join = None;
    config.raft_addr = node.raft_addr.to_string();
    config.admin_addr = node.admin_addr.to_string();
    if let Some(port) = service_port {
        config.port = port;
    } else {
        config.port = 0;
    }

    let server = Server::new(&config)
        .await
        .expect("Could not restart clustered db server");
    node.service_addr = server
        .local_addr()
        .expect("Could not get restarted local addr");
    node.shutdown = server.cancellation_token();
    node.handle = tokio::spawn(async move { server.start().await });

    wait_for_node_ready(node.admin_addr, node.node_id).await;
}

#[tokio::test]
async fn test_db_clustered_create_set_drop_via_raft() {
    let node = spawn_clustered_db_node(1).await;
    init_single_node_cluster(&node).await;

    let allowed_ids = HashSet::from([node.node_id]);
    let excluded_ids = HashSet::new();
    let (leader_id, _term) = wait_for_leader_from_any(
        std::slice::from_ref(&node),
        &allowed_ids,
        &excluded_ids,
        "single-node-init",
    )
    .await;
    assert_eq!(leader_id, node.node_id);

    let mut client = db_client(node.service_addr).await;
    client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "cluster-store-a".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
        }))
        .await
        .expect("create_store should succeed");

    wait_for_store_presence(node.service_addr, "cluster-store-a", true).await;

    let set = client
        .set(tonic::Request::new(db_query_types::Set {
            store: "cluster-store-a".to_string(),
            inputs: vec![DbStoreEntry {
                key: Some(StoreKey {
                    key: vec![1.0, 2.0, 3.0],
                }),
                value: Some(StoreValue {
                    value: HashMap::new(),
                }),
            }],
        }))
        .await
        .expect("set should succeed")
        .into_inner();
    assert!(set.upsert.is_some(), "set should return upsert metadata");

    let dropped = client
        .drop_store(tonic::Request::new(db_query_types::DropStore {
            store: "cluster-store-a".to_string(),
            error_if_not_exists: true,
        }))
        .await
        .expect("drop_store should succeed")
        .into_inner();
    assert_eq!(dropped.deleted_count, 1);

    wait_for_store_presence(node.service_addr, "cluster-store-a", false).await;

    node.shutdown.cancel();
    node.handle.abort();
}

#[tokio::test]
async fn test_db_cluster_replication_and_single_failover() {
    let mut nodes = vec![
        spawn_clustered_db_node(1).await,
        spawn_clustered_db_node(2).await,
        spawn_clustered_db_node(3).await,
    ];

    init_three_node_cluster(&nodes).await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    let allowed_ids: HashSet<u64> = nodes.iter().map(|n| n.node_id).collect();
    let expected_voters = allowed_ids.clone();
    let mut excluded_ids = HashSet::new();
    wait_for_exact_voter_set(
        &nodes,
        &expected_voters,
        "db-single-pre-failover-membership",
    )
    .await;

    let (leader_id, leader_term) =
        wait_for_leader_from_any(&nodes, &allowed_ids, &excluded_ids, "db-single-initial").await;
    let leader_idx = nodes
        .iter()
        .position(|n| n.node_id == leader_id)
        .expect("leader node not found");
    let leader_service = nodes[leader_idx].service_addr;

    let mut leader_client = db_client(leader_service).await;
    leader_client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "replicated_store".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
        }))
        .await
        .expect("leader create_store should succeed");

    let expected_followers: HashSet<u64> = nodes
        .iter()
        .filter(|n| n.node_id != leader_id)
        .map(|n| n.node_id)
        .collect();
    wait_for_leader_replication_targets(
        nodes[leader_idx].admin_addr,
        &expected_followers,
        "db-single-post-create-store",
    )
    .await;

    excluded_ids.insert(leader_id);
    nodes[leader_idx].shutdown.cancel();
    tokio::time::sleep(Duration::from_millis(300)).await;

    let (new_leader_id, new_leader_term) = wait_for_leader_from_any(
        &nodes,
        &allowed_ids,
        &excluded_ids,
        "db-single-post-first-failover",
    )
    .await;
    assert_ne!(new_leader_id, leader_id);
    assert!(new_leader_term > leader_term);
    let new_leader_service = nodes
        .iter()
        .find(|n| n.node_id == new_leader_id)
        .expect("new leader node not found")
        .service_addr;
    wait_for_store_presence(new_leader_service, "replicated_store", true).await;

    let mut new_leader_client = db_client(new_leader_service).await;
    let dropped = new_leader_client
        .drop_store(tonic::Request::new(db_query_types::DropStore {
            store: "replicated_store".to_string(),
            error_if_not_exists: true,
        }))
        .await
        .expect("drop_store on new leader should succeed")
        .into_inner();
    assert_eq!(dropped.deleted_count, 1);
    wait_for_store_presence(new_leader_service, "replicated_store", false).await;

    for n in &mut nodes {
        n.shutdown.cancel();
        n.handle.abort();
    }
}

#[tokio::test]
async fn test_db_cluster_replication_and_double_failover() {
    let mut nodes = vec![
        spawn_clustered_db_node(1).await,
        spawn_clustered_db_node(2).await,
        spawn_clustered_db_node(3).await,
    ];

    init_three_node_cluster(&nodes).await;
    tokio::time::sleep(Duration::from_secs(2)).await;

    let allowed_ids: HashSet<u64> = nodes.iter().map(|n| n.node_id).collect();
    let expected_voters = allowed_ids.clone();
    let mut excluded_ids = HashSet::new();
    wait_for_exact_voter_set(&nodes, &expected_voters, "db-double-pre-membership").await;

    let (leader_id_1, term_1) =
        wait_for_leader_from_any(&nodes, &allowed_ids, &excluded_ids, "db-double-initial").await;
    let leader_idx_1 = nodes
        .iter()
        .position(|n| n.node_id == leader_id_1)
        .expect("initial leader node not found");
    let leader_service_1 = nodes[leader_idx_1].service_addr;

    let mut leader_client_1 = db_client(leader_service_1).await;
    leader_client_1
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "replicated_store".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
        }))
        .await
        .expect("leader create_store should succeed");

    let expected_followers: HashSet<u64> = nodes
        .iter()
        .filter(|n| n.node_id != leader_id_1)
        .map(|n| n.node_id)
        .collect();
    wait_for_leader_replication_targets(
        nodes[leader_idx_1].admin_addr,
        &expected_followers,
        "db-double-post-create",
    )
    .await;
    wait_for_store_presence(leader_service_1, "replicated_store", true).await;

    excluded_ids.insert(leader_id_1);
    nodes[leader_idx_1].shutdown.cancel();
    tokio::time::sleep(Duration::from_millis(400)).await;

    let (leader_id_2, term_2) = wait_for_leader_from_any(
        &nodes,
        &allowed_ids,
        &excluded_ids,
        "db-double-post-first-failover",
    )
    .await;
    assert_ne!(leader_id_2, leader_id_1);
    assert!(term_2 > term_1);
    let leader_idx_2 = nodes
        .iter()
        .position(|n| n.node_id == leader_id_2)
        .expect("second leader node not found");
    let leader_service_2 = nodes[leader_idx_2].service_addr;
    wait_for_store_presence(leader_service_2, "replicated_store", true).await;

    let mut leader_client_2 = db_client(leader_service_2).await;
    let dropped = leader_client_2
        .drop_store(tonic::Request::new(db_query_types::DropStore {
            store: "replicated_store".to_string(),
            error_if_not_exists: true,
        }))
        .await
        .expect("drop_store on second leader should succeed")
        .into_inner();
    assert_eq!(dropped.deleted_count, 1);
    wait_for_store_presence(leader_service_2, "replicated_store", false).await;

    wait_for_consistent_survivor_view(
        &nodes,
        &excluded_ids,
        &expected_voters,
        leader_id_2,
        "db-double-before-second-failover",
    )
    .await;

    excluded_ids.insert(leader_id_2);
    nodes[leader_idx_2].shutdown.cancel();
    tokio::time::sleep(Duration::from_millis(400)).await;

    let lone_node = nodes
        .iter()
        .find(|n| !excluded_ids.contains(&n.node_id))
        .expect("expected one surviving node");
    let mut lone_client = db_client(lone_node.service_addr).await;

    let read_err = lone_client
        .list_stores(tonic::Request::new(db_query_types::ListStores {}))
        .await
        .expect_err("list_stores should fail without quorum");
    assert_unavailable(read_err, "list_stores without quorum");

    let write_err = lone_client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "should_fail_without_quorum".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
        }))
        .await
        .expect_err("create_store should fail without quorum");
    assert_unavailable(write_err, "create_store without quorum");

    let restart_target_id = leader_id_1;
    let restart_idx = nodes
        .iter()
        .position(|n| n.node_id == restart_target_id)
        .expect("restart target node not found");
    restart_clustered_db_node(&mut nodes[restart_idx], None).await;
    excluded_ids.remove(&restart_target_id);

    let (leader_id_3, _term_3) = wait_for_leader_from_any(
        &nodes,
        &allowed_ids,
        &excluded_ids,
        "db-double-post-quorum-restore",
    )
    .await;
    let leader_service_3 = nodes
        .iter()
        .find(|n| n.node_id == leader_id_3)
        .expect("restored leader not found")
        .service_addr;
    wait_for_store_presence(leader_service_3, "replicated_store", false).await;

    for n in &mut nodes {
        n.shutdown.cancel();
        n.handle.abort();
    }
}

#[tokio::test]
async fn test_grpc_ping_test() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let response = client
        .ping(tonic::Request::new(ahnlich_types::db::query::Ping {}))
        .await
        .expect("Failed to ping");

    let expected = db_response_types::Pong {};
    println!("Response: {response:?}");
    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_maximum_client_restriction_works() {
    let server = Server::new(&CONFIG_WITH_MAX_CLIENTS)
        .await
        .expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let mut first_client = DbServiceClient::connect(channel).await.expect("Failure");
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let mut second_client = DbServiceClient::connect(channel).await.expect("Failure");
    let response = second_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");
    assert_eq!(response.into_inner().clients.len(), 2);
    let response = first_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");

    assert_eq!(response.into_inner().clients.len(), 2);
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let mut third_client = DbServiceClient::connect(channel).await.expect("Failure");
    third_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect_err("third client failed to error with a max of 2");
    let response = second_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");
    assert_eq!(response.into_inner().clients.len(), 2);
    drop(first_client);
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let mut fourth_client = DbServiceClient::connect(channel).await.expect("Failure");
    let response = fourth_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");
    // The third client never connected so we expect only 2
    assert_eq!(response.into_inner().clients.len(), 2);
}

#[tokio::test]
async fn test_server_client_info() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");

    let mut first_client = DbServiceClient::connect(channel).await.expect("Failure");

    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let second_client = DbServiceClient::connect(channel).await.expect("Failure");

    let response = first_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");

    assert_eq!(response.into_inner().clients.len(), 2);
    drop(second_client);

    let response = first_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");
    assert_eq!(response.into_inner().clients.len(), 1);
}

#[tokio::test]
async fn test_simple_stores_list() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let response = client
        .list_stores(tonic::Request::new(ahnlich_types::db::query::ListStores {}))
        .await
        .expect("Failed to get store's list");

    let expected = db_response_types::StoreList { stores: vec![] };
    println!("Response: {response:?}");
    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_create_stores() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let queries = vec![
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to create stores");

    let error_response = ServerError::StoreAlreadyExists(StoreName {
        value: "Main".to_string(),
    });

    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: error_response.to_string(),
                    code: 6,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 0,
                        size_in_bytes: 1056,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_del_pred() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let queries = vec![
        db_pipeline::DbQuery {
            query: Some(Query::DelPred(db_query_types::DelPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "planet".into(),
                            value: Some(MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "earth".to_string(),
                                    ),
                                ),
                            }),
                        })),
                    })),
                }),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Should not error as it is correct query
        // but should delete nothing as nothing matches predicate
        db_pipeline::DbQuery {
            query: Some(Query::DelPred(db_query_types::DelPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "planet".into(),
                            value: Some(MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "earth".to_string(),
                                    ),
                                ),
                            }),
                        })),
                    })),
                }),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.4, 1.5],
                        }),
                        value: Some(ahnlich_types::keyval::StoreValue {
                            value: HashMap::from_iter([(
                                "planet".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("jupiter".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(ahnlich_types::keyval::StoreValue {
                            value: HashMap::from_iter([(
                                "planet".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("mars".into())),
                                },
                            )]),
                        }),
                    },
                ],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
        db_pipeline::DbQuery {
            query: Some(Query::DelPred(db_query_types::DelPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "planet".into(),
                            value: Some(MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "mars".to_string(),
                                    ),
                                ),
                            }),
                        })),
                    })),
                }),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![
                    StoreKey {
                        key: vec![1.4, 1.5],
                    },
                    StoreKey {
                        key: vec![1.6, 1.7],
                    },
                ],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::DelPred(db_query_types::DelPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "planet".into(),
                            value: Some(MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "mars".to_string(),
                                    ),
                                ),
                            }),
                        })),
                    })),
                }),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to create stores");

    let error_response = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: error_response.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 2,
                        size_in_bytes: 1264,
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "planet".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("mars".into())),
                                },
                            )]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 0,
                        size_in_bytes: 1056,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_del_key() {
    // Server setup (same as your gRPC examples)
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![],
            })),
        },
        // Create the store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 4,
                create_predicates: vec!["role".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Should not error but delete nothing (empty store)
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 1.1, 1.2, 1.3],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.1, 1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                ],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2], // 3 dimensions vs store's 4
                }],
            })),
        },
        // Should work - delete existing key
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 4,
        input_dimension: 3,
    };

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 2,
                        size_in_bytes: 1176,
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 1,
                        size_in_bytes: 1128,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_server_with_persistence() {
    // First server instance
    //
    // Clean up - delete persistence file
    let _ = std::fs::remove_file(&*PERSISTENCE_FILE);

    let server = Server::new(&CONFIG_WITH_PERSISTENCE)
        .await
        .expect("Failed to create server");
    let write_flag = server.write_flag();
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // First set of operations
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![],
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 4,
                create_predicates: vec!["role".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Should not error but delete nothing
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 1.1, 1.2, 1.3],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.1, 1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "role".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::Image(vec![1, 2, 3])),
                                },
                            )]),
                        }),
                    },
                ],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2],
                }],
            })),
        },
        // Should delete existing key
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 4,
        input_dimension: 3,
    };

    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 2,
                        size_in_bytes: 1304,
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3, // DimensionMismatch
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 1,
                        size_in_bytes: 1256,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
    assert!(write_flag.load(Ordering::SeqCst));

    // Wait for persistence
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Second server instance
    let server = Server::new(&CONFIG_WITH_PERSISTENCE)
        .await
        .expect("Failed to create server");
    let write_flag = server.write_flag();
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    // Verify persistence file exists and is not empty
    let file_metadata = std::fs::metadata(
        &CONFIG_WITH_PERSISTENCE
            .common
            .persist_location
            .clone()
            .unwrap(),
    )
    .unwrap();
    assert!(file_metadata.len() > 0, "The persistence file is empty");

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(200)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Second set of operations to verify persistence
    let queries = vec![
        // Should error as store exists from persistence
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec!["role".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Should not error as store exists
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![],
            })),
        },
        // Should get persisted data
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.1, 1.2, 1.3, 1.4],
                }],
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let already_exists_error = ServerError::StoreAlreadyExists(StoreName {
        value: "Main".into(),
    });

    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: already_exists_error.to_string(),
                    code: 6,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.1, 1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "role".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::Image(vec![1, 2, 3])),
                                },
                            )]),
                        }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
    assert!(!write_flag.load(Ordering::SeqCst));

    // Clean up - delete persistence file
    let _ = std::fs::remove_file(&*PERSISTENCE_FILE);
}

#[tokio::test]
async fn test_set_in_store() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![],
            })),
        },
        // Create the store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["role".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Valid set operation
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![DbStoreEntry {
                    key: Some(StoreKey {
                        key: vec![1.23, 1.0, 0.2],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                }],
            })),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![DbStoreEntry {
                    key: Some(StoreKey {
                        key: vec![2.1], // 1 dimension vs store's 3
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                }],
            })),
        },
        // Upsert operation
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.23, 1.0, 0.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "role".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("headmaster".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![0.03, 5.1, 3.23],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                ],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 3,
        input_dimension: 1,
    };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 1,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 1,
                        updated: 1,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 2,
                        size_in_bytes: 1304,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_remove_non_linear_indices() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Create store with KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["medal".into()],
                non_linear_indices: vec![NonLinearAlgorithm::KdTree.into()],
                error_if_exists: true,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![5.0, 5.1, 5.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    },
                ],
            })),
        },
        // Get similar items using KDTree
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree.into(),
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0, 3.0],
                }),
                condition: None,
            })),
        },
        // Remove KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::DropNonLinearAlgorithmIndex(
                db_query_types::DropNonLinearAlgorithmIndex {
                    store: "Main".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                    error_if_not_exists: false,
                },
            )),
        },
        // Should error as index doesn't exist (with error_if_not_exists=true)
        db_pipeline::DbQuery {
            query: Some(Query::DropNonLinearAlgorithmIndex(
                db_query_types::DropNonLinearAlgorithmIndex {
                    store: "Main".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                    error_if_not_exists: true,
                },
            )),
        },
        // Should error as KDTree no longer exists
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree as i32,
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0, 3.0],
                }),
                condition: None,
            })),
        },
        // Recreate KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::CreateNonLinearAlgorithmIndex(
                db_query_types::CreateNonLinearAlgorithmIndex {
                    store: "Main".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                },
            )),
        },
        // Should succeed as index exists again
        db_pipeline::DbQuery {
            query: Some(Query::DropNonLinearAlgorithmIndex(
                db_query_types::DropNonLinearAlgorithmIndex {
                    store: "Main".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                    error_if_not_exists: true,
                },
            )),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let non_linear_index_err = ServerError::NonLinearIndexNotFound(NonLinearAlgorithm::KdTree);

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 3,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![2.0, 2.1, 2.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("gold".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 1.4599998 }),
                        },
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![1.2, 1.3, 1.4],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("silver".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 3.0600002 }),
                        },
                    ],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: non_linear_index_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: non_linear_index_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::CreateIndex(
                db_response_types::CreateIndex { created_indexes: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_get_sim_n_non_linear() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Create store with KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["medal".into()],
                non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                error_if_exists: true,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![5.0, 5.1, 5.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    },
                ],
            })),
        },
        // Get 2 closest matches without condition
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree as i32,
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0, 3.0],
                }),
                condition: None,
            })),
        },
        // return just 1 entry regardless of closest_n
        // due to precondition satisfying just one
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree as i32,
                search_input: Some(StoreKey {
                    key: vec![5.0, 2.1, 2.2],
                }),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }),
                        })),
                    })),
                }),
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 3,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![2.0, 2.1, 2.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("gold".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 1.4599998 }),
                        },
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![1.2, 1.3, 1.4],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("silver".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 3.0600002 }),
                        },
                    ],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![db_response_types::GetSimNEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                        similarity: Some(Similarity { value: 9.0 }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_get_sim_n() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::CosineSimilarity as i32,
                search_input: Some(StoreKey { key: vec![] }),
                condition: None,
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["medal".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![5.0, 5.1, 5.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    },
                ],
            })),
        },
        // Error due to non-linear algorithm not existing
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree as i32,
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0, 3.0],
                }),
                condition: None,
            })),
        },
        // Error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::EuclideanDistance as i32,
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0],
                }),
                condition: None,
            })),
        },
        // Get with condition (should return 1 match)
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::CosineSimilarity as i32,
                search_input: Some(StoreKey {
                    key: vec![5.0, 2.1, 2.2],
                }),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }),
                        })),
                    })),
                }),
            })),
        },
        // Get closest 2 with DotProduct
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::DotProductSimilarity as i32,
                search_input: Some(StoreKey {
                    key: vec![1.0, 2.1, 2.2],
                }),
                condition: None,
            })),
        },
        // Get closest 2 with EuclideanDistance
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::EuclideanDistance as i32,
                search_input: Some(StoreKey {
                    key: vec![1.0, 2.1, 2.2],
                }),
                condition: None,
            })),
        },
        // Get closest 1 where medal is not gold
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 1,
                algorithm: Algorithm::CosineSimilarity as i32,
                search_input: Some(StoreKey {
                    key: vec![5.0, 2.1, 2.2],
                }),

                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }),
                        })),
                    })),
                }),
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 3,
        input_dimension: 2,
    };
    let non_linear_index_err = ServerError::NonLinearIndexNotFound(NonLinearAlgorithm::KdTree);

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 3,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: non_linear_index_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![db_response_types::GetSimNEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                        similarity: Some(Similarity {
                            value: 0.9036338825194858,
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![5.0, 5.1, 5.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("bronze".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 27.149998 }),
                        },
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![2.0, 2.1, 2.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("gold".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 11.25 }),
                        },
                    ],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![2.0, 2.1, 2.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("gold".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 1.0 }),
                        },
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![1.2, 1.3, 1.4],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("silver".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity {
                                value: 1.1489125293076061,
                            }),
                        },
                    ],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![db_response_types::GetSimNEntry {
                        key: Some(StoreKey {
                            key: vec![5.0, 5.1, 5.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                        similarity: Some(Similarity {
                            value: 0.9119372494019118,
                        }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_get_pred() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }),
                        })),
                    })),
                }),
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["medal".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.3, 1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    },
                ],
            })),
        },
        // Should return empty (no matches)
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::In(predicates::In {
                            key: "medal".into(),
                            values: vec![MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }],
                        })),
                    })),
                }),
            })),
        },
        // Get where medal != silver
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("silver".into())),
                            }),
                        })),
                    })),
                }),
            })),
        },
        // Get where medal != bronze
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("bronze".into())),
                            }),
                        })),
                    })),
                }),
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get { entries: vec![] },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.3, 1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_get_key() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let socket_addr = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", socket_addr);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![],
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 0.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "title".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("sorcerer".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 0.3],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "title".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("elf".into())),
                                },
                            )]),
                        }),
                    },
                ],
            })),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![
                    StoreKey {
                        key: vec![0.2, 0.3, 0.4],
                    },
                    StoreKey {
                        key: vec![0.2, 0.3, 0.4],
                    },
                    StoreKey {
                        key: vec![0.4, 0.6],
                    },
                ],
            })),
        },
        // Should return empty (keys don't exist)
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![
                    StoreKey {
                        key: vec![0.4, 0.6],
                    },
                    StoreKey {
                        key: vec![0.2, 0.5],
                    },
                ],
            })),
        },
        // Get existing keys
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![
                    StoreKey {
                        key: vec![1.2, 0.3],
                    },
                    StoreKey {
                        key: vec![0.4, 0.6],
                    },
                    StoreKey {
                        key: vec![1.0, 0.2],
                    },
                ],
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 2,
        input_dimension: 3,
    };

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get { entries: vec![] },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![
                        DbStoreEntry {
                            key: Some(StoreKey {
                                key: vec![1.2, 0.3],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "title".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("elf".into())),
                                    },
                                )]),
                            }),
                        },
                        DbStoreEntry {
                            key: Some(StoreKey {
                                key: vec![1.0, 0.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "title".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "sorcerer".into(),
                                        )),
                                    },
                                )]),
                            }),
                        },
                    ],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());

    let response = client
        .info_server(tonic::Request::new(ahnlich_types::db::query::InfoServer {}))
        .await
        .expect("Failed call info server");

    let info_response = response.into_inner().info.unwrap();

    assert_eq!(info_response.address, socket_addr.to_string());
    assert_eq!(info_response.version, env!("CARGO_PKG_VERSION").to_string());
    assert_eq!(info_response.r#type, ServerType::Database as i32);
}

#[tokio::test]
async fn test_create_pred_index() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::CreatePredIndex(db_query_types::CreatePredIndex {
                store: "Main".to_string(),
                predicates: vec!["planet".into()],
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec!["galaxy".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "andromeda".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "humanoid".into(),
                                        )),
                                    },
                                ),
                            ]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "milkyway".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("insects".into())),
                                    },
                                ),
                            ]),
                        }),
                    },
                ],
            })),
        },
        // Should return 0 (no new indexes created)
        db_pipeline::DbQuery {
            query: Some(Query::CreatePredIndex(db_query_types::CreatePredIndex {
                store: "Main".to_string(),
                predicates: vec!["galaxy".into()],
            })),
        },
        // Get with galaxy="milkyway"
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "galaxy".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("milkyway".into())),
                            }),
                        })),
                    })),
                }),
            })),
        },
        // Get with life-form="humanoid"
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "life-form".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("humanoid".into())),
                            }),
                        })),
                    })),
                }),
            })),
        },
        // Get with life-form IN ["insects"]
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::In(predicates::In {
                            key: "life-form".into(),
                            values: vec![MetadataValue {
                                value: Some(MetadataValueEnum::RawString("insects".into())),
                            }],
                        })),
                    })),
                }),
            })),
        },
        // Get with life-form NOT IN ["humanoid"]
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotIn(predicates::NotIn {
                            key: "life-form".into(),
                            values: vec![MetadataValue {
                                value: Some(MetadataValueEnum::RawString("humanoid".into())),
                            }],
                        })),
                    })),
                }),
            })),
        },
        // Create indexes for technology and life-form (should return 2)
        db_pipeline::DbQuery {
            query: Some(Query::CreatePredIndex(db_query_types::CreatePredIndex {
                store: "Main".to_string(),
                predicates: vec!["technology".into(), "life-form".into(), "galaxy".into()],
            })),
        },
        // Verify humanoid still works after indexing
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "life-form".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("humanoid".into())),
                            }),
                        })),
                    })),
                }),
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::CreateIndex(
                db_response_types::CreateIndex { created_indexes: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "milkyway".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("insects".into())),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "andromeda".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "humanoid".into(),
                                        )),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "milkyway".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("insects".into())),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "milkyway".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("insects".into())),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::CreateIndex(
                db_response_types::CreateIndex { created_indexes: 2 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "andromeda".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "humanoid".into(),
                                        )),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_drop_pred_index() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: true,
                predicates: vec!["planet".into()],
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["galaxy".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // Should not error (error_if_not_exists=false)
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: false,
                predicates: vec!["planet".into()],
            })),
        },
        // Should error (predicate doesn't exist)
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: true,
                predicates: vec!["planet".into()],
            })),
        },
        // Should succeed (galaxy predicate exists)
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: true,
                predicates: vec!["galaxy".into()],
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let predicate_not_found_err = ServerError::PredicateNotFound("planet".into());

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: predicate_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_drop_stores() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should not error (error_if_not_exists=false)
        db_pipeline::DbQuery {
            query: Some(Query::DropStore(db_query_types::DropStore {
                store: "Main".to_string(),
                error_if_not_exists: false,
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        // List stores
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
        // Should succeed (store exists)
        db_pipeline::DbQuery {
            query: Some(Query::DropStore(db_query_types::DropStore {
                store: "Main".to_string(),
                error_if_not_exists: true,
            })),
        },
        // Should error (store doesn't exist)
        db_pipeline::DbQuery {
            query: Some(Query::DropStore(db_query_types::DropStore {
                store: "Main".to_string(),
                error_if_not_exists: true,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 0,
                        size_in_bytes: 1056,
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}
