use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::{collections::HashMap, net::SocketAddr};
use tokio::time::Duration;
use tonic::transport::Channel;
use utils::server::AhnlichServerUtils;

use crate::{
    cli::{AIProxyConfig, server::SupportedModels},
    server::handler::AIProxyServer,
};

use ahnlich_types::{
    ai::{
        models::AiModel,
        pipeline::{self as ai_pipeline, ai_query::Query},
        preprocess::PreprocessAction,
        query as ai_query_types,
    },
    algorithm::algorithms::Algorithm,
    keyval::{AiStoreEntry, StoreInput, StoreValue, store_input::Value},
    metadata::{MetadataValue, metadata_value},
    services::ai_service::ai_service_client::AiServiceClient,
};

static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());
static AI_CONFIG: Lazy<AIProxyConfig> = Lazy::new(|| {
    AIProxyConfig::default()
        .os_select_port()
        .set_supported_models(vec![SupportedModels::SfaceYunet])
});

async fn provision_test_servers() -> SocketAddr {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let db_port = server.local_addr().unwrap().port();
    tokio::spawn(async move { server.start().await });

    let mut config = AI_CONFIG.clone();
    config.db_port = db_port;

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not initialize ai proxy");
    let ai_address = ai_server.local_addr().expect("Could not get local addr");
    tokio::spawn(async move { ai_server.start().await });

    tokio::time::sleep(Duration::from_millis(200)).await;
    ai_address
}

#[tokio::test]
async fn test_sface_yunet_face_detection() {
    // Scenario: one image with multiple faces is indexed.
    // Each detected face produces one 128-dim embedding stored as a separate entry.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "sface_yunet_detection_store".to_string();
    let image_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::SfaceYunet.into(),
                    index_model: AiModel::SfaceYunet.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: true,
                    store_original: false,
                })),
            },
            ai_pipeline::AiQuery {
                query: Some(Query::Set(ai_query_types::Set {
                    store: store_name.clone(),
                    inputs: vec![AiStoreEntry {
                        key: Some(StoreInput {
                            value: Some(Value::Image(image_bytes)),
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from([(
                                "label".to_string(),
                                MetadataValue {
                                    value: Some(metadata_value::Value::RawString(
                                        "group_photo".to_string(),
                                    )),
                                },
                            )]),
                        }),
                    }],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: HashMap::new(),
                })),
            },
        ],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Pipeline request failed");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        let upsert = set_response.upsert.as_ref().expect("Expected upsert info");
        // YuNet should detect exactly 6 faces in the group photo
        assert_eq!(
            upsert.inserted, 6,
            "Expected 6 face embeddings, got {}",
            upsert.inserted
        );
        assert_eq!(upsert.updated, 0);
        println!("✓ Detected {} faces from group photo", upsert.inserted);
    } else {
        panic!("Expected Set response, got: {:?}", responses.get(1));
    }
}

#[tokio::test]
async fn test_sface_yunet_single_face() {
    // Scenario: an image with exactly one face is indexed.
    // Exactly one 128-dim embedding is stored; the Set reports inserted=1.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "sface_yunet_single_face_store".to_string();
    let image_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::SfaceYunet.into(),
                    index_model: AiModel::SfaceYunet.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: true,
                    store_original: false,
                })),
            },
            ai_pipeline::AiQuery {
                query: Some(Query::Set(ai_query_types::Set {
                    store: store_name.clone(),
                    inputs: vec![AiStoreEntry {
                        key: Some(StoreInput {
                            value: Some(Value::Image(image_bytes)),
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from([(
                                "label".to_string(),
                                MetadataValue {
                                    value: Some(metadata_value::Value::RawString(
                                        "single_face".to_string(),
                                    )),
                                },
                            )]),
                        }),
                    }],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: HashMap::new(),
                })),
            },
        ],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Pipeline request failed");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        let upsert = set_response.upsert.as_ref().expect("Expected upsert info");
        assert_eq!(upsert.inserted, 1, "Expected exactly 1 face embedding");
        assert_eq!(upsert.updated, 0);
    } else {
        panic!("Expected Set response, got: {:?}", responses.get(1));
    }
}

#[tokio::test]
async fn test_sface_yunet_no_faces() {
    // Scenario: an image with no detectable faces is indexed.
    // YuNet returns zero detections; the Set succeeds with 0 insertions.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "sface_yunet_no_faces_store".to_string();
    let image_bytes = include_bytes!("../../test_data/no_face.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::SfaceYunet.into(),
                    index_model: AiModel::SfaceYunet.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: true,
                    store_original: false,
                })),
            },
            ai_pipeline::AiQuery {
                query: Some(Query::Set(ai_query_types::Set {
                    store: store_name.clone(),
                    inputs: vec![AiStoreEntry {
                        key: Some(StoreInput {
                            value: Some(Value::Image(image_bytes)),
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from([(
                                "label".to_string(),
                                MetadataValue {
                                    value: Some(metadata_value::Value::RawString(
                                        "no_face".to_string(),
                                    )),
                                },
                            )]),
                        }),
                    }],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: HashMap::new(),
                })),
            },
        ],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Pipeline request failed");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        let upsert = set_response.upsert.as_ref().expect("Expected upsert info");
        assert_eq!(upsert.inserted, 0, "No faces should be stored");
        assert_eq!(upsert.updated, 0);
    } else {
        panic!("Expected Set response, got: {:?}", responses.get(1));
    }
}

#[tokio::test]
async fn test_sface_yunet_get_sim_n() {
    // Scenario: a single-face image and a multi-face image are indexed together.
    // Querying with the single-face image returns itself as top result with
    // cosine similarity > 0.99, and all other faces with lower similarity.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "sface_yunet_sim_store".to_string();
    let single_face_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();
    let multi_face_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::SfaceYunet.into(),
                    index_model: AiModel::SfaceYunet.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: true,
                    store_original: false,
                })),
            },
            ai_pipeline::AiQuery {
                query: Some(Query::Set(ai_query_types::Set {
                    store: store_name.clone(),
                    inputs: vec![
                        AiStoreEntry {
                            key: Some(StoreInput {
                                value: Some(Value::Image(single_face_bytes.clone())),
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from([(
                                    "label".to_string(),
                                    MetadataValue {
                                        value: Some(metadata_value::Value::RawString(
                                            "single".to_string(),
                                        )),
                                    },
                                )]),
                            }),
                        },
                        AiStoreEntry {
                            key: Some(StoreInput {
                                value: Some(Value::Image(multi_face_bytes)),
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from([(
                                    "label".to_string(),
                                    MetadataValue {
                                        value: Some(metadata_value::Value::RawString(
                                            "group".to_string(),
                                        )),
                                    },
                                )]),
                            }),
                        },
                    ],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: HashMap::new(),
                })),
            },
            ai_pipeline::AiQuery {
                query: Some(Query::GetSimN(ai_query_types::GetSimN {
                    store: store_name.clone(),
                    search_input: Some(StoreInput {
                        value: Some(Value::Image(single_face_bytes)),
                    }),
                    closest_n: 10,
                    algorithm: Algorithm::CosineSimilarity.into(),
                    condition: None,
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: HashMap::new(),
                })),
            },
        ],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Pipeline request failed");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 3);

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::GetSimN(sim_response)),
    }) = responses.get(2)
    {
        assert!(
            !sim_response.entries.is_empty(),
            "Expected at least one result"
        );
        let top = &sim_response.entries[0];
        let top_similarity = top
            .similarity
            .as_ref()
            .expect("Top result should have similarity")
            .value;
        assert!(
            top_similarity > 0.99,
            "Top result should be the same face (similarity={top_similarity})"
        );
        println!("✓ Top similarity: {top_similarity:.4}");
    } else {
        panic!("Expected GetSimN response, got: {:?}", responses.get(2));
    }
}

#[tokio::test]
async fn test_sface_yunet_multi_face_query_errors() {
    // Scenario: a GetSimN query is issued with a multi-face image.
    // A query must produce exactly one embedding; multiple faces make it ambiguous.
    // The server must reject this with InvalidArgument.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "sface_yunet_multi_query_err_store".to_string();
    let single_face_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();
    let multi_face_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::SfaceYunet.into(),
                    index_model: AiModel::SfaceYunet.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: true,
                    store_original: false,
                })),
            },
            ai_pipeline::AiQuery {
                query: Some(Query::Set(ai_query_types::Set {
                    store: store_name.clone(),
                    inputs: vec![AiStoreEntry {
                        key: Some(StoreInput {
                            value: Some(Value::Image(single_face_bytes)),
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    }],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: HashMap::new(),
                })),
            },
        ],
    };

    client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Setup pipeline failed");

    // Now query with multi-face image — must error
    let get_query = ai_query_types::GetSimN {
        store: store_name.clone(),
        search_input: Some(StoreInput {
            value: Some(Value::Image(multi_face_bytes)),
        }),
        closest_n: 5,
        algorithm: Algorithm::CosineSimilarity.into(),
        condition: None,
        preprocess_action: PreprocessAction::ModelPreprocessing.into(),
        execution_provider: None,
        model_params: HashMap::new(),
    };

    let result = client.get_sim_n(tonic::Request::new(get_query)).await;

    assert!(result.is_err(), "Expected error for multi-face query");
    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::InvalidArgument,
        "Expected InvalidArgument, got: {:?}",
        status.code()
    );
    assert!(
        status.message().contains("embeddings"),
        "Expected embedding count in error message, got: {}",
        status.message()
    );
    println!(
        "✓ Multi-face query correctly rejected: {}",
        status.message()
    );
}

#[tokio::test]
async fn test_sface_yunet_mixed_batch_no_face_does_not_fail_batch() {
    // Scenario: a batch contains a face image and a no-face image.
    // The no-face image produces zero embeddings and is silently skipped.
    // The batch must succeed, storing only the face from the valid image.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "sface_yunet_mixed_batch_store".to_string();
    let single_face_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();
    let no_face_bytes = include_bytes!("../../test_data/no_face.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::SfaceYunet.into(),
                    index_model: AiModel::SfaceYunet.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: true,
                    store_original: false,
                })),
            },
            ai_pipeline::AiQuery {
                query: Some(Query::Set(ai_query_types::Set {
                    store: store_name.clone(),
                    inputs: vec![
                        // Has a face — should produce 1 stored embedding
                        AiStoreEntry {
                            key: Some(StoreInput {
                                value: Some(Value::Image(single_face_bytes)),
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from([(
                                    "source".to_string(),
                                    MetadataValue {
                                        value: Some(metadata_value::Value::RawString(
                                            "has_face".to_string(),
                                        )),
                                    },
                                )]),
                            }),
                        },
                        // No face — produces zero embeddings, silently skipped
                        AiStoreEntry {
                            key: Some(StoreInput {
                                value: Some(Value::Image(no_face_bytes)),
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from([(
                                    "source".to_string(),
                                    MetadataValue {
                                        value: Some(metadata_value::Value::RawString(
                                            "no_face".to_string(),
                                        )),
                                    },
                                )]),
                            }),
                        },
                    ],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: HashMap::new(),
                })),
            },
        ],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Mixed batch Set must not fail even when one entry has no faces");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        let upsert = set_response.upsert.as_ref().expect("Expected upsert info");
        assert_eq!(
            upsert.inserted, 1,
            "Only the face from the valid image should be stored; no-face image is skipped"
        );
        assert_eq!(upsert.updated, 0);
    } else {
        panic!("Expected Set response, got: {:?}", responses.get(1));
    }
}

#[tokio::test]
async fn test_sface_yunet_high_confidence_threshold() {
    // Scenario: Test that model_params confidence_threshold works correctly for SFaceYunet.
    // With a higher threshold (0.75), fewer faces should be detected compared to default (0.6).
    // The Friends cast image should have fewer detections with higher threshold.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "sface_yunet_high_threshold_store".to_string();
    let image_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();

    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::SfaceYunet.into(),
            index_model: AiModel::SfaceYunet.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })),
    };

    // First, insert with default threshold (0.6) to establish baseline
    let set_default_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Image(image_bytes.clone())),
                }),
                value: Some(StoreValue {
                    value: HashMap::new(),
                }),
            }],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(), // Empty = use default threshold (0.6)
        })),
    };

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_default_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline with default threshold");

    let responses = response.into_inner().responses;
    let default_face_count = if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        set_response
            .upsert
            .as_ref()
            .expect("Expected upsert info")
            .inserted
    } else {
        panic!("Expected Set response");
    };

    // Now test with high threshold (0.89)
    // Use a different store name to avoid any timing issues with drop/recreate
    let high_threshold_store_name = format!("{}_high", store_name);

    let recreate_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: high_threshold_store_name.clone(),
            query_model: AiModel::SfaceYunet.into(),
            index_model: AiModel::SfaceYunet.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })),
    };

    let mut high_threshold_params = HashMap::new();
    high_threshold_params.insert("confidence_threshold".to_string(), "0.75".to_string());

    let set_high_threshold_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: high_threshold_store_name.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Image(image_bytes)),
                }),
                value: Some(StoreValue {
                    value: HashMap::new(),
                }),
            }],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: high_threshold_params, // Custom high threshold
        })),
    };

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![recreate_store_query, set_high_threshold_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline with high threshold");

    let responses = response.into_inner().responses;
    let high_threshold_face_count = if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        set_response
            .upsert
            .as_ref()
            .expect("Expected upsert info")
            .inserted
    } else {
        panic!(
            "Expected Set response at index 1, got: {:?}",
            responses.get(1)
        );
    };

    // Verify that high threshold detects fewer faces than default
    println!(
        "SFaceYunet: Default threshold (0.6) detected {} faces, high threshold (0.75) detected {} faces",
        default_face_count, high_threshold_face_count
    );

    assert!(
        default_face_count >= 6,
        "Default threshold (0.6) should detect at least 6 faces"
    );
    assert!(
        high_threshold_face_count < default_face_count,
        "High confidence threshold (0.75) should detect fewer faces than default (0.6). \
         Got {} faces with 0.75 vs {} with 0.6",
        high_threshold_face_count,
        default_face_count
    );
    assert!(
        high_threshold_face_count > 0,
        "Even with high threshold (0.75), at least one clear face should be detected"
    );
}
