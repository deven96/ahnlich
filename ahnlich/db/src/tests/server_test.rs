use crate::cli::ServerConfig;
use crate::server::handler::Server;
use ahnlich_types::bincode::BinCodeSerAndDeser;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::metadata::MetadataKey;
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::predicate::Predicate;
use ahnlich_types::predicate::PredicateCondition;
use ahnlich_types::query::Query;
use ahnlich_types::query::ServerQuery;
use ahnlich_types::server::ConnectedClient;
use ahnlich_types::server::ServerInfo;
use ahnlich_types::server::ServerResponse;
use ahnlich_types::server::ServerResult;
use ahnlich_types::server::StoreInfo;
use ahnlich_types::server::StoreUpsert;
use ahnlich_types::similarity::Algorithm;
use ahnlich_types::similarity::Similarity;
use futures::future::join_all;
use ndarray::array;
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

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

#[tokio::test]
async fn test_maximum_client_restriction_works() {
    let server = Server::new(&CONFIG_WITH_MAX_CLIENTS)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    //
    // Connect to the server multiple times
    let first_stream = TcpStream::connect(address).await.unwrap();
    let first_stream_addr = first_stream.local_addr().unwrap();
    let other_stream = TcpStream::connect(address).await.unwrap();
    let mut third_stream_fail = TcpStream::connect(address).await.unwrap();
    // failed stream should return EOF '0' as it was closed by server
    assert_eq!(
        third_stream_fail
            .read(&mut [])
            .await
            .expect("Could not read failed stream"),
        0
    );
    let message = ServerQuery::from_queries(&[Query::ListClients]);
    let expected_response = HashSet::from_iter([
        ConnectedClient {
            address: format!("{first_stream_addr}"),
            time_connected: SystemTime::now(),
        },
        ConnectedClient {
            address: format!("{}", other_stream.local_addr().unwrap()),
            time_connected: SystemTime::now(),
        },
    ]);
    let mut expected = ServerResult::with_capacity(1);
    expected.push(Ok(ServerResponse::ClientList(expected_response.clone())));
    let mut reader = BufReader::new(first_stream);
    query_server_assert_result(&mut reader, message, expected.clone()).await;
}

#[tokio::test]
async fn test_server_client_info() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    //
    // Connect to the server multiple times
    let first_stream = TcpStream::connect(address).await.unwrap();
    let other_stream = TcpStream::connect(address).await.unwrap();
    let first_stream_addr = first_stream.local_addr().unwrap();
    let expected_response = HashSet::from_iter([
        ConnectedClient {
            address: format!("{first_stream_addr}"),
            time_connected: SystemTime::now(),
        },
        ConnectedClient {
            address: format!("{}", other_stream.local_addr().unwrap()),
            time_connected: SystemTime::now(),
        },
    ]);
    let message = ServerQuery::from_queries(&[Query::ListClients]);
    let mut expected = ServerResult::with_capacity(1);
    expected.push(Ok(ServerResponse::ClientList(expected_response.clone())));
    let mut reader = BufReader::new(first_stream);
    query_server_assert_result(&mut reader, message, expected.clone()).await;
    // drop other stream and see if it reflects
    drop(other_stream);
    let expected_response = HashSet::from_iter([ConnectedClient {
        address: format!("{first_stream_addr}"),
        time_connected: SystemTime::now(),
    }]);
    let message = ServerQuery::from_queries(&[Query::ListClients]);
    let mut expected = ServerResult::with_capacity(1);
    expected.push(Ok(ServerResponse::ClientList(expected_response.clone())));
    query_server_assert_result(&mut reader, message, expected.clone()).await;
}

#[tokio::test]
async fn test_simple_stores_list() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[Query::ListStores]);
    let mut expected = ServerResult::with_capacity(1);
    let expected_response = HashSet::from_iter([]);
    expected.push(Ok(ServerResponse::StoreList(expected_response)));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_create_stores() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(3).unwrap(),
            create_predicates: HashSet::new(),
            error_if_exists: true,
        },
        // difference in dimensions don't matter as name is the same so this should error
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(2).unwrap(),
            create_predicates: HashSet::new(),
            error_if_exists: true,
        },
        // Should not error despite existing
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(2).unwrap(),
            create_predicates: HashSet::new(),
            error_if_exists: false,
        },
        Query::ListStores,
    ]);
    let mut expected = ServerResult::with_capacity(4);
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Err("Store Main already exists".to_string()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 0,
            size_in_bytes: 1712,
        },
    ]))));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_del_pred() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store does not exist
        Query::DelPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::NotEquals {
                key: MetadataKey::new("planet".into()),
                value: MetadataValue::new("earth".into()),
            }),
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(2).unwrap(),
            create_predicates: HashSet::from_iter([MetadataKey::new("planet".into())]),
            error_if_exists: true,
        },
        // should not error as it is correct query
        // but should delete nothing as nothing matches predicate
        Query::DelPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("planet".into()),
                value: MetadataValue::new("earth".into()),
            }),
        },
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![
                (
                    StoreKey(array![1.4, 1.5]),
                    HashMap::from_iter([(
                        MetadataKey::new("planet".into()),
                        MetadataValue::new("jupiter".into()),
                    )]),
                ),
                (
                    StoreKey(array![1.6, 1.7]),
                    HashMap::from_iter([(
                        MetadataKey::new("planet".into()),
                        MetadataValue::new("mars".into()),
                    )]),
                ),
            ],
        },
        Query::ListStores,
        // should delete the jupiter planet key
        Query::DelPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::NotEquals {
                key: MetadataKey::new("planet".into()),
                value: MetadataValue::new("mars".into()),
            }),
        },
        Query::GetKey {
            store: StoreName("Main".to_string()),
            keys: vec![StoreKey(array![1.4, 1.5]), StoreKey(array![1.6, 1.7])],
        },
        // should delete the mars planet key
        Query::DelPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("planet".into()),
                value: MetadataValue::new("mars".into()),
            }),
        },
        Query::ListStores,
    ]);
    let mut expected = ServerResult::with_capacity(9);
    expected.push(Err("Store Main not found".to_string()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Del(0)));
    expected.push(Ok(ServerResponse::Set(StoreUpsert {
        inserted: 2,
        updated: 0,
    })));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 2,
            size_in_bytes: 2104,
        },
    ]))));
    expected.push(Ok(ServerResponse::Del(1)));
    expected.push(Ok(ServerResponse::Get(vec![(
        StoreKey(array![1.6, 1.7]),
        HashMap::from_iter([(
            MetadataKey::new("planet".into()),
            MetadataValue::new("mars".into()),
        )]),
    )])));
    expected.push(Ok(ServerResponse::Del(1)));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 0,
            size_in_bytes: 1816,
        },
    ]))));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_del_key() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store does not exist
        Query::DelKey {
            store: StoreName("Main".to_string()),
            keys: vec![],
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(4).unwrap(),
            create_predicates: HashSet::from_iter([MetadataKey::new("role".into())]),
            error_if_exists: true,
        },
        // should not error as it is correct dimensions
        // but should delete nothing as nothing exists in the store yet
        Query::DelKey {
            store: StoreName("Main".to_string()),
            keys: vec![StoreKey(array![1.0, 1.1, 1.2, 1.3])],
        },
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![
                (StoreKey(array![1.0, 1.1, 1.2, 1.3]), HashMap::new()),
                (StoreKey(array![1.1, 1.2, 1.3, 1.4]), HashMap::new()),
            ],
        },
        Query::ListStores,
        // should error as different dimensions
        Query::DelKey {
            store: StoreName("Main".to_string()),
            keys: vec![StoreKey(array![1.0, 1.1, 1.2])],
        },
        // should work as key exists
        Query::DelKey {
            store: StoreName("Main".to_string()),
            keys: vec![StoreKey(array![1.0, 1.1, 1.2, 1.3])],
        },
        Query::ListStores,
    ]);
    let mut expected = ServerResult::with_capacity(8);
    expected.push(Err("Store Main not found".to_string()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Del(0)));
    expected.push(Ok(ServerResponse::Set(StoreUpsert {
        inserted: 2,
        updated: 0,
    })));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 2,
            size_in_bytes: 1880,
        },
    ]))));
    expected.push(Err(
        "Store dimension is [4], input dimension of [3] was specified".to_string(),
    ));
    expected.push(Ok(ServerResponse::Del(1)));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 1,
            size_in_bytes: 1808,
        },
    ]))));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await;
}

#[tokio::test]
async fn test_server_with_persistence() {
    let server = Server::new(&CONFIG_WITH_PERSISTENCE)
        .await
        .expect("Could not initialize server");
    let write_flag = server.write_flag();
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store does not exist
        Query::DelKey {
            store: StoreName("Main".to_string()),
            keys: vec![],
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(4).unwrap(),
            create_predicates: HashSet::from_iter([MetadataKey::new("role".into())]),
            error_if_exists: true,
        },
        // should not error as it is correct dimensions
        // but should delete nothing as nothing exists in the store yet
        Query::DelKey {
            store: StoreName("Main".to_string()),
            keys: vec![StoreKey(array![1.0, 1.1, 1.2, 1.3])],
        },
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![
                (StoreKey(array![1.0, 1.1, 1.2, 1.3]), HashMap::new()),
                (StoreKey(array![1.1, 1.2, 1.3, 1.4]), HashMap::new()),
            ],
        },
        Query::ListStores,
        // should error as different dimensions
        Query::DelKey {
            store: StoreName("Main".to_string()),
            keys: vec![StoreKey(array![1.0, 1.1, 1.2])],
        },
        // should work as key exists
        Query::DelKey {
            store: StoreName("Main".to_string()),
            keys: vec![StoreKey(array![1.0, 1.1, 1.2, 1.3])],
        },
        Query::ListStores,
    ]);
    let mut expected = ServerResult::with_capacity(8);
    expected.push(Err("Store Main not found".to_string()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Del(0)));
    expected.push(Ok(ServerResponse::Set(StoreUpsert {
        inserted: 2,
        updated: 0,
    })));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 2,
            size_in_bytes: 1880,
        },
    ]))));
    expected.push(Err(
        "Store dimension is [4], input dimension of [3] was specified".to_string(),
    ));
    expected.push(Ok(ServerResponse::Del(1)));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 1,
            size_in_bytes: 1808,
        },
    ]))));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await;
    // write flag should show that a write has occured
    assert!(write_flag.load(Ordering::SeqCst));
    // Allow some time for persistence to kick in
    tokio::time::sleep(Duration::from_millis(200)).await;
    // start another server with existing persistence
    let server = Server::new(&CONFIG_WITH_PERSISTENCE)
        .await
        .expect("Could not initialize server");
    let write_flag = server.write_flag();
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store was loaded from previous persistence and main still exists
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(2).unwrap(),
            create_predicates: HashSet::from_iter([MetadataKey::new("role".into())]),
            error_if_exists: true,
        },
        // should not error as store exists
        Query::DelKey {
            store: StoreName("Main".to_string()),
            keys: vec![],
        },
        Query::GetKey {
            store: StoreName("Main".to_string()),
            keys: vec![StoreKey(array![1.1, 1.2, 1.3, 1.4])],
        },
    ]);

    let mut expected = ServerResult::with_capacity(3);
    expected.push(Err("Store Main already exists".to_string()));
    expected.push(Ok(ServerResponse::Del(0)));
    expected.push(Ok(ServerResponse::Get(vec![(
        StoreKey(array![1.1, 1.2, 1.3, 1.4]),
        HashMap::new(),
    )])));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await;
    assert!(!write_flag.load(Ordering::SeqCst));
    // delete persistence
    let _ = std::fs::remove_file(&*PERSISTENCE_FILE);
}

#[tokio::test]
async fn test_set_in_store() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store does not exist
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![],
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(3).unwrap(),
            create_predicates: HashSet::from_iter([MetadataKey::new("role".into())]),
            error_if_exists: true,
        },
        // should not error as it is correct dimensions
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![(StoreKey(array![1.23, 1.0, 0.2]), HashMap::new())],
        },
        // should error as it is incorrect dimensions
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![(StoreKey(array![2.1]), HashMap::new())],
        },
        // should upsert existing value and add new value
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![
                (
                    StoreKey(array![1.23, 1.0, 0.2]),
                    HashMap::from_iter([(
                        MetadataKey::new("role".into()),
                        MetadataValue::new("headmaster".into()),
                    )]),
                ),
                (StoreKey(array![0.03, 5.1, 3.23]), HashMap::new()),
            ],
        },
        Query::ListStores,
    ]);
    let mut expected = ServerResult::with_capacity(6);
    expected.push(Err("Store Main not found".to_string()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Set(StoreUpsert {
        inserted: 1,
        updated: 0,
    })));
    expected.push(Err(
        "Store dimension is [3], input dimension of [1] was specified".to_string(),
    ));
    expected.push(Ok(ServerResponse::Set(StoreUpsert {
        inserted: 1,
        updated: 1,
    })));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 2,
            size_in_bytes: 2008,
        },
    ]))));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_get_sim_n() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store does not yet exist
        Query::GetSimN {
            store: StoreName("Main".to_string()),
            search_input: StoreKey(array![]),
            closest_n: NonZeroUsize::new(2).unwrap(),
            algorithm: Algorithm::CosineSimilarity,
            condition: None,
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(3).unwrap(),
            create_predicates: HashSet::from_iter([MetadataKey::new("medal".into())]),
            error_if_exists: true,
        },
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![
                (
                    StoreKey(array![1.2, 1.3, 1.4]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::new("silver".into()),
                    )]),
                ),
                (
                    StoreKey(array![2.0, 2.1, 2.2]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::new("gold".into()),
                    )]),
                ),
                (
                    StoreKey(array![5.0, 5.1, 5.2]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::new("bronze".into()),
                    )]),
                ),
            ],
        },
        // error due to dimension mismatch
        Query::GetSimN {
            store: StoreName("Main".to_string()),
            closest_n: NonZeroUsize::new(2).unwrap(),
            algorithm: Algorithm::EuclideanDistance,
            search_input: StoreKey(array![1.1, 2.0]),
            condition: None,
        },
        // return just 1 entry regardless of closest_n
        // due to precondition satisfying just one
        Query::GetSimN {
            store: StoreName("Main".to_string()),
            closest_n: NonZeroUsize::new(2).unwrap(),
            algorithm: Algorithm::CosineSimilarity,
            search_input: StoreKey(array![5.0, 2.1, 2.2]),
            condition: Some(PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("medal".into()),
                value: MetadataValue::new("gold".into()),
            })),
        },
        // Get closest 2 without precondition using DotProduct
        Query::GetSimN {
            store: StoreName("Main".to_string()),
            closest_n: NonZeroUsize::new(2).unwrap(),
            algorithm: Algorithm::DotProductSimilarity,
            search_input: StoreKey(array![1.0, 2.1, 2.2]),
            condition: None,
        },
        // Get closest 2 without precondition using EuclideanDistance
        Query::GetSimN {
            store: StoreName("Main".to_string()),
            closest_n: NonZeroUsize::new(2).unwrap(),
            algorithm: Algorithm::EuclideanDistance,
            search_input: StoreKey(array![1.0, 2.1, 2.2]),
            condition: None,
        },
        // get closest one where medal is not gold
        Query::GetSimN {
            store: StoreName("Main".to_string()),
            closest_n: NonZeroUsize::new(1).unwrap(),
            algorithm: Algorithm::CosineSimilarity,
            search_input: StoreKey(array![5.0, 2.1, 2.2]),
            condition: Some(PredicateCondition::Value(Predicate::NotEquals {
                key: MetadataKey::new("medal".into()),
                value: MetadataValue::new("gold".into()),
            })),
        },
    ]);
    let mut expected = ServerResult::with_capacity(8);
    expected.push(Err("Store Main not found".to_string()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Set(StoreUpsert {
        inserted: 3,
        updated: 0,
    })));
    expected.push(Err(
        "Store dimension is [3], input dimension of [2] was specified".into(),
    ));
    expected.push(Ok(ServerResponse::GetSimN(vec![(
        StoreKey(array![2.0, 2.1, 2.2]),
        HashMap::from_iter([(
            MetadataKey::new("medal".into()),
            MetadataValue::new("gold".into()),
        )]),
        Similarity(0.9036338825194858),
    )])));
    expected.push(Ok(ServerResponse::GetSimN(vec![
        (
            StoreKey(array![5.0, 5.1, 5.2]),
            HashMap::from_iter([(
                MetadataKey::new("medal".into()),
                MetadataValue::new("bronze".into()),
            )]),
            Similarity(27.149998),
        ),
        (
            StoreKey(array![2.0, 2.1, 2.2]),
            HashMap::from_iter([(
                MetadataKey::new("medal".into()),
                MetadataValue::new("gold".into()),
            )]),
            Similarity(11.25),
        ),
    ])));
    expected.push(Ok(ServerResponse::GetSimN(vec![
        (
            StoreKey(array![2.0, 2.1, 2.2]),
            HashMap::from_iter([(
                MetadataKey::new("medal".into()),
                MetadataValue::new("gold".into()),
            )]),
            Similarity(1.0),
        ),
        (
            StoreKey(array![1.2, 1.3, 1.4]),
            HashMap::from_iter([(
                MetadataKey::new("medal".into()),
                MetadataValue::new("silver".into()),
            )]),
            Similarity(1.1489125293076061),
        ),
    ])));
    expected.push(Ok(ServerResponse::GetSimN(vec![(
        StoreKey(array![5.0, 5.1, 5.2]),
        HashMap::from_iter([(
            MetadataKey::new("medal".into()),
            MetadataValue::new("bronze".into()),
        )]),
        Similarity(0.9119372494019118),
    )])));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_get_pred() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store does not yet exist
        Query::GetPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("medal".into()),
                value: MetadataValue::new("gold".into()),
            }),
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(3).unwrap(),
            create_predicates: HashSet::from_iter([MetadataKey::new("medal".into())]),
            error_if_exists: true,
        },
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![
                (
                    StoreKey(array![1.2, 1.3, 1.4]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::new("silver".into()),
                    )]),
                ),
                (
                    StoreKey(array![1.3, 1.4, 1.5]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::new("bronze".into()),
                    )]),
                ),
            ],
        },
        // should not error but return 0
        Query::GetPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::In {
                key: MetadataKey::new("medal".into()),
                value: HashSet::from_iter([MetadataValue::new("gold".into())]),
            }),
        },
        Query::GetPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::NotEquals {
                key: MetadataKey::new("medal".into()),
                value: MetadataValue::new("silver".into()),
            }),
        },
        Query::GetPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::NotEquals {
                key: MetadataKey::new("medal".into()),
                value: MetadataValue::new("bronze".into()),
            }),
        },
    ]);
    let mut expected = ServerResult::with_capacity(8);
    expected.push(Err("Store Main not found".to_string()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Set(StoreUpsert {
        inserted: 2,
        updated: 0,
    })));
    expected.push(Ok(ServerResponse::Get(vec![])));
    expected.push(Ok(ServerResponse::Get(vec![(
        StoreKey(array![1.3, 1.4, 1.5]),
        HashMap::from_iter([(
            MetadataKey::new("medal".into()),
            MetadataValue::new("bronze".into()),
        )]),
    )])));
    expected.push(Ok(ServerResponse::Get(vec![(
        StoreKey(array![1.2, 1.3, 1.4]),
        HashMap::from_iter([(
            MetadataKey::new("medal".into()),
            MetadataValue::new("silver".into()),
        )]),
    )])));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_get_key() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store does not yet exist
        Query::GetKey {
            store: StoreName("Main".to_string()),
            keys: vec![],
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(2).unwrap(),
            create_predicates: HashSet::new(),
            error_if_exists: true,
        },
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![
                (
                    StoreKey(array![1.0, 0.2]),
                    HashMap::from_iter([(
                        MetadataKey::new("title".into()),
                        MetadataValue::new("sorcerer".into()),
                    )]),
                ),
                (
                    StoreKey(array![1.2, 0.3]),
                    HashMap::from_iter([(
                        MetadataKey::new("title".into()),
                        MetadataValue::new("elf".into()),
                    )]),
                ),
            ],
        },
        // should error as dimension mismatch
        Query::GetKey {
            store: StoreName("Main".to_string()),
            keys: vec![
                StoreKey(array![0.2, 0.3, 0.4]),
                StoreKey(array![0.2, 0.3, 0.4, 0.6]),
                StoreKey(array![0.4, 0.6]),
            ],
        },
        // should not error however the keys do not exist so should be empty
        Query::GetKey {
            store: StoreName("Main".to_string()),
            keys: vec![StoreKey(array![0.4, 0.6]), StoreKey(array![0.2, 0.5])],
        },
        // Gets back the existing key in order
        Query::GetKey {
            store: StoreName("Main".to_string()),
            keys: vec![
                StoreKey(array![1.2, 0.3]),
                StoreKey(array![0.4, 0.6]),
                StoreKey(array![1.0, 0.2]),
            ],
        },
        Query::InfoServer,
    ]);
    let mut expected = ServerResult::with_capacity(7);
    expected.push(Err("Store Main not found".to_string()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Set(StoreUpsert {
        inserted: 2,
        updated: 0,
    })));
    expected.push(Err(
        "Store dimension is [2], input dimension of [3] was specified".to_string(),
    ));
    expected.push(Ok(ServerResponse::Get(vec![])));
    expected.push(Ok(ServerResponse::Get(vec![
        (
            StoreKey(array![1.2, 0.3]),
            HashMap::from_iter([(
                MetadataKey::new("title".into()),
                MetadataValue::new("elf".into()),
            )]),
        ),
        (
            StoreKey(array![1.0, 0.2]),
            HashMap::from_iter([(
                MetadataKey::new("title".into()),
                MetadataValue::new("sorcerer".into()),
            )]),
        ),
    ])));
    expected.push(Ok(ServerResponse::InfoServer(ServerInfo {
        address: "127.0.0.1:1369".to_string(),
        version: *ahnlich_types::version::VERSION,
        r#type: ahnlich_types::server::ServerType::Database,
        limit: CONFIG.allocator_size,
        remaining: 1073609219,
    })));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_create_index() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store does not yet exist
        Query::CreateIndex {
            store: StoreName("Main".to_string()),
            predicates: HashSet::from_iter([MetadataKey::new("planet".into())]),
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(2).unwrap(),
            create_predicates: HashSet::from_iter([MetadataKey::new("galaxy".into())]),
            error_if_exists: true,
        },
        Query::Set {
            store: StoreName("Main".to_string()),
            inputs: vec![
                (
                    StoreKey(array![1.4, 1.5]),
                    HashMap::from_iter([
                        (
                            MetadataKey::new("galaxy".into()),
                            MetadataValue::new("andromeda".into()),
                        ),
                        (
                            MetadataKey::new("life-form".into()),
                            MetadataValue::new("insects".into()),
                        ),
                    ]),
                ),
                (
                    StoreKey(array![1.6, 1.7]),
                    HashMap::from_iter([
                        (
                            MetadataKey::new("galaxy".into()),
                            MetadataValue::new("milkyway".into()),
                        ),
                        (
                            MetadataKey::new("life-form".into()),
                            MetadataValue::new("insects".into()),
                        ),
                    ]),
                ),
            ],
        },
        // should return CreateIndex(0) as nothing new was indexed
        Query::CreateIndex {
            store: StoreName("Main".to_string()),
            predicates: HashSet::from_iter([MetadataKey::new("galaxy".into())]),
        },
        // get predicate should work as galaxy is indexed
        Query::GetPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("galaxy".into()),
                value: MetadataValue::new("milkyway".into()),
            }),
        },
        // get predicate should fail as life-form is not indexed
        Query::GetPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("life-form".into()),
                value: MetadataValue::new("humanoid".into()),
            }),
        },
        // should create 2 new indexes
        Query::CreateIndex {
            store: StoreName("Main".to_string()),
            predicates: HashSet::from_iter([
                MetadataKey::new("technology".into()),
                MetadataKey::new("life-form".into()),
                MetadataKey::new("galaxy".into()),
            ]),
        },
        // now get pred for life-form should work as it is indexed
        Query::GetPred {
            store: StoreName("Main".to_string()),
            condition: PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("life-form".into()),
                value: MetadataValue::new("humanoid".into()),
            }),
        },
    ]);
    let mut expected = ServerResult::with_capacity(8);
    expected.push(Err("Store Main not found".into()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Set(StoreUpsert {
        inserted: 2,
        updated: 0,
    })));
    expected.push(Ok(ServerResponse::CreateIndex(0)));
    expected.push(Ok(ServerResponse::Get(vec![(
        StoreKey(array![1.6, 1.7]),
        HashMap::from_iter([
            (
                MetadataKey::new("galaxy".into()),
                MetadataValue::new("milkyway".into()),
            ),
            (
                MetadataKey::new("life-form".into()),
                MetadataValue::new("insects".into()),
            ),
        ]),
    )])));
    expected.push(Err(
        "Predicate life-form not found in store, attempt CREATEINDEX with predicate".into(),
    ));
    expected.push(Ok(ServerResponse::CreateIndex(2)));
    expected.push(Ok(ServerResponse::Get(vec![])));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_drop_index() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should error as store does not yet exist
        Query::DropIndex {
            store: StoreName("Main".to_string()),
            error_if_not_exists: true,
            predicates: HashSet::from_iter([MetadataKey::new("planet".into())]),
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(3).unwrap(),
            create_predicates: HashSet::from_iter([MetadataKey::new("galaxy".into())]),
            error_if_exists: true,
        },
        // should not error even though predicate does not exist
        Query::DropIndex {
            store: StoreName("Main".to_string()),
            error_if_not_exists: false,
            predicates: HashSet::from_iter([MetadataKey::new("planet".into())]),
        },
        // should error as predicate does not exist
        Query::DropIndex {
            store: StoreName("Main".to_string()),
            error_if_not_exists: true,
            predicates: HashSet::from_iter([MetadataKey::new("planet".into())]),
        },
        // should not error
        Query::DropIndex {
            store: StoreName("Main".to_string()),
            error_if_not_exists: true,
            predicates: HashSet::from_iter([MetadataKey::new("galaxy".into())]),
        },
    ]);
    let mut expected = ServerResult::with_capacity(5);
    expected.push(Err("Store Main not found".into()));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Del(0)));
    expected.push(Err(
        "Predicate planet not found in store, attempt CREATEINDEX with predicate".into(),
    ));
    expected.push(Ok(ServerResponse::Del(1)));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_drop_stores() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should not error as error if not exists is set to false, however it should return Del(0)
        Query::DropStore {
            store: StoreName("Main".to_string()),
            error_if_not_exists: false,
        },
        Query::CreateStore {
            store: StoreName("Main".to_string()),
            dimension: NonZeroUsize::new(3).unwrap(),
            create_predicates: HashSet::new(),
            error_if_exists: true,
        },
        Query::ListStores,
        // should not error
        Query::DropStore {
            store: StoreName("Main".to_string()),
            error_if_not_exists: true,
        },
        // should error
        Query::DropStore {
            store: StoreName("Main".to_string()),
            error_if_not_exists: true,
        },
    ]);
    let mut expected = ServerResult::with_capacity(5);
    expected.push(Ok(ServerResponse::Del(0)));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 0,
            size_in_bytes: 1712,
        },
    ]))));
    expected.push(Ok(ServerResponse::Del(1)));
    expected.push(Err("Store Main not found".to_string()));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_run_server_echos() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let tasks = vec![
        tokio::spawn(async move {
            let message = ServerQuery::from_queries(&[Query::InfoServer, Query::Ping]);
            let mut expected = ServerResult::with_capacity(2);
            expected.push(Ok(ServerResponse::InfoServer(ServerInfo {
                address: "127.0.0.1:1369".to_string(),
                version: *ahnlich_types::version::VERSION,
                r#type: ahnlich_types::server::ServerType::Database,
                limit: CONFIG.allocator_size,
                remaining: 1073614873,
            })));
            expected.push(Ok(ServerResponse::Pong));
            let stream = TcpStream::connect(address).await.unwrap();
            let mut reader = BufReader::new(stream);
            query_server_assert_result(&mut reader, message, expected).await
        }),
        tokio::spawn(async move {
            let message = ServerQuery::from_queries(&[Query::Ping, Query::InfoServer]);
            let mut expected = ServerResult::with_capacity(2);
            expected.push(Ok(ServerResponse::Pong));
            expected.push(Ok(ServerResponse::InfoServer(ServerInfo {
                address: "127.0.0.1:1369".to_string(),
                version: *ahnlich_types::version::VERSION,
                r#type: ahnlich_types::server::ServerType::Database,
                limit: CONFIG.allocator_size,
                remaining: 1073614873,
            })));
            let stream = TcpStream::connect(address).await.unwrap();
            let mut reader = BufReader::new(stream);
            query_server_assert_result(&mut reader, message, expected).await
        }),
    ];
    join_all(tasks).await;
}

async fn query_server_assert_result(
    reader: &mut BufReader<TcpStream>,
    query: ServerQuery,
    expected_result: ServerResult,
) {
    // Message to send
    let serialized_message = query.serialize().unwrap();

    // Send the message
    reader.write_all(&serialized_message).await.unwrap();

    // get length of response header
    let mut header = [0u8; ahnlich_types::bincode::RESPONSE_HEADER_LEN];
    timeout(Duration::from_secs(1), reader.read_exact(&mut header))
        .await
        .unwrap()
        .unwrap();
    let mut length_header = [0u8; ahnlich_types::bincode::LENGTH_HEADER_SIZE];
    length_header.copy_from_slice(&header[13..=20]);

    // read only the actual length size
    let data_length = u64::from_le_bytes(length_header);
    let mut response = vec![0u8; data_length as usize];

    timeout(Duration::from_secs(1), reader.read_exact(&mut response))
        .await
        .unwrap()
        .unwrap();

    let response = ServerResult::deserialize(&response).unwrap();

    assert_eq!(response, expected_result);
}
