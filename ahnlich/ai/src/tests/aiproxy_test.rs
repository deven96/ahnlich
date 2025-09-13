use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;

use utils::server::AhnlichServerUtils;

use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::{collections::HashMap, net::SocketAddr, sync::atomic::Ordering};

use crate::{
    cli::{AIProxyConfig, server::SupportedModels},
    engine::ai::models::ModelDetails,
    error::AIProxyError,
    server::handler::AIProxyServer,
};

use ahnlich_types::{
    ai::server::GetSimNEntry,
    keyval::StoreKey,
    metadata::{MetadataValue, metadata_value::Value as MValue},
};
use ahnlich_types::{ai::server::SingleInputToEmbedding, algorithm::algorithms::Algorithm};
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

use std::path::PathBuf;
use tokio::time::Duration;
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

#[tokio::test]
async fn test_convert_store_input_to_embeddings() {
    let address = provision_test_servers().await;

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");

    let mut client = AiServiceClient::connect(channel).await.expect("Failure");

    fn all_models() -> Vec<i32> {
        use AiModel::*;
        vec![
            AllMiniLmL6V2 as i32,
            AllMiniLmL12V2 as i32,
            BgeBaseEnV15 as i32,
            BgeLargeEnV15 as i32,
            Resnet50 as i32,
            ClipVitB32Image as i32,
            ClipVitB32Text as i32,
        ]
    }

    for (index, &model) in all_models().iter().enumerate() {
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

        let matching_metadatavalue = if index_model_repr.input_type().as_str_name() == "RAW_STRING"
        {
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
            value: HashMap::from_iter([(
                matching_metadatakey.clone(),
                matching_metadatavalue.clone(),
            )]),
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

        let expected_entries = ai_response_types::StoreInputToEmbeddingsList {
            values: vec![
                SingleInputToEmbedding {
                    input: Some(store_input_1.clone()),
                    embedding: Some(StoreKey { key: vec![] }),
                },
                SingleInputToEmbedding {
                    input: Some(store_input_2.clone()),
                    embedding: Some(StoreKey { key: vec![] }),
                },
            ],
        };

        let store_inputs = vec![store_input_1, store_input_2];

        let query = ai_query_types::ConvertStoreInputToEmbeddings {
            store_inputs,
            preprocess_action: Some(PreprocessAction::NoPreprocessing.into()),
            model,
        };

        let response = client
            .convert_store_input_to_embeddings(tonic::Request::new(query))
            .await
            .expect("Failed to convert store input to embeddings");

        let response_entries = response.into_inner().values;

        assert_eq!(response_entries.len(), expected_entries.values.len());

        // Verify all expected entries are present (order-independent)
        for expected_entry in expected_entries.values {
            response_entries.iter().any(|e| {
                e.input == expected_entry.input
                    && e.embedding.is_some()
                    && e.embedding.as_ref().unwrap().key.len() > 0
            });
        }
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
