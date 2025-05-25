use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;

use utils::server::AhnlichServerUtils;

use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    num::NonZeroUsize,
    sync::atomic::Ordering,
};

use crate::{
    cli::{server::SupportedModels, AIProxyConfig},
    engine::ai::models::ModelDetails,
    server::handler::AIProxyServer,
};

use grpc_types::{
    ai::{
        models::AiModel,
        pipeline::{self as ai_pipeline, ai_query::Query},
        preprocess::PreprocessAction,
        query::{self as ai_query_types, StoreEntry},
        server::{self as ai_response_types, AiStoreInfo, GetEntry},
    },
    keyval::{store_input::Value, StoreInput, StoreName, StoreValue},
    services::ai_service::ai_service_client::AiServiceClient,
};

use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
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

#[tokio::test]
async fn test_simple_ai_proxy_ping() {
    let address = provision_test_servers().await;

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");

    let mut client = AiServiceClient::connect(channel).await.expect("Failure");

    let response = client
        .ping(tonic::Request::new(grpc_types::ai::query::Ping {}))
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
    let create_store = grpc_types::ai::query::CreateStore {
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
    let message = grpc_types::ai::query::ListStores {};
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

    let inputs = vec![StoreEntry {
        key: Some(store_entry_input.clone()),
        value: HashMap::new(),
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
                store: "Main".to_string(),
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

    let message = grpc_types::ai::query::GetKey {
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
