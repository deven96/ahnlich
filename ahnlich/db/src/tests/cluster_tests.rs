use crate::cli::ServerConfig;
use crate::server::handler::Server;
use ahnlich_types::db::{
    pipeline::{DbQuery, DbRequestPipeline, db_query::Query, db_server_response::Response},
    query as db_query_types,
};
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::metadata::metadata_value::Value as MetadataValueEnum;
use ahnlich_types::services::db_service::db_service_client::DbServiceClient;
use ahnlich_types::shared::cluster::{ClusterInfoQuery, NodeRole};
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::task::JoinHandle;
use tokio::time::{Duration, sleep};
use tonic::transport::Channel;
use utils::server::AhnlichServerUtils;

struct SpawnedServer {
    addr: std::net::SocketAddr,
    _data_dir: TempDir,
    task: JoinHandle<()>,
}

impl Drop for SpawnedServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

fn cluster_data_dir(tempdir: &TempDir) -> PathBuf {
    let path = tempdir.path().join("raft-data");
    std::fs::create_dir_all(&path).expect("create raft data dir");
    path
}

fn store_value(label: &str) -> StoreValue {
    let mut value = HashMap::new();
    value.insert(
        "label".to_owned(),
        MetadataValue {
            value: Some(MetadataValueEnum::RawString(label.to_owned())),
        },
    );
    StoreValue { value }
}

fn store_entry(label: &str) -> DbStoreEntry {
    DbStoreEntry {
        key: Some(StoreKey {
            key: vec![0.1, 0.2, 0.3],
        }),
        value: Some(store_value(label)),
    }
}

async fn connect_client(addr: std::net::SocketAddr) -> DbServiceClient<Channel> {
    DbServiceClient::connect(format!("http://{addr}"))
        .await
        .expect("connect db client")
}

async fn wait_for_ping(addr: std::net::SocketAddr) {
    for _ in 0..50 {
        if let Ok(mut client) = DbServiceClient::connect(format!("http://{addr}")).await
            && client
                .ping(tonic::Request::new(db_query_types::Ping {}))
                .await
                .is_ok()
        {
            return;
        }
        sleep(Duration::from_millis(100)).await;
    }
    panic!("db server at {addr} did not become ready");
}

async fn spawn_clustered_server(
    config: ServerConfig,
    data_dir: TempDir,
) -> (SpawnedServer, std::net::SocketAddr) {
    let server = Server::new(&config)
        .await
        .expect("create clustered db server");
    let addr = server.local_addr().expect("db local addr");
    let cluster_addr = server.cluster_local_addr().expect("cluster local addr");
    let task = tokio::spawn(async move {
        let _ = server.start().await;
    });
    wait_for_ping(addr).await;
    (
        SpawnedServer {
            addr,
            _data_dir: data_dir,
            task,
        },
        cluster_addr,
    )
}

async fn wait_for_cluster_size(client: &mut DbServiceClient<Channel>, expected_nodes: usize) {
    for _ in 0..50 {
        let response = client
            .cluster_info(tonic::Request::new(ClusterInfoQuery {}))
            .await
            .expect("cluster info query should succeed")
            .into_inner();
        if response.nodes.len() == expected_nodes {
            return;
        }
        sleep(Duration::from_millis(200)).await;
    }
    panic!("cluster did not reach {expected_nodes} visible nodes in time");
}

async fn wait_for_store_entry_count(
    client: &mut DbServiceClient<Channel>,
    store: &str,
    expected_entries: u64,
) {
    for _ in 0..50 {
        let response = client
            .get_store(tonic::Request::new(db_query_types::GetStore {
                store: store.to_owned(),
                schema: None,
            }))
            .await;

        if let Ok(response) = response
            && response.into_inner().len == expected_entries
        {
            return;
        }

        sleep(Duration::from_millis(200)).await;
    }
    panic!("replicated store did not reach {expected_entries} entries in time");
}

#[tokio::test]
async fn test_single_node_clustered_pipeline_and_cluster_info() {
    let data_dir = tempfile::tempdir().expect("tempdir");
    let config = ServerConfig::default()
        .os_select_port()
        .cluster_bootstrap(std::net::SocketAddr::from(([127, 0, 0, 1], 0)))
        .cluster_data_dir(cluster_data_dir(&data_dir));

    let (server, _) = spawn_clustered_server(config, data_dir).await;
    let mut client = connect_client(server.addr).await;

    let pipeline = DbRequestPipeline {
        queries: vec![
            DbQuery {
                query: Some(Query::CreateStore(db_query_types::CreateStore {
                    store: "clustered-main".to_owned(),
                    dimension: 3,
                    create_predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: true,
                    schema: None,
                })),
            },
            DbQuery {
                query: Some(Query::Set(db_query_types::Set {
                    store: "clustered-main".to_owned(),
                    inputs: vec![store_entry("first")],
                    schema: None,
                })),
            },
            DbQuery {
                query: Some(Query::ListStores(db_query_types::ListStores {
                    schema: None,
                })),
            },
            DbQuery {
                query: Some(Query::ClusterInfo(ClusterInfoQuery {})),
            },
        ],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline))
        .await
        .expect("clustered pipeline should succeed")
        .into_inner();

    assert_eq!(response.responses.len(), 4);
    assert!(matches!(
        response.responses[0].response,
        Some(Response::Unit(_))
    ));
    let set_response = match response.responses[1].response.clone() {
        Some(Response::Set(set)) => set,
        other => panic!("expected Set response, got {other:?}"),
    };
    assert_eq!(
        set_response.upsert,
        Some(ahnlich_types::shared::info::StoreUpsert {
            inserted: 1,
            updated: 0,
        })
    );

    let store_list = match response.responses[2].response.clone() {
        Some(Response::StoreList(store_list)) => store_list,
        other => panic!("expected StoreList response, got {other:?}"),
    };
    assert_eq!(store_list.stores.len(), 1);
    let info = &store_list.stores[0];
    assert_eq!(info.name, "clustered-main");
    assert_eq!(info.dimension, 3);
    assert_eq!(info.len, 1);
    assert!(info.size_in_bytes > 0);
    assert!(info.non_linear_indices.is_empty());
    assert!(info.predicate_indices.is_empty());

    let cluster_info = match response.responses[3].response.clone() {
        Some(Response::ClusterInfo(cluster_info)) => cluster_info,
        other => panic!("expected ClusterInfo response, got {other:?}"),
    };
    assert_eq!(cluster_info.nodes.len(), 1);
    assert_eq!(cluster_info.nodes[0].role, NodeRole::Leader as i32);
}

#[tokio::test]
async fn test_three_node_cluster_replication_and_follower_forwarding() {
    let leader_dir = tempfile::tempdir().expect("leader tempdir");
    let follower_one_dir = tempfile::tempdir().expect("follower one tempdir");
    let follower_two_dir = tempfile::tempdir().expect("follower two tempdir");

    let (leader, leader_cluster_addr) = spawn_clustered_server(
        ServerConfig::default()
            .os_select_port()
            .cluster_bootstrap(std::net::SocketAddr::from(([127, 0, 0, 1], 0)))
            .cluster_data_dir(cluster_data_dir(&leader_dir)),
        leader_dir,
    )
    .await;

    let (follower_one, _) = spawn_clustered_server(
        ServerConfig::default()
            .os_select_port()
            .cluster_join(
                std::net::SocketAddr::from(([127, 0, 0, 1], 0)),
                leader_cluster_addr,
            )
            .cluster_data_dir(cluster_data_dir(&follower_one_dir)),
        follower_one_dir,
    )
    .await;

    let (follower_two, _) = spawn_clustered_server(
        ServerConfig::default()
            .os_select_port()
            .cluster_join(
                std::net::SocketAddr::from(([127, 0, 0, 1], 0)),
                leader_cluster_addr,
            )
            .cluster_data_dir(cluster_data_dir(&follower_two_dir)),
        follower_two_dir,
    )
    .await;

    let mut leader_client = connect_client(leader.addr).await;
    let mut follower_one_client = connect_client(follower_one.addr).await;
    let mut follower_two_client = connect_client(follower_two.addr).await;

    wait_for_cluster_size(&mut leader_client, 3).await;

    leader_client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "replicated-store".to_owned(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: None,
        }))
        .await
        .expect("leader create_store should succeed");

    follower_one_client
        .set(tonic::Request::new(db_query_types::Set {
            store: "replicated-store".to_owned(),
            inputs: vec![store_entry("replicated")],
            schema: None,
        }))
        .await
        .expect("follower set should be forwarded to the leader");

    let pipeline_response = follower_two_client
        .pipeline(tonic::Request::new(DbRequestPipeline {
            queries: vec![
                DbQuery {
                    query: Some(Query::Set(db_query_types::Set {
                        store: "replicated-store".to_owned(),
                        inputs: vec![DbStoreEntry {
                            key: Some(StoreKey {
                                key: vec![0.4, 0.5, 0.6],
                            }),
                            value: Some(store_value("forwarded-pipeline")),
                        }],
                        schema: None,
                    })),
                },
                DbQuery {
                    query: Some(Query::ListStores(db_query_types::ListStores {
                        schema: None,
                    })),
                },
            ],
        }))
        .await
        .expect("follower pipeline should be forwarded to the leader")
        .into_inner();

    assert!(matches!(
        pipeline_response.responses[0].response,
        Some(Response::Set(_))
    ));
    let pipeline_store_list = match pipeline_response.responses[1].response.clone() {
        Some(Response::StoreList(store_list)) => store_list,
        other => panic!("expected forwarded StoreList response, got {other:?}"),
    };
    assert_eq!(pipeline_store_list.stores.len(), 1);

    wait_for_store_entry_count(&mut follower_one_client, "replicated-store", 2).await;
    wait_for_store_entry_count(&mut follower_two_client, "replicated-store", 2).await;

    let cluster_info = leader_client
        .cluster_info(tonic::Request::new(ClusterInfoQuery {}))
        .await
        .expect("leader cluster_info should succeed")
        .into_inner();
    assert_eq!(cluster_info.nodes.len(), 3);
    assert_eq!(
        cluster_info
            .nodes
            .iter()
            .filter(|node| node.role == NodeRole::Leader as i32)
            .count(),
        1
    );

    let leader_store_list = leader_client
        .list_stores(tonic::Request::new(db_query_types::ListStores {
            schema: None,
        }))
        .await
        .expect("leader ListStores should succeed")
        .into_inner();
    assert_eq!(leader_store_list.stores.len(), 1);

    let replicated_store = &leader_store_list.stores[0];
    assert_eq!(replicated_store.name, "replicated-store");
    assert_eq!(replicated_store.dimension, 3);
    assert_eq!(replicated_store.len, 2);
    assert!(replicated_store.size_in_bytes > 0);
    assert!(replicated_store.non_linear_indices.is_empty());
    assert!(replicated_store.predicate_indices.is_empty());

    let follower_one_store_list = follower_one_client
        .list_stores(tonic::Request::new(db_query_types::ListStores {
            schema: None,
        }))
        .await
        .expect("follower ListStores should be forwarded to the leader")
        .into_inner();
    assert_eq!(follower_one_store_list.stores.len(), 1);

    let follower_two_store_list = follower_two_client
        .list_stores(tonic::Request::new(db_query_types::ListStores {
            schema: None,
        }))
        .await
        .expect("second follower ListStores should be forwarded to the leader")
        .into_inner();
    assert_eq!(follower_two_store_list.stores.len(), 1);
}
