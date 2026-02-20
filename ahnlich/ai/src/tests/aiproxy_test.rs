use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;
use ahnlich_replication::config::RaftStorageEngine;
use ahnlich_replication::proto::cluster_admin::cluster_admin_service_client::ClusterAdminServiceClient;
use ahnlich_replication::proto::cluster_admin::{
    AddLearnerRequest, ChangeMembershipRequest, GetLeaderRequest, InitClusterRequest, NodeInfo,
};

use utils::server::AhnlichServerUtils;

use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::{
    collections::{HashMap, HashSet},
    net::{SocketAddr, TcpListener},
    sync::atomic::Ordering,
};

use crate::{
    cli::{AIProxyConfig, server::SupportedModels},
    engine::ai::models::ModelDetails,
    error::AIProxyError,
    server::handler::AIProxyServer,
};

use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::{
    ai::server::GetSimNEntry,
    metadata::{MetadataValue, metadata_value::Value as MValue},
};
use ahnlich_types::{
    ai::{
        models::AiModel,
        pipeline::{self as ai_pipeline, ai_query::Query},
        preprocess::PreprocessAction,
        query::{self as ai_query_types},
        server::{self as ai_response_types, AiStoreInfo, GetEntry},
    },
    keyval::{AiStoreEntry, StoreInput, StoreName, StoreValue, store_input::Value},
    services::ai_service::ai_service_client::AiServiceClient,
    shared::info::StoreUpsert,
};
use ahnlich_types::{
    predicates::{
        self, Predicate, PredicateCondition, predicate::Kind as PredicateKind,
        predicate_condition::Kind as PredicateConditionKind,
    },
    similarity::Similarity,
};

use bincode::deserialize;
use openraft::{BasicNode, RaftMetrics, ServerState};
use std::path::PathBuf;
use test_case::test_case;
use tokio::task::JoinHandle;
use tokio::time::Duration;
use tokio_util::sync::CancellationToken;
use tonic::transport::Channel;

static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());
static AI_CONFIG: Lazy<AIProxyConfig> = Lazy::new(|| AIProxyConfig::default().os_select_port());

static PERSISTENCE_FILE: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ahnlich_ai_proxy.dat"));

static AI_CONFIG_WITH_PERSISTENCE: Lazy<AIProxyConfig> = Lazy::new(|| {
    AIProxyConfig::default()
        .os_select_port()
        .set_persistence_interval(200)
        .set_persist_location((*PERSISTENCE_FILE).clone())
});

static AI_CONFIG_LIMITED_MODELS: Lazy<AIProxyConfig> = Lazy::new(|| {
    AIProxyConfig::default()
        .os_select_port()
        .set_supported_models(vec![SupportedModels::AllMiniLML6V2])
});

async fn provision_test_servers_without_db_client() -> SocketAddr {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");

    tokio::spawn(async move { server.start().await });

    let mut config = AI_CONFIG.clone();

    config.without_db = true;

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not initialize ai proxy");

    let ai_address = ai_server.local_addr().expect("Could not get local addr");

    // start up ai proxy
    let _ = tokio::spawn(async move { ai_server.start().await });
    // Allow some time for the servers to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    ai_address
}

async fn provision_test_servers() -> SocketAddr {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    // let db_address = server.local_addr().expect("Could not get local addr");
    let db_port = server.local_addr().unwrap().port();

    tokio::spawn(async move { server.start().await });

    let mut config = AI_CONFIG.clone();
    config.db_port = db_port;

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not initialize ai proxy");

    let ai_address = ai_server.local_addr().expect("Could not get local addr");

    // start up ai proxy
    let _ = tokio::spawn(async move { ai_server.start().await });
    // Allow some time for the servers to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    ai_address
}

fn reserve_ephemeral_addr() -> SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to reserve local port");
    listener
        .local_addr()
        .expect("failed to read reserved local addr")
}

async fn provision_clustered_test_servers() -> (SocketAddr, SocketAddr, u64, SocketAddr) {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let db_port = server.local_addr().unwrap().port();
    tokio::spawn(async move { server.start().await });

    let mut config = AI_CONFIG.clone();
    config.db_port = db_port;
    config.cluster_enabled = true;
    config.raft_storage = RaftStorageEngine::Memory;
    config.raft_node_id = 1;
    let raft_addr = reserve_ephemeral_addr();
    let admin_addr = reserve_ephemeral_addr();
    config.raft_addr = raft_addr.to_string();
    config.admin_addr = admin_addr.to_string();
    config.raft_join = None;

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not initialize clustered ai proxy");
    let ai_address = ai_server.local_addr().expect("Could not get local addr");

    let _ = tokio::spawn(async move { ai_server.start().await });
    tokio::time::sleep(Duration::from_millis(250)).await;

    (ai_address, admin_addr, 1, raft_addr)
}

async fn init_single_node_cluster(
    admin_addr: SocketAddr,
    node_id: u64,
    raft_addr: SocketAddr,
    service_addr: SocketAddr,
) {
    let mut client = ClusterAdminServiceClient::connect(format!("http://{admin_addr}"))
        .await
        .expect("failed to connect cluster admin");
    client
        .init_cluster(tonic::Request::new(InitClusterRequest {
            nodes: vec![NodeInfo {
                id: node_id,
                raft_addr: raft_addr.to_string(),
                admin_addr: admin_addr.to_string(),
                service_addr: service_addr.to_string(),
            }],
        }))
        .await
        .expect("failed to init single-node cluster");
}

async fn wait_for_leader(admin_addr: SocketAddr, expected_node_id: u64) {
    let mut client = ClusterAdminServiceClient::connect(format!("http://{admin_addr}"))
        .await
        .expect("failed to connect cluster admin");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let res = client
            .get_leader(tonic::Request::new(GetLeaderRequest {}))
            .await
            .expect("get_leader rpc failed")
            .into_inner();
        if res.leader_id == expected_node_id {
            return;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for raft leader; last leader_id={}",
            res.leader_id
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

struct ClusteredAiNode {
    node_id: u64,
    raft_addr: SocketAddr,
    admin_addr: SocketAddr,
    service_addr: SocketAddr,
    shutdown: CancellationToken,
    handle: JoinHandle<std::io::Result<()>>,
}

async fn spawn_clustered_ai_node(node_id: u64, db_port: u16) -> ClusteredAiNode {
    let mut config = AI_CONFIG.clone();
    config.db_port = db_port;
    config.cluster_enabled = true;
    config.raft_storage = RaftStorageEngine::Memory;
    config.supported_models = vec![SupportedModels::AllMiniLML6V2];
    config.raft_node_id = node_id;
    config.raft_join = None;

    let raft_addr = reserve_ephemeral_addr();
    let admin_addr = reserve_ephemeral_addr();
    config.raft_addr = raft_addr.to_string();
    config.admin_addr = admin_addr.to_string();

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not initialize clustered ai proxy");
    let service_addr = ai_server.local_addr().expect("Could not get local addr");
    let shutdown = ai_server.cancellation_token();

    let handle = tokio::spawn(async move { ai_server.start().await });
    wait_for_node_ready(service_addr, admin_addr, node_id).await;

    ClusteredAiNode {
        node_id,
        raft_addr,
        admin_addr,
        service_addr,
        shutdown,
        handle,
    }
}

async fn cluster_admin_client(
    admin_addr: SocketAddr,
) -> ClusterAdminServiceClient<tonic::transport::Channel> {
    ClusterAdminServiceClient::connect(format!("http://{admin_addr}"))
        .await
        .expect("failed to connect cluster admin")
}

async fn ai_client(service_addr: SocketAddr) -> AiServiceClient<tonic::transport::Channel> {
    AiServiceClient::connect(
        Channel::from_shared(format!("http://{service_addr}")).expect("Failed to get channel"),
    )
    .await
    .expect("Failure")
}

async fn wait_for_node_ready(_service_addr: SocketAddr, admin_addr: SocketAddr, node_id: u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        let mut metrics_ok = false;
        if let Ok(mut admin) =
            ClusterAdminServiceClient::connect(format!("http://{admin_addr}")).await
        {
            metrics_ok = admin
                .get_metrics(tonic::Request::new(
                    ahnlich_replication::proto::cluster_admin::GetMetricsRequest {},
                ))
                .await
                .is_ok();
        }

        if metrics_ok {
            return;
        }

        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for node readiness: node_id={node_id} service={_service_addr} admin={admin_addr}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn init_three_node_cluster(nodes: &[ClusteredAiNode]) {
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
    wait_for_leader(n1.admin_addr, n1.node_id).await;

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
    nodes: &[ClusteredAiNode],
    expected_voters: &HashSet<u64>,
    phase: &str,
) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    let mut last_observations: Vec<String> = Vec::new();
    loop {
        let mut this_round: Vec<String> = Vec::new();
        let mut matching = 0usize;

        for node in nodes {
            let metric_line =
                match ClusterAdminServiceClient::connect(format!("http://{}", node.admin_addr))
                    .await
                {
                    Ok(mut admin_client) => match admin_client
                        .get_metrics(tonic::Request::new(
                            ahnlich_replication::proto::cluster_admin::GetMetricsRequest {},
                        ))
                        .await
                    {
                        Ok(resp) => match deserialize::<RaftMetrics<u64, BasicNode>>(
                            &resp.into_inner().metrics,
                        ) {
                            Ok(m) => {
                                let voters: HashSet<u64> =
                                    m.membership_config.membership().voter_ids().collect();
                                if voters == *expected_voters {
                                    matching += 1;
                                }
                                format!(
                                    "node={} state={:?} leader={:?} term={} voters={:?}",
                                    node.node_id, m.state, m.current_leader, m.current_term, voters
                                )
                            }
                            Err(e) => format!("node={} metrics_decode=ERR({})", node.node_id, e),
                        },
                        Err(e) => format!("node={} metrics_rpc=ERR({})", node.node_id, e),
                    },
                    Err(e) => format!("node={} admin_connect=ERR({})", node.node_id, e),
                };
            this_round.push(metric_line);
        }

        if matching == nodes.len() {
            return;
        }

        if !this_round.is_empty() {
            last_observations = this_round;
        }

        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for exact voter set during phase={phase}\nexpected={:?}\nlast observations:\n{}",
            expected_voters,
            last_observations.join("\n")
        );
        tokio::time::sleep(Duration::from_millis(120)).await;
    }
}

async fn wait_for_leader_from_any(
    nodes: &[ClusteredAiNode],
    allowed: &HashSet<u64>,
    excluded: &HashSet<u64>,
    phase: &str,
) -> (u64, u64) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    let mut last_observations: Vec<String> = Vec::new();
    loop {
        let mut this_round: Vec<String> = Vec::new();
        for node in nodes {
            if !allowed.contains(&node.node_id) || excluded.contains(&node.node_id) {
                continue;
            }
            let Ok(mut client) = AiServiceClient::connect(
                Channel::from_shared(format!("http://{}", node.service_addr))
                    .expect("Failed to get channel"),
            )
            .await
            else {
                this_round.push(format!(
                    "node={} service={} ping_connect=ERR",
                    node.node_id, node.service_addr
                ));
                continue;
            };

            let ping_ok = client
                .ping(tonic::Request::new(ai_query_types::Ping {}))
                .await
                .is_ok();

            let mut leader_from_this_node: Option<(u64, u64)> = None;
            let metric_line = match ClusterAdminServiceClient::connect(format!(
                "http://{}",
                node.admin_addr
            ))
            .await
            {
                Ok(mut admin_client) => match admin_client
                    .get_metrics(tonic::Request::new(
                        ahnlich_replication::proto::cluster_admin::GetMetricsRequest {},
                    ))
                    .await
                {
                    Ok(resp) => {
                        match deserialize::<RaftMetrics<u64, BasicNode>>(&resp.into_inner().metrics)
                        {
                            Ok(m) => {
                                let voters: Vec<u64> =
                                    m.membership_config.membership().voter_ids().collect();
                                if m.state == ServerState::Leader
                                    && m.current_leader == Some(node.node_id)
                                    && allowed.contains(&node.node_id)
                                    && !excluded.contains(&node.node_id)
                                {
                                    leader_from_this_node = Some((node.node_id, m.current_term));
                                }
                                format!(
                                    "node={} state={:?} leader={:?} term={} ping_ok={} voters={:?} repl={:?}",
                                    node.node_id,
                                    m.state,
                                    m.current_leader,
                                    m.current_term,
                                    ping_ok,
                                    voters,
                                    m.replication
                                        .as_ref()
                                        .map(|r| r.keys().cloned().collect::<Vec<_>>())
                                )
                            }
                            Err(e) => format!(
                                "node={} metrics_decode=ERR({}) ping_ok={}",
                                node.node_id, e, ping_ok
                            ),
                        }
                    }
                    Err(e) => format!(
                        "node={} metrics_rpc=ERR({}) ping_ok={}",
                        node.node_id, e, ping_ok
                    ),
                },
                Err(e) => format!(
                    "node={} admin_connect=ERR({}) ping_ok={}",
                    node.node_id, e, ping_ok
                ),
            };
            this_round.push(metric_line);

            if let Some((leader_id, term)) = leader_from_this_node {
                if ping_ok {
                    return (leader_id, term);
                }
            }
        }

        if !this_round.is_empty() {
            last_observations = this_round;
        }

        assert!(
            tokio::time::Instant::now() < deadline,
            "timed out waiting for leader election during phase={phase}\nlast observations:\n{}",
            last_observations.join("\n")
        );
        tokio::time::sleep(Duration::from_millis(120)).await;
    }
}

async fn dump_cluster_metrics(nodes: &[ClusteredAiNode], label: &str) {
    eprintln!("=== cluster metrics dump: {label} ===");
    for node in nodes {
        let line = match ClusterAdminServiceClient::connect(format!("http://{}", node.admin_addr))
            .await
        {
            Ok(mut admin_client) => match admin_client
                .get_metrics(tonic::Request::new(
                    ahnlich_replication::proto::cluster_admin::GetMetricsRequest {},
                ))
                .await
            {
                Ok(resp) => {
                    match deserialize::<RaftMetrics<u64, BasicNode>>(&resp.into_inner().metrics) {
                        Ok(m) => {
                            let voters: Vec<u64> =
                                m.membership_config.membership().voter_ids().collect();
                            format!(
                                "node={} state={:?} leader={:?} term={} voters={:?} repl={:?}",
                                node.node_id,
                                m.state,
                                m.current_leader,
                                m.current_term,
                                voters,
                                m.replication
                                    .as_ref()
                                    .map(|r| r.keys().cloned().collect::<Vec<_>>())
                            )
                        }
                        Err(e) => format!("node={} metrics_decode=ERR({})", node.node_id, e),
                    }
                }
                Err(e) => format!("node={} metrics_rpc=ERR({})", node.node_id, e),
            },
            Err(e) => format!("node={} admin_connect=ERR({})", node.node_id, e),
        };
        eprintln!("{line}");
    }
    eprintln!("=== end cluster metrics dump ===");
}

async fn wait_for_store_presence(service_addr: SocketAddr, store_name: &str, should_exist: bool) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(10);
    loop {
        let mut client = ai_client(service_addr).await;
        let listed = client
            .list_stores(tonic::Request::new(ai_query_types::ListStores {}))
            .await
            .expect("list_stores should succeed")
            .into_inner();
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

async fn wait_for_leader_replication_targets(
    leader_admin_addr: SocketAddr,
    expected_followers: &HashSet<u64>,
    phase: &str,
) {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
    loop {
        let mut admin = cluster_admin_client(leader_admin_addr).await;
        let metrics = admin
            .get_metrics(tonic::Request::new(
                ahnlich_replication::proto::cluster_admin::GetMetricsRequest {},
            ))
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
            "timed out waiting for replication targets during phase={phase}; expected={expected_followers:?}"
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
    nodes: &[ClusteredAiNode],
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
                .get_metrics(tonic::Request::new(
                    ahnlich_replication::proto::cluster_admin::GetMetricsRequest {},
                ))
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

async fn restart_clustered_ai_node(
    node: &mut ClusteredAiNode,
    db_port: u16,
    service_port: Option<u16>,
) {
    node.shutdown.cancel();
    node.handle.abort();
    tokio::time::sleep(Duration::from_millis(300)).await;

    let mut config = AI_CONFIG.clone();
    config.db_port = db_port;
    config.cluster_enabled = true;
    config.raft_storage = RaftStorageEngine::Memory;
    config.supported_models = vec![SupportedModels::AllMiniLML6V2];
    config.raft_node_id = node.node_id;
    config.raft_join = None;
    config.raft_addr = node.raft_addr.to_string();
    config.admin_addr = node.admin_addr.to_string();
    if let Some(port) = service_port {
        config.port = port;
    } else {
        config.port = 0;
    }

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not restart clustered ai proxy");
    node.service_addr = ai_server
        .local_addr()
        .expect("Could not get restarted local addr");
    node.shutdown = ai_server.cancellation_token();
    node.handle = tokio::spawn(async move { ai_server.start().await });
    wait_for_node_ready(node.service_addr, node.admin_addr, node.node_id).await;
}

#[tokio::test]
async fn test_simple_ai_proxy_ping() {
    let address = provision_test_servers().await;

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");

    let mut client = AiServiceClient::connect(channel).await.expect("Failure");

    let response = client
        .ping(tonic::Request::new(ahnlich_types::ai::query::Ping {}))
        .await
        .expect("Failed to ping");

    let expected = ai_response_types::Pong {};
    println!("Response: {response:?}");
    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_ai_proxy_create_store_success() {
    let address = provision_test_servers().await;

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");

    let mut client = AiServiceClient::connect(channel).await.expect("Failure");

    let store_name = "Sample Store".to_string();
    let create_store = ahnlich_types::ai::query::CreateStore {
        store: store_name.clone(),
        query_model: AiModel::AllMiniLmL6V2.into(),
        index_model: AiModel::AllMiniLmL6V2.into(),
        predicates: vec![],
        non_linear_indices: vec![],
        error_if_exists: true,
        store_original: true,
    };
    let response = client
        .create_store(tonic::Request::new(create_store))
        .await
        .expect("Failed to Create Store");

    let expected = ai_response_types::Unit {};
    assert_eq!(expected, response.into_inner());

    // list stores to verify it's present.
    let message = ahnlich_types::ai::query::ListStores {};
    let response = client
        .list_stores(tonic::Request::new(message))
        .await
        .expect("Failed to Create Store");

    let ai_model: ModelDetails = SupportedModels::from(&AiModel::AllMiniLmL6V2).to_model_details();

    let expected = ai_response_types::StoreList {
        stores: vec![AiStoreInfo {
            name: store_name,
            query_model: AiModel::AllMiniLmL6V2.into(),
            index_model: AiModel::AllMiniLmL6V2.into(),
            embedding_size: ai_model.embedding_size.get() as u64,
        }],
    };
    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_ai_proxy_clustered_create_drop_and_purge_via_raft() {
    let (ai_addr, admin_addr, node_id, raft_addr) = provision_clustered_test_servers().await;
    init_single_node_cluster(admin_addr, node_id, raft_addr, ai_addr).await;
    wait_for_leader(admin_addr, node_id).await;

    let mut client = AiServiceClient::connect(
        Channel::from_shared(format!("http://{ai_addr}")).expect("Failed to get channel"),
    )
    .await
    .expect("Failure");

    let create_store = |name: &str| ai_query_types::CreateStore {
        store: name.to_string(),
        query_model: AiModel::AllMiniLmL6V2.into(),
        index_model: AiModel::AllMiniLmL6V2.into(),
        predicates: vec![],
        non_linear_indices: vec![],
        error_if_exists: true,
        store_original: true,
    };

    client
        .create_store(tonic::Request::new(create_store("cluster-store-a")))
        .await
        .expect("create_store should succeed");

    let listed = client
        .list_stores(tonic::Request::new(ai_query_types::ListStores {}))
        .await
        .expect("list_stores should succeed")
        .into_inner();
    assert!(
        listed.stores.iter().any(|s| s.name == "cluster-store-a"),
        "created store should be visible after raft write"
    );

    let dropped = client
        .drop_store(tonic::Request::new(ai_query_types::DropStore {
            store: "cluster-store-a".to_string(),
            error_if_not_exists: true,
        }))
        .await
        .expect("drop_store should succeed")
        .into_inner();
    assert_eq!(dropped.deleted_count, 1);

    let listed_after_drop = client
        .list_stores(tonic::Request::new(ai_query_types::ListStores {}))
        .await
        .expect("list_stores should succeed")
        .into_inner();
    assert!(
        !listed_after_drop
            .stores
            .iter()
            .any(|s| s.name == "cluster-store-a"),
        "dropped store should be absent"
    );

    client
        .create_store(tonic::Request::new(create_store("cluster-store-b")))
        .await
        .expect("create_store b should succeed");
    client
        .create_store(tonic::Request::new(create_store("cluster-store-c")))
        .await
        .expect("create_store c should succeed");

    let purged = client
        .purge_stores(tonic::Request::new(ai_query_types::PurgeStores {}))
        .await
        .expect("purge_stores should succeed")
        .into_inner();
    assert_eq!(purged.deleted_count, 2);

    let listed_after_purge = client
        .list_stores(tonic::Request::new(ai_query_types::ListStores {}))
        .await
        .expect("list_stores should succeed")
        .into_inner();
    assert!(
        listed_after_purge.stores.is_empty(),
        "all stores should be purged"
    );
}

#[tokio::test]
async fn test_ai_proxy_cluster_replication_and_single_failover() {
    let db_config = ServerConfig::default().os_select_port();
    let server = Server::new(&db_config)
        .await
        .expect("Failed to create server");
    let db_port = server.local_addr().unwrap().port();
    let db_handle = tokio::spawn(async move { server.start().await });

    let mut nodes = vec![
        spawn_clustered_ai_node(1, db_port).await,
        spawn_clustered_ai_node(2, db_port).await,
        spawn_clustered_ai_node(3, db_port).await,
    ];

    init_three_node_cluster(&nodes).await;
    // Give membership changes enough time to propagate before failover operations.
    tokio::time::sleep(Duration::from_secs(5)).await;
    dump_cluster_metrics(&nodes, "after-init-three-node-cluster").await;

    let allowed_ids: HashSet<u64> = nodes.iter().map(|n| n.node_id).collect();
    let expected_voters = allowed_ids.clone();
    let mut excluded_ids = HashSet::new();
    wait_for_exact_voter_set(&nodes, &expected_voters, "pre-failover-membership").await;

    let (leader_id, leader_term) =
        wait_for_leader_from_any(&nodes, &allowed_ids, &excluded_ids, "initial").await;
    let leader_idx = nodes
        .iter()
        .position(|n| n.node_id == leader_id)
        .expect("leader node not found");
    let leader_service = nodes[leader_idx].service_addr;

    let mut leader_client = ai_client(leader_service).await;
    leader_client
        .create_store(tonic::Request::new(ai_query_types::CreateStore {
            store: "replicated_store".to_string(),
            query_model: AiModel::AllMiniLmL6V2.into(),
            index_model: AiModel::AllMiniLmL6V2.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        }))
        .await
        .expect("leader create_store should succeed");
    let leader_admin_addr = nodes[leader_idx].admin_addr;
    let expected_followers: HashSet<u64> = nodes
        .iter()
        .filter(|n| n.node_id != leader_id)
        .map(|n| n.node_id)
        .collect();
    wait_for_leader_replication_targets(
        leader_admin_addr,
        &expected_followers,
        "post-create-store",
    )
    .await;

    // Fail current leader and check replicated state via new leader.
    excluded_ids.insert(leader_id);
    dump_cluster_metrics(&nodes, "before-first-leader-abort").await;
    nodes[leader_idx].shutdown.cancel();
    tokio::time::sleep(Duration::from_millis(300)).await;
    dump_cluster_metrics(&nodes, "after-first-leader-abort").await;
    let (new_leader_id, new_leader_term) =
        wait_for_leader_from_any(&nodes, &allowed_ids, &excluded_ids, "post-first-failover").await;
    assert_ne!(
        new_leader_id, leader_id,
        "expected a different leader after failover"
    );
    assert!(
        new_leader_term > leader_term,
        "expected term to increase after failover: old={leader_term}, new={new_leader_term}"
    );
    let new_leader_service = nodes
        .iter()
        .find(|n| n.node_id == new_leader_id)
        .expect("new leader node not found")
        .service_addr;
    wait_for_store_presence(new_leader_service, "replicated_store", true).await;

    // Verify state change after failover leader write.
    let mut new_leader_client = ai_client(new_leader_service).await;
    let dropped = new_leader_client
        .drop_store(tonic::Request::new(ai_query_types::DropStore {
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
    db_handle.abort();
}

#[tokio::test]
async fn test_ai_proxy_cluster_replication_and_double_failover() {
    let db_config = ServerConfig::default().os_select_port();
    let server = Server::new(&db_config)
        .await
        .expect("Failed to create server");
    let db_port = server.local_addr().unwrap().port();
    let db_handle = tokio::spawn(async move { server.start().await });

    let mut nodes = vec![
        spawn_clustered_ai_node(1, db_port).await,
        spawn_clustered_ai_node(2, db_port).await,
        spawn_clustered_ai_node(3, db_port).await,
    ];

    init_three_node_cluster(&nodes).await;
    tokio::time::sleep(Duration::from_secs(5)).await;
    dump_cluster_metrics(&nodes, "double-failover-after-init").await;

    let allowed_ids: HashSet<u64> = nodes.iter().map(|n| n.node_id).collect();
    let expected_voters = allowed_ids.clone();
    let mut excluded_ids = HashSet::new();
    wait_for_exact_voter_set(&nodes, &expected_voters, "double-failover-pre-membership").await;

    let (leader_id_1, term_1) = wait_for_leader_from_any(
        &nodes,
        &allowed_ids,
        &excluded_ids,
        "double-failover-initial",
    )
    .await;
    let leader_idx_1 = nodes
        .iter()
        .position(|n| n.node_id == leader_id_1)
        .expect("initial leader node not found");
    let leader_service_1 = nodes[leader_idx_1].service_addr;
    let mut leader_client_1 = ai_client(leader_service_1).await;

    leader_client_1
        .create_store(tonic::Request::new(ai_query_types::CreateStore {
            store: "replicated_store".to_string(),
            query_model: AiModel::AllMiniLmL6V2.into(),
            index_model: AiModel::AllMiniLmL6V2.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
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
        "double-failover-post-create",
    )
    .await;
    wait_for_store_presence(leader_service_1, "replicated_store", true).await;

    // First failover.
    excluded_ids.insert(leader_id_1);
    dump_cluster_metrics(&nodes, "double-failover-before-first-leader-down").await;
    nodes[leader_idx_1].shutdown.cancel();
    tokio::time::sleep(Duration::from_millis(400)).await;
    dump_cluster_metrics(&nodes, "double-failover-after-first-leader-down").await;

    let (leader_id_2, term_2) = wait_for_leader_from_any(
        &nodes,
        &allowed_ids,
        &excluded_ids,
        "double-failover-post-first-failover",
    )
    .await;
    assert_ne!(
        leader_id_2, leader_id_1,
        "leader must change after first failover"
    );
    assert!(
        term_2 > term_1,
        "term must increase after first failover: old={term_1} new={term_2}"
    );
    let leader_idx_2 = nodes
        .iter()
        .position(|n| n.node_id == leader_id_2)
        .expect("second leader node not found");
    let leader_service_2 = nodes[leader_idx_2].service_addr;
    wait_for_store_presence(leader_service_2, "replicated_store", true).await;

    // Write through second leader and verify committed state.
    let mut leader_client_2 = ai_client(leader_service_2).await;
    let dropped = leader_client_2
        .drop_store(tonic::Request::new(ai_query_types::DropStore {
            store: "replicated_store".to_string(),
            error_if_not_exists: true,
        }))
        .await
        .expect("drop_store on second leader should succeed")
        .into_inner();
    assert_eq!(dropped.deleted_count, 1);
    wait_for_store_presence(leader_service_2, "replicated_store", false).await;

    // Quorum-safe choreography before second failover.
    wait_for_consistent_survivor_view(
        &nodes,
        &excluded_ids,
        &expected_voters,
        leader_id_2,
        "double-failover-before-second-leader-down",
    )
    .await;

    // Second failover, now only one node remains.
    excluded_ids.insert(leader_id_2);
    dump_cluster_metrics(&nodes, "double-failover-before-second-leader-down").await;
    nodes[leader_idx_2].shutdown.cancel();
    tokio::time::sleep(Duration::from_millis(400)).await;
    dump_cluster_metrics(&nodes, "double-failover-after-second-leader-down").await;

    let lone_node = nodes
        .iter()
        .find(|n| !excluded_ids.contains(&n.node_id))
        .expect("expected one surviving node");
    let mut lone_client = ai_client(lone_node.service_addr).await;

    let read_err = lone_client
        .list_stores(tonic::Request::new(ai_query_types::ListStores {}))
        .await
        .expect_err("list_stores should fail without quorum");
    assert_unavailable(read_err, "list_stores without quorum");

    let write_err = lone_client
        .create_store(tonic::Request::new(ai_query_types::CreateStore {
            store: "should_fail_without_quorum".to_string(),
            query_model: AiModel::AllMiniLmL6V2.into(),
            index_model: AiModel::AllMiniLmL6V2.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        }))
        .await
        .expect_err("create_store should fail without quorum");
    assert_unavailable(write_err, "create_store without quorum");

    // Stronger safety step: restore quorum by restarting one stopped node.
    let restart_target_id = leader_id_1;
    let restart_idx = nodes
        .iter()
        .position(|n| n.node_id == restart_target_id)
        .expect("restart target node not found");
    restart_clustered_ai_node(&mut nodes[restart_idx], db_port, None).await;
    excluded_ids.remove(&restart_target_id);
    dump_cluster_metrics(&nodes, "double-failover-after-restart-one-node").await;

    let (leader_id_3, _term_3) = wait_for_leader_from_any(
        &nodes,
        &allowed_ids,
        &excluded_ids,
        "double-failover-post-quorum-restore",
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
    db_handle.abort();
}

#[tokio::test]
async fn test_ai_store_get_key_works() {
    let address = provision_test_servers().await;

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");

    let mut client = AiServiceClient::connect(channel).await.expect("Failure");

    let store_name = StoreName {
        value: String::from("Deven Kicks"),
    };

    let store_entry_input = StoreInput {
        value: Some(Value::RawString(String::from("Jordan 3"))),
    };

    let inputs = vec![AiStoreEntry {
        key: Some(store_entry_input.clone()),
        value: Some(StoreValue {
            value: HashMap::new(),
        }),
    }];

    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.value.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.value.clone(),
                inputs,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };

    let _ = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let message = ahnlich_types::ai::query::GetKey {
        store: store_name.value.clone(),
        keys: vec![store_entry_input.clone()],
    };

    let response = client
        .get_key(tonic::Request::new(message))
        .await
        .expect("Failed to get key");

    let expected = ai_response_types::Get {
        entries: vec![GetEntry {
            key: Some(store_entry_input),
            value: Some(StoreValue {
                value: HashMap::new(),
            }),
        }],
    };

    assert!(response.into_inner().entries.len() == expected.entries.len())
}

// TODO!
#[tokio::test]
async fn test_list_clients_works() {
    //     let address = provision_test_servers().await;
    // let _first_stream = TcpStream::connect(address).await.unwrap();
    // let second_stream = TcpStream::connect(address).await.unwrap();
    // let message = AIServerQuery::from_queries(&[AIQuery::ListClients]);
    // let mut reader = BufReader::new(second_stream);
    // let response = get_server_response(&mut reader, message).await;
    // let inner = response.into_inner();

    // // only two clients are connected
    // match inner.as_slice() {
    //     [Ok(AIServerResponse::ClientList(connected_clients))] => {
    //         assert!(connected_clients.len() == 2)
    //     }
    //     a => {
    //         assert!(false, "Unexpected result for client list {:?}", a);
    //     }
    // };
}

#[tokio::test]
async fn test_ai_store_no_original() {
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let store_name = "Deven Kicks".to_string();
    let matching_metadatakey = "Brand".to_string();
    let matching_metadatavalue = MetadataValue {
        value: Some(MValue::RawString("Nike".into())),
    };

    let nike_store_value = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]),
    };

    let adidas_store_value = StoreValue {
        value: HashMap::from_iter([(
            matching_metadatakey.clone(),
            MetadataValue {
                value: Some(MValue::RawString("Adidas".into())),
            },
        )]),
    };

    let store_data = vec![
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Jordan 3".into())),
            }),
            value: Some(nike_store_value.clone()),
        },
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Air Force 1".into())),
            }),
            value: Some(nike_store_value.clone()),
        },
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Yeezy".into())),
            }),
            value: Some(adidas_store_value),
        },
    ];

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![matching_metadatakey.clone(), "Original".to_string()],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: false,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: store_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let _ = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    // println!("res .... {:?}", res.into_inner());

    // Test GetPred
    let condition = PredicateCondition {
        kind: Some(PredicateConditionKind::Value(Predicate {
            kind: Some(PredicateKind::Equals(predicates::Equals {
                key: matching_metadatakey,
                value: Some(matching_metadatavalue),
            })),
        })),
    };

    let get_pred_message = ahnlich_types::ai::query::GetPred {
        store: store_name,
        condition: Some(condition),
    };

    let response = client
        .get_pred(tonic::Request::new(get_pred_message))
        .await
        .expect("Failed to get pred")
        .into_inner();

    println!("res .... {:?}", response);

    let expected = ahnlich_types::ai::server::Get {
        entries: vec![
            GetEntry {
                key: None,
                value: Some(nike_store_value.clone()),
            },
            GetEntry {
                key: None,
                value: Some(nike_store_value.clone()),
            },
        ],
    };

    assert_eq!(response.entries.len(), expected.entries.len());
}

#[tokio::test]
async fn test_ai_proxy_get_pred_succeeds() {
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let store_name = "Deven Kicks".to_string();
    let matching_metadatakey = "Brand".to_string();
    let matching_metadatavalue = MetadataValue {
        value: Some(MValue::RawString("Nike".into())),
    };

    let nike_store_value = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]),
    };

    let adidas_store_value = StoreValue {
        value: HashMap::from_iter([(
            matching_metadatakey.clone(),
            MetadataValue {
                value: Some(MValue::RawString("Adidas".into())),
            },
        )]),
    };

    let store_data = vec![
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Jordan 3".into())),
            }),
            value: Some(nike_store_value.clone()),
        },
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Air Force 1".into())),
            }),
            value: Some(nike_store_value.clone()),
        },
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Yeezy".into())),
            }),
            value: Some(adidas_store_value.clone()),
        },
    ];

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![matching_metadatakey.clone(), "Original".to_string()],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: store_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let _ = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    // Test GetPred
    let condition = PredicateCondition {
        kind: Some(PredicateConditionKind::Value(Predicate {
            kind: Some(PredicateKind::Equals(predicates::Equals {
                key: matching_metadatakey.clone(),
                value: Some(matching_metadatavalue.clone()),
            })),
        })),
    };

    let get_pred_message = ahnlich_types::ai::query::GetPred {
        store: store_name.clone(),
        condition: Some(condition),
    };

    let response = client
        .get_pred(tonic::Request::new(get_pred_message))
        .await
        .expect("Failed to get pred");

    let expected_entries = vec![
        GetEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Jordan 3".into())),
            }),
            value: Some(nike_store_value.clone()),
        },
        GetEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Air Force 1".into())),
            }),
            value: Some(nike_store_value.clone()),
        },
    ];

    let response_entries = response.into_inner().entries;
    assert_eq!(response_entries.len(), expected_entries.len());

    // Verify all expected entries are present (order-independent)
    for expected_entry in expected_entries {
        assert!(
            response_entries.contains(&expected_entry),
            "Missing entry: {:?}",
            expected_entry
        );
    }
}

#[tokio::test]
async fn test_ai_proxy_get_sim_n_succeeds() {
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let store_name = "Deven Kicks".to_string();
    let matching_metadatakey = "Brand".to_string();
    let matching_metadatavalue = MetadataValue {
        value: Some(MValue::RawString("Nike".into())),
    };

    let nike_store_value = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]),
    };

    let adidas_store_value = StoreValue {
        value: HashMap::from_iter([(
            matching_metadatakey.clone(),
            MetadataValue {
                value: Some(MValue::RawString("Adidas".into())),
            },
        )]),
    };

    let store_data = vec![
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Jordan 3".into())),
            }),
            value: Some(nike_store_value.clone()),
        },
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Air Force 1".into())),
            }),
            value: Some(nike_store_value.clone()),
        },
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Yeezy".into())),
            }),
            value: Some(adidas_store_value.clone()),
        },
    ];

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![matching_metadatakey.clone(), "Original".to_string()],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: store_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let _ = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    // Test GetSimN
    let get_sim_n_message = ahnlich_types::ai::query::GetSimN {
        store: store_name.clone(),
        search_input: Some(StoreInput {
            value: Some(Value::RawString("Yeezy".into())),
        }),
        condition: None,
        closest_n: 1,
        algorithm: Algorithm::DotProductSimilarity.into(),
        preprocess_action: PreprocessAction::ModelPreprocessing.into(),
        execution_provider: None,
    };

    let response = client
        .get_sim_n(tonic::Request::new(get_sim_n_message))
        .await
        .expect("Failed to get similar items");

    let expected_entry = GetSimNEntry {
        key: Some(StoreInput {
            value: Some(Value::RawString("Yeezy".into())),
        }),
        value: Some(adidas_store_value.clone()),
        similarity: Some(Similarity { value: 0.99999994 }),
    };

    let response_entries = response.into_inner().entries;
    assert_eq!(response_entries.len(), 1);
    assert_eq!(response_entries[0].key, expected_entry.key);
    assert_eq!(
        response_entries[0]
            .similarity
            .map(|sim| format!("{:.4}", sim.value)),
        expected_entry
            .similarity
            .map(|sim| format!("{:.4}", sim.value))
    );
}

#[test_case(1, AiModel::AllMiniLmL6V2.into(); "AllMiniLmL6V2")]
#[test_case(2, AiModel::AllMiniLmL12V2.into(); "AllMiniLmL12V2")]
#[test_case(3, AiModel::BgeBaseEnV15.into(); "BgeBaseEnV15")]
#[test_case(4, AiModel::BgeLargeEnV15.into(); "BgeLargeEnV15e")]
#[test_case(5, AiModel::Resnet50.into(); "Resnet50")]
#[test_case(6, AiModel::ClipVitB32Image.into(); "ClipVitB32Image")]
#[test_case(7, AiModel::ClipVitB32Text.into(); "ClipVitB32Text")]
#[tokio::test]
async fn test_convert_store_input_to_embeddings(index: usize, model: i32) {
    let address = provision_test_servers().await;

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");

    let mut client = AiServiceClient::connect(channel).await.expect("Failure");

    let ai_model = AiModel::try_from(model)
        .map_err(|_| AIProxyError::InputNotSpecified("AI Model Value".to_string()));

    let index_model_repr: ModelDetails =
        SupportedModels::from(&ai_model.unwrap()).to_model_details();

    let store_name = "Deven Kicks".to_string() + index.to_string().as_str();

    let matching_metadatakey = if index_model_repr.input_type().as_str_name() == "RAW_STRING" {
        "Brand".to_string() + index.to_string().as_str()
    } else if index_model_repr.input_type().as_str_name() == "IMAGE" {
        "Animal".to_string() + index.to_string().as_str()
    } else {
        "".to_string()
    };

    let matching_metadatavalue = if index_model_repr.input_type().as_str_name() == "RAW_STRING" {
        MetadataValue {
            value: Some(MValue::RawString("Nike".into())),
        }
    } else if index_model_repr.input_type().as_str_name() == "IMAGE" {
        MetadataValue {
            value: Some(MValue::RawString("Mammal".into())),
        }
    } else {
        MetadataValue {
            value: Some(MValue::RawString("".into())),
        }
    };

    let store_value = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]),
    };

    let store_input_1 = if index_model_repr.input_type().as_str_name() == "RAW_STRING" {
        StoreInput {
            value: Some(Value::RawString("Jordan 3".into())),
        }
    } else if index_model_repr.input_type().as_str_name() == "IMAGE" {
        StoreInput {
            value: Some(Value::Image(include_bytes!("./images/cat.png").to_vec())),
        }
    } else {
        StoreInput {
            value: Some(Value::RawString("".into())),
        }
    };

    let store_input_2 = if index_model_repr.input_type().as_str_name() == "RAW_STRING" {
        StoreInput {
            value: Some(Value::RawString("Air Force 1".into())),
        }
    } else if index_model_repr.input_type().as_str_name() == "IMAGE" {
        StoreInput {
            value: Some(Value::Image(include_bytes!("./images/dog.jpg").to_vec())),
        }
    } else {
        StoreInput {
            value: Some(Value::RawString("".into())),
        }
    };

    let store_data = vec![
        AiStoreEntry {
            key: Some(store_input_1.clone()),
            value: Some(store_value.clone()),
        },
        AiStoreEntry {
            key: Some(store_input_2.clone()),
            value: Some(store_value.clone()),
        },
    ];

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: model,
                index_model: model,
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: store_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };

    let _ = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let store_inputs = vec![store_input_1, store_input_2];

    let query = ai_query_types::ConvertStoreInputToEmbeddings {
        store_inputs: store_inputs.clone(),
        preprocess_action: Some(PreprocessAction::NoPreprocessing.into()),
        model,
    };

    let response = client
        .convert_store_input_to_embeddings(tonic::Request::new(query))
        .await
        .expect("Failed to convert store input to embeddings");

    let response_entries = response.into_inner().values;

    assert_eq!(response_entries.len(), store_inputs.len());

    // Verify all expected entries are present (order-independent)
    for input in store_inputs {
        assert!(
            response_entries
                .iter()
                .any(|e| { e.input == Some(input.clone()) && e.variant.is_some() })
        )
    }
}

#[test_case(1, AiModel::AllMiniLmL6V2.into(); "AllMiniLmL6V2")]
#[test_case(2, AiModel::AllMiniLmL12V2.into(); "AllMiniLmL12V2")]
#[test_case(3, AiModel::BgeBaseEnV15.into(); "BgeBaseEnV15")]
#[test_case(4, AiModel::BgeLargeEnV15.into(); "BgeLargeEnV15e")]
#[test_case(5, AiModel::Resnet50.into(); "Resnet50")]
#[test_case(6, AiModel::ClipVitB32Image.into(); "ClipVitB32Image")]
#[test_case(7, AiModel::ClipVitB32Text.into(); "ClipVitB32Text")]
#[tokio::test]
async fn test_convert_store_input_to_embeddings_without_db(index: usize, model: i32) {
    let address = provision_test_servers_without_db_client().await;

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");

    let mut client = AiServiceClient::connect(channel).await.expect("Failure");

    let ai_model = AiModel::try_from(model)
        .map_err(|_| AIProxyError::InputNotSpecified("AI Model Value".to_string()));

    let index_model_repr: ModelDetails =
        SupportedModels::from(&ai_model.unwrap()).to_model_details();

    let store_name = "Deven Kicks".to_string() + index.to_string().as_str();

    let matching_metadatakey = if index_model_repr.input_type().as_str_name() == "RAW_STRING" {
        "Brand".to_string() + index.to_string().as_str()
    } else if index_model_repr.input_type().as_str_name() == "IMAGE" {
        "Animal".to_string() + index.to_string().as_str()
    } else {
        "".to_string()
    };

    let matching_metadatavalue = if index_model_repr.input_type().as_str_name() == "RAW_STRING" {
        MetadataValue {
            value: Some(MValue::RawString("Nike".into())),
        }
    } else if index_model_repr.input_type().as_str_name() == "IMAGE" {
        MetadataValue {
            value: Some(MValue::RawString("Mammal".into())),
        }
    } else {
        MetadataValue {
            value: Some(MValue::RawString("".into())),
        }
    };

    let store_value = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]),
    };

    let store_input_1 = if index_model_repr.input_type().as_str_name() == "RAW_STRING" {
        StoreInput {
            value: Some(Value::RawString("Jordan 3".into())),
        }
    } else if index_model_repr.input_type().as_str_name() == "IMAGE" {
        StoreInput {
            value: Some(Value::Image(include_bytes!("./images/cat.png").to_vec())),
        }
    } else {
        StoreInput {
            value: Some(Value::RawString("".into())),
        }
    };

    let store_input_2 = if index_model_repr.input_type().as_str_name() == "RAW_STRING" {
        StoreInput {
            value: Some(Value::RawString("Air Force 1".into())),
        }
    } else if index_model_repr.input_type().as_str_name() == "IMAGE" {
        StoreInput {
            value: Some(Value::Image(include_bytes!("./images/dog.jpg").to_vec())),
        }
    } else {
        StoreInput {
            value: Some(Value::RawString("".into())),
        }
    };

    let store_data = vec![
        AiStoreEntry {
            key: Some(store_input_1.clone()),
            value: Some(store_value.clone()),
        },
        AiStoreEntry {
            key: Some(store_input_2.clone()),
            value: Some(store_value.clone()),
        },
    ];

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: model,
                index_model: model,
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: store_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };

    let _ = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let store_inputs = vec![store_input_1, store_input_2];

    let query = ai_query_types::ConvertStoreInputToEmbeddings {
        store_inputs: store_inputs.clone(),
        preprocess_action: Some(PreprocessAction::NoPreprocessing.into()),
        model,
    };

    let response = client
        .convert_store_input_to_embeddings(tonic::Request::new(query))
        .await
        .expect("Failed to convert store input to embeddings");

    let response_entries = response.into_inner().values;

    assert_eq!(response_entries.len(), store_inputs.len());

    // Verify all expected entries are present (order-independent)
    for input in store_inputs {
        assert!(
            response_entries
                .iter()
                .any(|e| { e.input == Some(input.clone()) && e.variant.is_some() })
        )
    }
}

#[tokio::test]
async fn test_ai_proxy_create_drop_pred_index() {
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let store_name = "Deven Kicks".to_string();
    let matching_metadatakey = "Brand".to_string();
    let matching_metadatavalue = MetadataValue {
        value: Some(MValue::RawString("Nike".into())),
    };

    let nike_store_value = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]),
    };

    let store_data = vec![AiStoreEntry {
        key: Some(StoreInput {
            value: Some(Value::RawString("Jordan 3".into())),
        }),
        value: Some(nike_store_value.clone()),
    }];

    let condition = PredicateCondition {
        kind: Some(PredicateConditionKind::Value(Predicate {
            kind: Some(PredicateKind::Equals(predicates::Equals {
                key: matching_metadatakey.clone(),
                value: Some(matching_metadatavalue.clone()),
            })),
        })),
    };

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::GetPred(ai_query_types::GetPred {
                store: store_name.clone(),
                condition: Some(condition.clone()),
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::CreatePredIndex(ai_query_types::CreatePredIndex {
                store: store_name.clone(),
                predicates: vec![matching_metadatakey.clone()],
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: store_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::GetPred(ai_query_types::GetPred {
                store: store_name.clone(),
                condition: Some(condition),
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::DropPredIndex(ai_query_types::DropPredIndex {
                store: store_name.clone(),
                predicates: vec![matching_metadatakey.clone()],
                error_if_not_exists: true,
            })),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let expected = ai_pipeline::AiResponsePipeline {
        responses: vec![
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Unit(
                    ai_response_types::Unit {},
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Get(
                    ai_response_types::Get { entries: vec![] },
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::CreateIndex(
                    ai_response_types::CreateIndex { created_indexes: 1 },
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Set(
                    ai_response_types::Set {
                        upsert: Some(StoreUpsert {
                            inserted: 1,
                            updated: 0,
                        }),
                    },
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Get(
                    ai_response_types::Get {
                        entries: vec![ai_response_types::GetEntry {
                            key: Some(StoreInput {
                                value: Some(Value::RawString("Jordan 3".into())),
                            }),
                            value: Some(nike_store_value.clone()),
                        }],
                    },
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Del(
                    ai_response_types::Del { deleted_count: 1 },
                )),
            },
        ],
    };

    assert_eq!(response.into_inner(), expected);
}

#[tokio::test]
async fn test_ai_proxy_del_key_drop_store() {
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let store_name = "Deven Kicks".to_string();
    let matching_metadatakey = "Brand".to_string();
    let matching_metadatavalue = MetadataValue {
        value: Some(MValue::RawString("Nike".into())),
    };

    let nike_store_value = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]),
    };

    let store_data = vec![AiStoreEntry {
        key: Some(StoreInput {
            value: Some(Value::RawString("Jordan 3".into())),
        }),
        value: Some(nike_store_value.clone()),
    }];

    let condition = PredicateCondition {
        kind: Some(PredicateConditionKind::Value(Predicate {
            kind: Some(PredicateKind::Equals(predicates::Equals {
                key: matching_metadatakey.clone(),
                value: Some(matching_metadatavalue.clone()),
            })),
        })),
    };

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: false,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: store_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::DelKey(ai_query_types::DelKey {
                store: store_name.clone(),
                keys: vec![StoreInput {
                    value: Some(Value::RawString("Jordan 3".into())),
                }],
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::GetPred(ai_query_types::GetPred {
                store: store_name.clone(),
                condition: Some(condition),
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::DropStore(ai_query_types::DropStore {
                store: store_name.clone(),
                error_if_not_exists: true,
            })),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let expected = ai_pipeline::AiResponsePipeline {
        responses: vec![
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Unit(
                    ai_response_types::Unit {},
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Unit(
                    ai_response_types::Unit {},
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Set(
                    ai_response_types::Set {
                        upsert: Some(StoreUpsert {
                            inserted: 1,
                            updated: 0,
                        }),
                    },
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Del(
                    ai_response_types::Del { deleted_count: 1 },
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Get(
                    ai_response_types::Get { entries: vec![] },
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Del(
                    ai_response_types::Del { deleted_count: 1 },
                )),
            },
        ],
    };

    assert_eq!(response.into_inner(), expected);
}

#[tokio::test]
async fn test_ai_proxy_test_with_persistence() {
    // Setup servers with persistence
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");

    let mut ai_proxy_config = AI_CONFIG_WITH_PERSISTENCE.clone();
    let db_port = server.local_addr().unwrap().port();
    ai_proxy_config.db_port = db_port;

    let ai_server = AIProxyServer::new(ai_proxy_config.clone())
        .await
        .expect("Could not initialize ai proxy");

    let address = ai_server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    tokio::time::sleep(Duration::from_millis(200)).await;

    let write_flag = ai_server.write_flag();

    // Start up ai proxy
    let _ = tokio::spawn(async move { ai_server.start().await });
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Create gRPC client
    let channel = Channel::from_shared(format!("http://{}", address))
        .expect("Failed to create channel")
        .connect()
        .await
        .expect("Failed to connect");
    let mut client = AiServiceClient::new(channel);

    let store_name = "Main".to_string();
    let store_name_2 = "Main2".to_string();

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name_2.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::DropStore(ai_query_types::DropStore {
                store: store_name.clone(),
                error_if_not_exists: true,
            })),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    // Verify pipeline responses
    let expected = ai_pipeline::AiResponsePipeline {
        responses: vec![
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Unit(
                    ai_response_types::Unit {},
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Unit(
                    ai_response_types::Unit {},
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Del(
                    ai_response_types::Del { deleted_count: 1 },
                )),
            },
        ],
    };

    assert_eq!(response.into_inner(), expected);
    assert!(write_flag.load(Ordering::SeqCst));

    // Allow time for persistence
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Start new server with persisted data
    let persisted_server = AIProxyServer::new(ai_proxy_config).await.unwrap();

    tokio::time::sleep(Duration::from_millis(100)).await;

    let persisted_address = persisted_server
        .local_addr()
        .expect("Could not get local addr");
    let persisted_write_flag = persisted_server.write_flag();
    let _ = tokio::spawn(async move { persisted_server.start().await });

    // Create new client for persisted server
    let channel = Channel::from_shared(format!("http://{}", persisted_address))
        .expect("Failed to create channel")
        .connect()
        .await
        .expect("Failed to connect");
    let mut persisted_client = AiServiceClient::new(channel);

    // Verify persisted data
    let list_response = persisted_client
        .list_stores(tonic::Request::new(ai_query_types::ListStores {}))
        .await
        .expect("Failed to list stores");

    let ai_model: ModelDetails = SupportedModels::from(&AiModel::AllMiniLmL6V2).to_model_details();
    let expected = ai_response_types::StoreList {
        stores: vec![ai_response_types::AiStoreInfo {
            name: store_name_2.clone(),
            query_model: AiModel::AllMiniLmL6V2.into(),
            index_model: AiModel::AllMiniLmL6V2.into(),
            embedding_size: ai_model.embedding_size.get() as u64,
        }],
    };

    assert_eq!(list_response.into_inner(), expected);
    assert!(!persisted_write_flag.load(Ordering::SeqCst));

    // Clean up persistence file
    let _ = std::fs::remove_file(&*PERSISTENCE_FILE);
}

#[tokio::test]
async fn test_ai_proxy_destroy_database() {
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let store_name = "Deven Kicks".to_string();
    let ai_model: ModelDetails = SupportedModels::from(&AiModel::AllMiniLmL6V2).to_model_details();

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::ListStores(ai_query_types::ListStores {})),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::PurgeStores(ai_query_types::PurgeStores {})),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::ListStores(ai_query_types::ListStores {})),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let expected = ai_pipeline::AiResponsePipeline {
        responses: vec![
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Unit(
                    ai_response_types::Unit {},
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::StoreList(
                    ai_response_types::StoreList {
                        stores: vec![ai_response_types::AiStoreInfo {
                            name: store_name,
                            query_model: AiModel::AllMiniLmL6V2.into(),
                            index_model: AiModel::AllMiniLmL6V2.into(),
                            embedding_size: ai_model.embedding_size.get() as u64,
                        }],
                    },
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Del(
                    ai_response_types::Del { deleted_count: 1 },
                )),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::StoreList(
                    ai_response_types::StoreList { stores: vec![] },
                )),
            },
        ],
    };

    assert_eq!(response.into_inner(), expected);
}

#[tokio::test]
async fn test_ai_proxy_binary_store_actions() {
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let store_name = "Deven Image Store".to_string();
    let matching_metadatakey = "Name".to_string();
    let matching_metadatavalue = MetadataValue {
        value: Some(MValue::RawString("Greatness".into())),
    };

    let store_value_1 = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]),
    };

    let store_value_2 = StoreValue {
        value: HashMap::from_iter([(
            matching_metadatakey.clone(),
            MetadataValue {
                value: Some(MValue::RawString("Deven".into())),
            },
        )]),
    };

    let store_data = vec![
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::Image(include_bytes!("./images/dog.jpg").to_vec())),
            }),
            value: Some(store_value_1.clone()),
        },
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::Image(include_bytes!("./images/test.webp").to_vec())),
            }),
            value: Some(store_value_2.clone()),
        },
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::Image(include_bytes!("./images/cat.png").to_vec())),
            }),
            value: Some(StoreValue {
                value: HashMap::from_iter([(
                    matching_metadatakey.clone(),
                    MetadataValue {
                        value: Some(MValue::RawString("Daniel".into())),
                    },
                )]),
            }),
        },
    ];

    let oversize_data = vec![AiStoreEntry {
        key: Some(StoreInput {
            value: Some(Value::Image(include_bytes!("./images/large.webp").to_vec())),
        }),
        value: Some(StoreValue {
            value: HashMap::from_iter([(
                matching_metadatakey.clone(),
                MetadataValue {
                    value: Some(MValue::RawString("Oversized".into())),
                },
            )]),
        }),
    }];

    let condition = PredicateCondition {
        kind: Some(PredicateConditionKind::Value(Predicate {
            kind: Some(PredicateKind::Equals(predicates::Equals {
                key: matching_metadatakey.clone(),
                value: Some(matching_metadatavalue.clone()),
            })),
        })),
    };

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::Resnet50.into(),
                index_model: AiModel::Resnet50.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::ListStores(ai_query_types::ListStores {})),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::CreatePredIndex(ai_query_types::CreatePredIndex {
                store: store_name.clone(),
                predicates: vec!["Name".to_string(), "Age".to_string()],
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: store_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: oversize_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::DropPredIndex(ai_query_types::DropPredIndex {
                store: store_name.clone(),
                predicates: vec!["Age".to_string()],
                error_if_not_exists: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::GetPred(ai_query_types::GetPred {
                store: store_name.clone(),
                condition: Some(condition),
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::PurgeStores(ai_query_types::PurgeStores {})),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let resnet_model: ModelDetails = SupportedModels::from(&AiModel::Resnet50).to_model_details();

    let expected = ai_pipeline::AiResponsePipeline {
        responses: vec![
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Unit(ai_response_types::Unit {})),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::StoreList(ai_response_types::StoreList {
                    stores: vec![AiStoreInfo {
                        name: store_name.clone(),
                        query_model: AiModel::Resnet50.into(),
                        index_model: AiModel::Resnet50.into(),
                        embedding_size: resnet_model.embedding_size.get() as u64,
                    }],
                })),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::CreateIndex(ai_response_types::CreateIndex { created_indexes: 2 })),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Set(ai_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 3,
                        updated: 0,
                    }),
                })),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Error(ahnlich_types::shared::info::ErrorResponse {
                    message: "Image Dimensions [(547, 821)] does not match the expected model dimensions [(224, 224)]".to_string(),
                    code: 3,
                })),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Del(ai_response_types::Del { deleted_count: 1 })),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Get(ai_response_types::Get {
                    entries: vec![GetEntry {
                        key: Some(StoreInput {
                            value: Some(Value::Image(include_bytes!("./images/dog.jpg").to_vec())),
                        }),
                        value: Some(store_value_1.clone()),
                    }],
                })),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Del(ai_response_types::Del { deleted_count: 1 })),
            },
        ],
    };

    assert_eq!(response.into_inner(), expected);
}

#[tokio::test]
async fn test_ai_proxy_binary_store_set_text_and_binary_fails() {
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let store_name = "Deven Mixed Store210u01".to_string();
    let matching_metadatakey = "Brand".to_string();
    let matching_metadatavalue = MetadataValue {
        value: Some(MValue::RawString("Nike".into())),
    };

    let store_value_1 = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]),
    };

    let store_data = vec![
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::Image(vec![93, 4, 1, 6, 2, 8, 8, 32, 45])),
            }),
            value: Some(store_value_1.clone()),
        },
        AiStoreEntry {
            key: Some(StoreInput {
                value: Some(Value::RawString("Buster Matthews is the name".into())),
            }),
            value: Some(StoreValue {
                value: HashMap::from_iter([(
                    "Description".to_string(),
                    MetadataValue {
                        value: Some(MValue::RawString("20 year old line backer".into())),
                    },
                )]),
            }),
        },
    ];

    // Create pipeline request
    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
                query_model: AiModel::AllMiniLmL6V2.into(),
                index_model: AiModel::AllMiniLmL6V2.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: true,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store_name.clone(),
                inputs: store_data,
                preprocess_action: PreprocessAction::NoPreprocessing.into(),
                execution_provider: None,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::PurgeStores(ai_query_types::PurgeStores {})),
        },
    ];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let expected = ai_pipeline::AiResponsePipeline {
        responses: vec![
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Unit(ai_response_types::Unit {})),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Error(ahnlich_types::shared::info::ErrorResponse  {
                    message: "Cannot index Input. Store expects [RawString], input type [Image] was provided".to_string(),
                    code: 3,
                })),
            },
            ai_pipeline::AiServerResponse {
                response: Some(ai_pipeline::ai_server_response::Response::Del(ai_response_types::Del { deleted_count: 1 })),
            },
        ],
    };

    assert_eq!(response.into_inner(), expected);
}

#[tokio::test]
async fn test_ai_proxy_create_store_errors_unsupported_models() {
    // Setup server with limited models
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let db_port = server.local_addr().unwrap().port();
    let mut config = AI_CONFIG_LIMITED_MODELS.clone();
    config.db_port = db_port;

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not initialize ai proxy");

    let address = ai_server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    let _ = tokio::spawn(async move { ai_server.start().await });
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Create gRPC client
    let channel = Channel::from_shared(format!("http://{}", address))
        .expect("Failed to create channel")
        .connect()
        .await
        .expect("Failed to connect");
    let mut client = AiServiceClient::new(channel);

    let store_name = "Error Handling Store".to_string();

    // Create pipeline request
    let queries = vec![ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::AllMiniLmL12V2.into(),
            index_model: AiModel::AllMiniLmL6V2.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        })),
    }];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let expected = ai_pipeline::AiResponsePipeline {
        responses: vec![ai_pipeline::AiServerResponse {
            response: Some(ai_pipeline::ai_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: AIProxyError::AIModelNotInitialized.to_string(),
                    code: 13,
                },
            )),
        }],
    };

    assert_eq!(response.into_inner(), expected);
}

#[tokio::test]
async fn test_ai_proxy_embedding_size_mismatch_error() {
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let store_name = "Deven Mixed Store210u01".to_string();

    let lml12_model: ModelDetails =
        SupportedModels::from(&AiModel::AllMiniLmL12V2).to_model_details();
    let bge_model: ModelDetails = SupportedModels::from(&AiModel::BgeBaseEnV15).to_model_details();

    // Create pipeline request
    let queries = vec![ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::AllMiniLmL6V2.into(),
            index_model: AiModel::BgeBaseEnV15.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        })),
    }];

    let pipelined_request = ai_pipeline::AiRequestPipeline { queries };
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let error_message = AIProxyError::DimensionsMismatchError {
        index_model_dim: bge_model.embedding_size.into(),
        query_model_dim: lml12_model.embedding_size.into(),
    };

    let expected = ai_pipeline::AiResponsePipeline {
        responses: vec![ai_pipeline::AiServerResponse {
            response: Some(ai_pipeline::ai_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: error_message.to_string(),
                    code: 3,
                },
            )),
        }],
    };

    assert_eq!(response.into_inner(), expected);
}
