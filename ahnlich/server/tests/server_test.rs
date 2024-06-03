use futures::future::join_all;
use ndarray::array;
use server::cli::ServerConfig;
use std::collections::HashMap;
use std::collections::HashSet;
use std::num::NonZeroUsize;
use std::time::SystemTime;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use types::bincode::BinCodeSerAndDeser;
use types::keyval::StoreKey;
use types::keyval::StoreName;
use types::metadata::MetadataKey;
use types::metadata::MetadataValue;
use types::query::Query;
use types::query::ServerQuery;
use types::server::ConnectedClient;
use types::server::ServerInfo;
use types::server::ServerResponse;
use types::server::ServerResult;
use types::server::StoreInfo;
use types::server::StoreUpsert;

#[tokio::test]
async fn test_server_client_info() {
    let server = server::Server::new(&ServerConfig::default())
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
    let server = server::Server::new(&ServerConfig::default())
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
    let server = server::Server::new(&ServerConfig::default())
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
async fn test_set_in_store() {
    let server = server::Server::new(&ServerConfig::default())
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
async fn test_drop_stores() {
    let server = server::Server::new(&ServerConfig::default())
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let message = ServerQuery::from_queries(&[
        // should not error
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
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Ok(ServerResponse::StoreList(HashSet::from_iter([
        StoreInfo {
            name: StoreName("Main".to_string()),
            len: 0,
            size_in_bytes: 1712,
        },
    ]))));
    expected.push(Ok(ServerResponse::Unit));
    expected.push(Err("Store Main not found".to_string()));
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_run_server_echos() {
    let server = server::Server::new(&ServerConfig::default())
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
                version: types::VERSION.to_string(),
                r#type: types::server::ServerType::Database,
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
                version: types::VERSION.to_string(),
                r#type: types::server::ServerType::Database,
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

    // get length of response
    let mut length_header = [0u8; types::bincode::LENGTH_HEADER_SIZE];
    timeout(
        Duration::from_secs(1),
        reader.read_exact(&mut length_header),
    )
    .await
    .unwrap()
    .unwrap();
    let data_length = u64::from_be_bytes(length_header);
    let mut response = vec![0u8; data_length as usize];

    timeout(Duration::from_secs(1), reader.read_exact(&mut response))
        .await
        .unwrap()
        .unwrap();

    let response = ServerResult::deserialize(false, &response).unwrap();

    assert_eq!(response, expected_result);
}
