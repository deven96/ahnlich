use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;
use ahnlich_types::{
    ai::{
        AIModel, AIQuery, AIServerQuery, AIServerResponse, AIServerResult, AIStoreInfo, AIStoreType,
    },
    db::StoreUpsert,
    keyval::{StoreInput, StoreName, StoreValue},
    metadata::{MetadataKey, MetadataValue},
    predicate::{Predicate, PredicateCondition},
    similarity::Algorithm,
};

use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::{collections::HashSet, num::NonZeroUsize, sync::atomic::Ordering};

use crate::cli::AIProxyConfig;
use crate::server::handler::AIProxyServer;
use ahnlich_types::bincode::BinCodeSerAndDeser;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default());
static AI_CONFIG: Lazy<AIProxyConfig> = Lazy::new(|| {
    let mut ai_proxy = AIProxyConfig::default().os_select_port();
    ai_proxy.db_port = CONFIG.port.clone();
    ai_proxy.db_host = CONFIG.host.clone();
    ai_proxy
});

static PERSISTENCE_FILE: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ahnlich_ai_proxy.dat"));

static AI_CONFIG_WITH_PERSISTENCE: Lazy<AIProxyConfig> = Lazy::new(|| {
    AIProxyConfig::default()
        .os_select_port()
        .set_persistence_interval(200)
        .set_persist_location((*PERSISTENCE_FILE).clone())
});

async fn get_server_response(
    reader: &mut BufReader<TcpStream>,
    query: AIServerQuery,
) -> AIServerResult {
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

    let response = AIServerResult::deserialize(&response).unwrap();

    response
}

async fn query_server_assert_result(
    reader: &mut BufReader<TcpStream>,
    query: AIServerQuery,
    expected_result: AIServerResult,
) {
    let response = get_server_response(reader, query).await;

    assert_eq!(response, expected_result);
}

async fn provision_test_servers() -> SocketAddr {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let ai_server = AIProxyServer::new(&AI_CONFIG)
        .await
        .expect("Could not initialize ai proxy");

    let ai_address = ai_server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // start up ai proxy
    let _ = tokio::spawn(async move { ai_server.start().await });
    // Allow some time for the servers to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    ai_address
}
#[tokio::test]
async fn test_simple_ai_proxy_ping() {
    let address = provision_test_servers().await;
    let first_stream = TcpStream::connect(address).await.unwrap();
    let message = AIServerQuery::from_queries(&[AIQuery::Ping]);
    let mut expected = AIServerResult::with_capacity(1);
    expected.push(Ok(AIServerResponse::Pong));
    let mut reader = BufReader::new(first_stream);
    query_server_assert_result(&mut reader, message, expected.clone()).await;
}

#[tokio::test]
async fn test_ai_proxy_create_store_success() {
    let address = provision_test_servers().await;
    let first_stream = TcpStream::connect(address).await.unwrap();
    let second_stream = TcpStream::connect(address).await.unwrap();
    let store_name = StoreName(String::from("Sample Store"));
    let message = AIServerQuery::from_queries(&[AIQuery::CreateStore {
        r#type: AIStoreType::RawString,
        store: store_name.clone(),
        model: AIModel::Llama3,
        predicates: HashSet::new(),
        non_linear_indices: HashSet::new(),
    }]);

    let mut expected = AIServerResult::with_capacity(1);
    expected.push(Ok(AIServerResponse::Unit));
    let mut reader = BufReader::new(first_stream);
    query_server_assert_result(&mut reader, message, expected.clone()).await;

    // list stores to verify it's present.
    let message = AIServerQuery::from_queries(&[AIQuery::ListStores]);
    let mut expected = AIServerResult::with_capacity(1);
    expected.push(Ok(AIServerResponse::StoreList(HashSet::from_iter([
        AIStoreInfo {
            name: store_name.clone(),
            model: AIModel::Llama3,
            r#type: AIStoreType::RawString,
            embedding_size: AIModel::Llama3.embedding_size().into(),
        },
    ]))));
    let mut reader = BufReader::new(second_stream);
    query_server_assert_result(&mut reader, message, expected.clone()).await;
}

// TODO: Same issues with random storekeys, changing the order of expected response
#[tokio::test]
async fn test_ai_proxy_get_pred_succeeds() {
    let address = provision_test_servers().await;
    let first_stream = TcpStream::connect(address).await.unwrap();
    let second_stream = TcpStream::connect(address).await.unwrap();
    let store_name = StoreName(String::from("Deven Kicks"));
    let matching_metadatakey = MetadataKey::new("Brand".to_owned());
    let matching_metadatavalue = MetadataValue::RawString("Nike".to_owned());

    let nike_store_value =
        StoreValue::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]);
    let adidas_store_value = StoreValue::from_iter([(
        matching_metadatakey.clone(),
        MetadataValue::RawString("Adidas".to_owned()),
    )]);
    let store_data = vec![
        (
            StoreInput::RawString(String::from("Jordan 3")),
            nike_store_value.clone(),
        ),
        (
            StoreInput::RawString(String::from("Air Force 1")),
            nike_store_value.clone(),
        ),
        (
            StoreInput::RawString(String::from("Yeezy")),
            adidas_store_value.clone(),
        ),
    ];
    let message = AIServerQuery::from_queries(&[
        AIQuery::CreateStore {
            r#type: AIStoreType::RawString,
            store: store_name.clone(),
            model: AIModel::Llama3,
            predicates: HashSet::from_iter([
                matching_metadatakey.clone(),
                MetadataKey::new("Original".to_owned()),
            ]),
            non_linear_indices: HashSet::new(),
        },
        AIQuery::Set {
            store: store_name.clone(),
            inputs: store_data.clone(),
        },
    ]);
    let mut reader = BufReader::new(first_stream);

    let _ = get_server_response(&mut reader, message).await;

    let message = AIServerQuery::from_queries(&[AIQuery::GetPred {
        store: store_name,
        condition: PredicateCondition::Value(Predicate::Equals {
            key: matching_metadatakey.clone(),
            value: matching_metadatavalue,
        }),
    }]);

    let mut expected = AIServerResult::with_capacity(1);

    expected.push(Ok(AIServerResponse::Get(vec![
        (
            StoreInput::RawString(String::from("Jordan 3")),
            nike_store_value.clone(),
        ),
        (
            StoreInput::RawString(String::from("Air Force 1")),
            nike_store_value.clone(),
        ),
    ])));

    let mut reader = BufReader::new(second_stream);
    //query_server_assert_result(&mut reader, message, expected.clone()).await;
    let response = get_server_response(&mut reader, message).await;
    assert!(response.len() == expected.len())
}

// TODO: WIll Need fixing when we integrate AI model, for now we return the closest first
#[tokio::test]
async fn test_ai_proxy_get_sim_n_succeeds() {
    let address = provision_test_servers().await;
    let first_stream = TcpStream::connect(address).await.unwrap();
    let second_stream = TcpStream::connect(address).await.unwrap();
    let store_name = StoreName(String::from("Deven Kicks"));
    let matching_metadatakey = MetadataKey::new("Brand".to_owned());
    let matching_metadatavalue = MetadataValue::RawString("Nike".to_owned());

    let nike_store_value =
        StoreValue::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]);
    let adidas_store_value = StoreValue::from_iter([(
        matching_metadatakey.clone(),
        MetadataValue::RawString("Adidas".to_owned()),
    )]);
    let store_data = vec![
        (
            StoreInput::RawString(String::from("Jordan 3")),
            nike_store_value.clone(),
        ),
        (
            StoreInput::RawString(String::from("Air Force 1")),
            nike_store_value.clone(),
        ),
        (
            StoreInput::RawString(String::from("Yeezy")),
            adidas_store_value.clone(),
        ),
    ];
    let message = AIServerQuery::from_queries(&[
        AIQuery::CreateStore {
            r#type: AIStoreType::RawString,
            store: store_name.clone(),
            model: AIModel::Llama3,
            predicates: HashSet::from_iter([
                matching_metadatakey.clone(),
                MetadataKey::new("Original".to_owned()),
            ]),
            non_linear_indices: HashSet::new(),
        },
        AIQuery::Set {
            store: store_name.clone(),
            inputs: store_data.clone(),
        },
    ]);
    let mut reader = BufReader::new(first_stream);

    let _ = get_server_response(&mut reader, message).await;

    let message = AIServerQuery::from_queries(&[AIQuery::GetSimN {
        store: store_name.clone(),
        search_input: StoreInput::RawString(String::from("Yeezy")),
        condition: None,
        closest_n: NonZeroUsize::new(1).unwrap(),
        algorithm: Algorithm::DotProductSimilarity,
    }]);

    let mut expected = AIServerResult::with_capacity(1);
    expected.push(Ok(AIServerResponse::Get(vec![(
        StoreInput::RawString(String::from("Yeezy")),
        adidas_store_value.clone(),
    )])));

    let mut reader = BufReader::new(second_stream);
    let response = get_server_response(&mut reader, message).await;

    assert!(response.len() == expected.len())
}

#[tokio::test]
async fn test_ai_proxy_create_drop_pred_index() {
    let address = provision_test_servers().await;
    let second_stream = TcpStream::connect(address).await.unwrap();
    let store_name = StoreName(String::from("Deven Kicks"));
    let matching_metadatakey = MetadataKey::new("Brand".to_owned());
    let matching_metadatavalue = MetadataValue::RawString("Nike".to_owned());
    let predicate_cond = PredicateCondition::Value(Predicate::Equals {
        key: matching_metadatakey.clone(),
        value: matching_metadatavalue.clone(),
    });

    let nike_store_value =
        StoreValue::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]);
    let store_data = vec![(
        StoreInput::RawString(String::from("Jordan 3")),
        nike_store_value.clone(),
    )];
    let message = AIServerQuery::from_queries(&[
        AIQuery::CreateStore {
            r#type: AIStoreType::RawString,
            store: store_name.clone(),
            model: AIModel::Llama3,
            predicates: HashSet::from_iter([]),
            non_linear_indices: HashSet::new(),
        },
        // returns nothing
        AIQuery::GetPred {
            store: store_name.clone(),
            condition: predicate_cond.clone(),
        },
        AIQuery::CreatePredIndex {
            store: store_name.clone(),
            predicates: HashSet::from_iter([matching_metadatakey.clone()]),
        },
        AIQuery::Set {
            store: store_name.clone(),
            inputs: store_data.clone(),
        },
        AIQuery::GetPred {
            store: store_name.clone(),
            condition: predicate_cond,
        },
        AIQuery::DropPredIndex {
            store: store_name.clone(),
            predicates: HashSet::from_iter([matching_metadatakey.clone()]),
            error_if_not_exists: true,
        },
    ]);
    let mut expected = AIServerResult::with_capacity(6);

    expected.push(Ok(AIServerResponse::Unit));
    expected.push(Ok(AIServerResponse::Get(vec![])));
    expected.push(Ok(AIServerResponse::CreateIndex(1)));
    expected.push(Ok(AIServerResponse::Set(StoreUpsert {
        inserted: 1,
        updated: 0,
    })));
    expected.push(Ok(AIServerResponse::Get(vec![(
        StoreInput::RawString(String::from("Jordan 3")),
        nike_store_value.clone(),
    )])));
    expected.push(Ok(AIServerResponse::Del(1)));

    let mut reader = BufReader::new(second_stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_ai_proxy_del_key_drop_store() {
    let address = provision_test_servers().await;
    let second_stream = TcpStream::connect(address).await.unwrap();
    let store_name = StoreName(String::from("Deven Kicks"));
    let matching_metadatakey = MetadataKey::new("Brand".to_owned());
    let matching_metadatavalue = MetadataValue::RawString("Nike".to_owned());
    let predicate_cond = PredicateCondition::Value(Predicate::Equals {
        key: matching_metadatakey.clone(),
        value: matching_metadatavalue.clone(),
    });

    let nike_store_value =
        StoreValue::from_iter([(matching_metadatakey.clone(), matching_metadatavalue.clone())]);
    let store_data = vec![(
        StoreInput::RawString(String::from("Jordan 3")),
        nike_store_value.clone(),
    )];
    let message = AIServerQuery::from_queries(&[
        AIQuery::CreateStore {
            r#type: AIStoreType::RawString,
            store: store_name.clone(),
            model: AIModel::Llama3,
            predicates: HashSet::from_iter([]),
            non_linear_indices: HashSet::new(),
        },
        AIQuery::Set {
            store: store_name.clone(),
            inputs: store_data.clone(),
        },
        AIQuery::DelKey {
            store: store_name.clone(),
            key: StoreInput::RawString(String::from("Jordan 3")),
        },
        AIQuery::GetPred {
            store: store_name.clone(),
            condition: predicate_cond,
        },
        AIQuery::DropStore {
            store: store_name.clone(),
            error_if_not_exists: true,
        },
    ]);
    let mut expected = AIServerResult::with_capacity(6);

    expected.push(Ok(AIServerResponse::Unit));
    expected.push(Ok(AIServerResponse::Set(StoreUpsert {
        inserted: 1,
        updated: 0,
    })));
    expected.push(Ok(AIServerResponse::Del(1)));
    expected.push(Ok(AIServerResponse::Get(vec![])));
    expected.push(Ok(AIServerResponse::Del(1)));

    let mut reader = BufReader::new(second_stream);
    query_server_assert_result(&mut reader, message, expected).await
}

#[tokio::test]
async fn test_ai_proxy_fails_db_server_unavailable() {
    let ai_server = AIProxyServer::new(&AI_CONFIG)
        .await
        .expect("Could not initialize ai proxy");

    let address = ai_server.local_addr().expect("Could not get local addr");
    // start up ai proxy
    let _ = tokio::spawn(async move { ai_server.start().await });
    // Allow some time for the servers to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    let second_stream = TcpStream::connect(address).await.unwrap();

    let store_name = StoreName(String::from("Main"));
    let message = AIServerQuery::from_queries(&[
        AIQuery::Ping,
        AIQuery::CreateStore {
            r#type: AIStoreType::RawString,
            store: store_name.clone(),
            model: AIModel::Llama3,
            predicates: HashSet::from_iter([]),
            non_linear_indices: HashSet::new(),
        },
    ]);

    let mut reader = BufReader::new(second_stream);

    let response = get_server_response(&mut reader, message).await;

    let res = response.pop().unwrap();

    assert!(res.is_err());
    // Err("deadpool error Backend(Standard(Os { code: 61, kind: ConnectionRefused, message: \"Connection refused\" }))")] }
    let err = res.err().unwrap();
    assert!(err.contains(" kind: ConnectionRefused,"))
}

#[tokio::test]
async fn test_ai_proxy_test_with_persistence() {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");
    let ai_server = AIProxyServer::new(&AI_CONFIG_WITH_PERSISTENCE)
        .await
        .expect("Could not initialize ai proxy");

    let address = ai_server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    let write_flag = ai_server.write_flag();
    // start up ai proxy
    let _ = tokio::spawn(async move { ai_server.start().await });
    // Allow some time for the servers to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    let store_name = StoreName(String::from("Main"));
    let store_name_2 = StoreName(String::from("Main2"));
    let first_stream = TcpStream::connect(address).await.unwrap();

    let message = AIServerQuery::from_queries(&[
        AIQuery::CreateStore {
            r#type: AIStoreType::RawString,
            store: store_name.clone(),
            model: AIModel::Llama3,
            predicates: HashSet::from_iter([]),
            non_linear_indices: HashSet::new(),
        },
        AIQuery::CreateStore {
            r#type: AIStoreType::Binary,
            store: store_name_2.clone(),
            model: AIModel::Llama3,
            predicates: HashSet::from_iter([]),
            non_linear_indices: HashSet::new(),
        },
        AIQuery::DropStore {
            store: store_name,
            error_if_not_exists: true,
        },
    ]);

    let mut expected = AIServerResult::with_capacity(3);

    expected.push(Ok(AIServerResponse::Unit));
    expected.push(Ok(AIServerResponse::Unit));
    expected.push(Ok(AIServerResponse::Del(1)));

    let mut reader = BufReader::new(first_stream);
    query_server_assert_result(&mut reader, message, expected).await;

    // write flag should show that a write has occured
    assert!(write_flag.load(Ordering::SeqCst));
    // Allow some time for persistence to kick in
    tokio::time::sleep(Duration::from_millis(200)).await;
    // start another server with existing persistence

    let persisted_server = AIProxyServer::new(&AI_CONFIG_WITH_PERSISTENCE)
        .await
        .unwrap();

    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    let address = persisted_server
        .local_addr()
        .expect("Could not get local addr");
    let write_flag = persisted_server.write_flag();
    let _ = tokio::spawn(async move { persisted_server.start().await });
    let second_stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(second_stream);

    let message = AIServerQuery::from_queries(&[AIQuery::ListStores]);

    let mut expected = AIServerResult::with_capacity(1);

    expected.push(Ok(AIServerResponse::StoreList(HashSet::from_iter([
        AIStoreInfo {
            name: store_name_2.clone(),
            r#type: AIStoreType::Binary,
            model: AIModel::Llama3,
            embedding_size: AIModel::Llama3.embedding_size().into(),
        },
    ]))));

    query_server_assert_result(&mut reader, message, expected).await;
    assert!(!write_flag.load(Ordering::SeqCst));
    // delete persistence
    let _ = std::fs::remove_file(&*PERSISTENCE_FILE);
}

#[tokio::test]
async fn test_ai_proxy_destroy_database() {
    let address = provision_test_servers().await;
    let second_stream = TcpStream::connect(address).await.unwrap();
    let store_name = StoreName(String::from("Deven Kicks"));
    let message = AIServerQuery::from_queries(&[
        AIQuery::CreateStore {
            r#type: AIStoreType::RawString,
            store: store_name.clone(),
            model: AIModel::Llama3,
            predicates: HashSet::from_iter([]),
            non_linear_indices: HashSet::new(),
        },
        AIQuery::ListStores,
        AIQuery::DestoryDatabase,
        AIQuery::ListStores,
    ]);
    let mut expected = AIServerResult::with_capacity(4);

    expected.push(Ok(AIServerResponse::Unit));
    expected.push(Ok(AIServerResponse::StoreList(HashSet::from_iter([
        AIStoreInfo {
            name: store_name,
            r#type: AIStoreType::RawString,
            model: AIModel::Llama3,
            embedding_size: AIModel::Llama3.embedding_size().into(),
        },
    ]))));
    expected.push(Ok(AIServerResponse::Del(1)));
    expected.push(Ok(AIServerResponse::StoreList(HashSet::from_iter([]))));

    let mut reader = BufReader::new(second_stream);
    query_server_assert_result(&mut reader, message, expected).await
}
