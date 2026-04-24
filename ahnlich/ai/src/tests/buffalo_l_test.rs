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

/// Returns optimized model parameters for face detection tests.
/// Uses confidence=0.7 and NMS=0.2 for consistent test results.
fn optimized_face_params() -> HashMap<String, String> {
    let mut params = HashMap::new();
    params.insert("confidence_threshold".to_string(), "0.7".to_string());
    params.insert("nms_threshold".to_string(), "0.2".to_string());
    params
}

fn genderage_params() -> HashMap<String, String> {
    let mut params = HashMap::new();
    params.insert("confidence_threshold".to_string(), "0.7".to_string());
    params.insert("nms_threshold".to_string(), "0.2".to_string());
    params.insert("attributes".to_string(), "genderage".to_string());
    params
}

static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());
static AI_CONFIG: Lazy<AIProxyConfig> = Lazy::new(|| {
    AIProxyConfig::default()
        .os_select_port()
        .set_supported_models(vec![SupportedModels::BuffaloL])
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
async fn test_buffalo_l_face_detection() {
    // Scenario: one image with multiple faces is indexed.
    // Each detected face produces one embedding and is stored as a separate entry.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_store".to_string();

    let image_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();

    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::BuffaloL.into(),
            index_model: AiModel::BuffaloL.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })),
    };

    let set_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
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
            model_params: optimized_face_params(),
        })),
    };

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        assert!(set_response.upsert.is_some());
        let upsert = set_response.upsert.as_ref().unwrap();
        // Friends cast has 6 faces
        assert_eq!(
            upsert.inserted, 6,
            "Expected 6 faces from Friends cast image"
        );
        assert_eq!(upsert.updated, 0);
    } else {
        panic!("Expected Set response");
    }
}

#[tokio::test]
async fn test_buffalo_l_batch_multiple_images() {
    // Scenario: multiple copies of the same multi-face image are sent in one Set call.
    // Duplicate face embeddings are updated rather than re-inserted, so the inserted
    // count equals the number of unique faces (6), not total faces across all images.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_batch_store".to_string();

    let image_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();

    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::BuffaloL.into(),
            index_model: AiModel::BuffaloL.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })),
    };

    let set_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
            inputs: vec![
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::Image(image_bytes.clone())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::Image(image_bytes.clone())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::Image(image_bytes)),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                },
            ],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: optimized_face_params(),
        })),
    };

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        assert!(set_response.upsert.is_some());
        let upsert = set_response.upsert.as_ref().unwrap();
        // 3 identical images with 6 faces each = 6 unique + 12 updates (with confidence=0.7, NMS=0.2)
        assert_eq!(
            upsert.inserted, 6,
            "Expected 6 unique faces from 3 identical images (confidence=0.7, NMS=0.2)"
        );
        assert_eq!(
            upsert.updated, 12,
            "Expected 12 updates from duplicate faces (6 faces × 2 extra copies, confidence=0.7, NMS=0.2)"
        );
    } else {
        panic!("Expected Set response");
    }
}

#[tokio::test]
async fn test_buffalo_l_no_faces() {
    // Scenario: an image with no detectable faces is indexed.
    // Buffalo_L returns zero embeddings for that image; the Set succeeds with 0 insertions.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_no_faces".to_string();

    let no_face_image = include_bytes!("../../test_data/no_face.jpg").to_vec();

    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::BuffaloL.into(),
            index_model: AiModel::BuffaloL.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })),
    };

    let set_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Image(no_face_image)),
                }),
                value: Some(StoreValue {
                    value: HashMap::new(),
                }),
            }],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        })),
    };

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        assert!(set_response.upsert.is_some());
        let upsert = set_response.upsert.as_ref().unwrap();
        assert_eq!(
            upsert.inserted, 0,
            "Should not insert anything when no faces detected"
        );
        assert_eq!(upsert.updated, 0);
    } else {
        panic!(
            "Expected Set response with 0 insertions for no faces, got: {:?}",
            responses.get(1)
        );
    }
}

#[tokio::test]
async fn test_buffalo_l_single_face() {
    // Scenario: an image with exactly one face is indexed.
    // Exactly one embedding is stored; the Set reports inserted=1.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_single_face".to_string();

    let single_face_image = include_bytes!("../../test_data/single_face.jpg").to_vec();

    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::BuffaloL.into(),
            index_model: AiModel::BuffaloL.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })),
    };

    let set_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Image(single_face_image)),
                }),
                value: Some(StoreValue {
                    value: HashMap::new(),
                }),
            }],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        })),
    };

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        assert!(set_response.upsert.is_some());
        let upsert = set_response.upsert.as_ref().unwrap();
        assert_eq!(
            upsert.inserted, 1,
            "Should insert exactly 1 face from single face image"
        );
        assert_eq!(upsert.updated, 0);
    } else {
        panic!("Expected Set response");
    }
}

#[tokio::test]
async fn test_buffalo_l_get_sim_n() {
    // Scenario: a single-face image and a multi-face image are indexed together (7 faces total).
    // Querying with the single-face image returns itself as the top result with cosine
    // similarity > 0.99, and all other (different) faces with similarity < 0.7.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_similarity".to_string();

    // Load test images
    let single_face_image = include_bytes!("../../test_data/single_face.jpg").to_vec();
    let multi_face_image = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();

    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::BuffaloL.into(),
            index_model: AiModel::BuffaloL.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })),
    };

    // Insert both the single face AND the multi-face image
    // The single face should be most similar to itself
    let set_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
            inputs: vec![
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::Image(single_face_image.clone())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from([(
                            "source".to_string(),
                            MetadataValue {
                                value: Some(metadata_value::Value::RawString(
                                    "single_lena".to_string(),
                                )),
                            },
                        )]),
                    }),
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::Image(multi_face_image)),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from([(
                            "source".to_string(),
                            MetadataValue {
                                value: Some(metadata_value::Value::RawString(
                                    "friends_cast".to_string(),
                                )),
                            },
                        )]),
                    }),
                },
            ],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: optimized_face_params(),
        })),
    };

    // Query with the same single face - the top result should be itself with high similarity
    let get_sim_n_query = ai_pipeline::AiQuery {
        query: Some(Query::GetSimN(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::Image(single_face_image)),
            }),
            condition: None,
            closest_n: 10,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: optimized_face_params(),
        })),
    };

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_query, get_sim_n_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 3);

    // Check the Set response to know what was inserted
    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = responses.get(1)
    {
        assert!(set_response.upsert.is_some());
        let upsert = set_response.upsert.as_ref().unwrap();
        // single_face.jpg has 1 face, faces_multiple.jpg has 6-7 faces (sometimes detects duplicate)
        // with optimized params (confidence=0.7, NMS=0.2) = 7-8 total
        assert!(
            upsert.inserted >= 7 && upsert.inserted <= 8,
            "Should insert 7-8 faces (1 from single_face.jpg, 6-7 from faces_multiple.jpg), got {}",
            upsert.inserted
        );
    }

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::GetSimN(get_sim_n_response)),
    }) = responses.get(2)
    {
        assert!(
            !get_sim_n_response.entries.is_empty(),
            "Should find similar faces"
        );
        assert!(
            get_sim_n_response.entries.len() <= 10,
            "Should return at most 10 results"
        );

        // The top result should be the same face we queried with
        let top_result = &get_sim_n_response.entries[0];
        let top_similarity = top_result
            .similarity
            .as_ref()
            .expect("Top result should have similarity")
            .value;

        // When querying with the exact same image, similarity should be very high (close to 1.0)
        // Face embeddings from the same image should have cosine similarity > 0.99
        assert!(
            top_similarity > 0.99,
            "Top result should have very high similarity (>0.99) when querying with same face, got: {}",
            top_similarity
        );

        // Verify results are sorted by similarity (descending for cosine similarity)
        for i in 1..get_sim_n_response.entries.len() {
            let prev_sim = get_sim_n_response.entries[i - 1]
                .similarity
                .as_ref()
                .map(|s| s.value)
                .unwrap_or(0.0);
            let curr_sim = get_sim_n_response.entries[i]
                .similarity
                .as_ref()
                .map(|s| s.value)
                .unwrap_or(0.0);
            assert!(
                prev_sim >= curr_sim,
                "Results should be sorted by similarity (descending), but entry {} ({}) < entry {} ({})",
                i - 1,
                prev_sim,
                i,
                curr_sim
            );
        }

        // The second-best results should be from the Friends cast (different faces)
        // and should have noticeably lower similarity
        if get_sim_n_response.entries.len() > 1 {
            let second_similarity = get_sim_n_response.entries[1]
                .similarity
                .as_ref()
                .expect("Second result should have similarity")
                .value;
            assert!(
                second_similarity < top_similarity,
                "Second result should have lower similarity than identical face"
            );
            // Different faces should have significantly lower similarity (typically < 0.6)
            // Allow small margin for edge cases where similarity is right at threshold
            assert!(
                second_similarity <= 0.71,
                "Different faces should have lower similarity (≤0.71), got: {}",
                second_similarity
            );
        }
    } else {
        panic!("Expected GetSimN response, got: {:?}", responses.get(2));
    }
}

#[tokio::test]
async fn test_buffalo_l_get_sim_n_multi_face_query_errors() {
    // Scenario: a GetSimN query is issued with a multi-face image.
    // A query must produce exactly one embedding; multiple faces make it ambiguous.
    // The server must reject this with InvalidArgument rather than silently picking a face.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_multi_face_query_error".to_string();
    let single_face_image = include_bytes!("../../test_data/single_face.jpg").to_vec();
    let multi_face_image = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();

    // Create store and insert a single face so the store is non-empty
    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::BuffaloL.into(),
                    index_model: AiModel::BuffaloL.into(),
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
                            value: Some(Value::Image(single_face_image)),
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

    // Now query with a multi-face image — must return InvalidArgument
    let err = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::Image(multi_face_image)),
            }),
            condition: None,
            closest_n: 3,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        }))
        .await
        .expect_err("Expected error when querying with multi-face image");

    assert_eq!(
        err.code(),
        tonic::Code::InvalidArgument,
        "Multi-face query should return InvalidArgument, got: {:?}",
        err
    );
    assert!(
        err.message().contains("embeddings"),
        "Error message should mention embedding count, got: {}",
        err.message()
    );
}

#[tokio::test]
async fn test_buffalo_l_face_index_metadata() {
    // Scenario: a multi-face image is indexed; each stored face embedding gets an
    // auto-injected `one_to_many_index` metadata field with a sequential integer (0, 1, 2, …).
    // A single-face query is used to retrieve all entries and verify the indices are present
    // and sequential. A multi-face query is intentionally avoided — it would be rejected.
    let address = provision_test_servers().await;
    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(200)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Could not connect");

    let store_name = "FaceIndexTest".to_string();

    // Load image with multiple faces (should detect at least 2 faces)
    let image_bytes =
        std::fs::read("./test_data/faces_multiple.jpg").expect("Could not read test image");
    // Single-face image used as query — querying with a multi-face image is ambiguous
    let query_image_bytes =
        std::fs::read("./test_data/single_face.jpg").expect("Could not read query image");

    // Create store without predicates - Buffalo_L should auto-add face_index predicate
    let create_store = ai_query_types::CreateStore {
        store: store_name.clone(),
        query_model: AiModel::BuffaloL.into(),
        index_model: AiModel::BuffaloL.into(),
        predicates: vec![], // Empty - face_index should be added automatically
        non_linear_indices: vec![],
        error_if_exists: true,
        store_original: false,
    };

    // Store the image
    let store_entry = AiStoreEntry {
        key: Some(StoreInput {
            value: Some(Value::Image(image_bytes.clone())),
        }),
        value: Some(StoreValue {
            value: HashMap::from_iter([(
                "photo_id".to_string(),
                MetadataValue {
                    value: Some(metadata_value::Value::RawString(
                        "group_photo_001".to_string(),
                    )),
                },
            )]),
        }),
    };

    let set_query = ai_query_types::Set {
        store: store_name.clone(),
        inputs: vec![store_entry],
        preprocess_action: PreprocessAction::ModelPreprocessing.into(),
        execution_provider: None,
        model_params: HashMap::new(),
    };

    // Execute pipeline: create store + set data
    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(create_store)),
            },
            ai_pipeline::AiQuery {
                query: Some(Query::Set(set_query)),
            },
        ],
    };

    let pipeline_response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Pipeline failed")
        .into_inner();

    // Verify we got multiple faces stored
    if let Some(ai_pipeline::ai_server_response::Response::Set(set_response)) =
        &pipeline_response.responses[1].response
    {
        let total_upserts = set_response
            .upsert
            .as_ref()
            .map(|u| u.inserted + u.updated)
            .unwrap_or(0);
        assert!(
            total_upserts >= 2,
            "Expected at least 2 faces detected, got: {}",
            total_upserts
        );

        // Now query to get all stored entries
        let get_query = ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::Image(query_image_bytes)),
            }),
            condition: None,
            closest_n: 10, // Get all faces
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        };

        let get_response = client
            .get_sim_n(tonic::Request::new(get_query))
            .await
            .expect("GetSimN failed")
            .into_inner();

        // Verify each entry has one_to_many_index metadata
        let output_indices: Vec<String> = get_response
            .entries
            .iter()
            .filter_map(|entry| {
                entry.value.as_ref().and_then(|v| {
                    v.value
                        .get(crate::AHNLICH_AI_ONE_TO_MANY_INDEX_META_KEY)
                        .and_then(|meta| {
                            if let Some(metadata_value::Value::RawString(idx)) = &meta.value {
                                Some(idx.clone())
                            } else {
                                None
                            }
                        })
                })
            })
            .collect();

        assert!(
            !output_indices.is_empty(),
            "Expected one_to_many_index metadata, but found none"
        );

        // Verify indices are sequential: 0, 1, 2, ...
        let mut expected_indices: Vec<String> =
            (0..output_indices.len()).map(|i| i.to_string()).collect();
        let mut found_indices = output_indices.clone();
        found_indices.sort();
        expected_indices.sort();

        assert_eq!(
            found_indices, expected_indices,
            "OneToMany indices should be sequential from 0"
        );

        println!(
            "✓ Successfully verified {} outputs with indices: {:?}",
            output_indices.len(),
            output_indices
        );
    } else {
        panic!(
            "Expected Set response, got: {:?}",
            pipeline_response.responses.get(1)
        );
    }
}

#[tokio::test]
async fn test_buffalo_l_mixed_batch_no_face_does_not_fail_batch() {
    // Scenario: a batch contains two entries — one image with detectable faces and one
    // with none. The no-face image produces zero embeddings and is silently skipped.
    // The batch as a whole must succeed, storing only the faces from the valid image.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_mixed_batch".to_string();

    let single_face_image = include_bytes!("../../test_data/single_face.jpg").to_vec();
    let no_face_image = include_bytes!("../../test_data/no_face.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::BuffaloL.into(),
                    index_model: AiModel::BuffaloL.into(),
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
                        // First entry: one detectable face — should produce 1 stored embedding.
                        AiStoreEntry {
                            key: Some(StoreInput {
                                value: Some(Value::Image(single_face_image)),
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
                        // Second entry: no detectable faces — produces zero embeddings and is
                        // skipped. The batch must not fail because of this entry.
                        AiStoreEntry {
                            key: Some(StoreInput {
                                value: Some(Value::Image(no_face_image)),
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
async fn test_buffalo_l_high_confidence_threshold() {
    // Scenario: Test that model_params confidence_threshold works correctly.
    // With a very high threshold (0.89), fewer faces should be detected compared to default (0.5).
    // The Friends cast image has 6 faces at default threshold, but with 0.89 we expect fewer.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_high_threshold_store".to_string();
    let image_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();

    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::BuffaloL.into(),
            index_model: AiModel::BuffaloL.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })),
    };

    // First, insert with default threshold (0.5) to establish baseline
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
            model_params: HashMap::new(), // Empty = use default threshold (0.5)
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
        panic!(
            "Expected Set response at index 1, got: {:?}",
            responses.get(1)
        );
    };

    // Now test with high threshold (0.89)
    // Use a different store name to avoid any timing issues with drop/recreate
    let high_threshold_store_name = format!("{}_high", store_name);

    let recreate_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: high_threshold_store_name.clone(),
            query_model: AiModel::BuffaloL.into(),
            index_model: AiModel::BuffaloL.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: false,
        })),
    };

    let mut high_threshold_params = HashMap::new();
    high_threshold_params.insert("confidence_threshold".to_string(), "0.89".to_string());

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
        "Default threshold (0.5) detected {} faces, high threshold (0.89) detected {} faces",
        default_face_count, high_threshold_face_count
    );

    // Default threshold (0.5) with default NMS (0.4) may detect 6-7 faces depending on NMS behavior
    assert!(
        default_face_count >= 6,
        "Default threshold should detect at least 6 faces, got {}",
        default_face_count
    );
    assert!(
        high_threshold_face_count < default_face_count,
        "High confidence threshold (0.89) should detect fewer faces than default (0.5). \
         Got {} faces with 0.89 vs {} with 0.5",
        high_threshold_face_count,
        default_face_count
    );
    assert!(
        high_threshold_face_count > 0,
        "Even with high threshold, at least one clear face should be detected"
    );
}

#[tokio::test]
async fn test_buffalo_l_bounding_box_metadata() {
    // Scenario: Verify that Buffalo_L returns normalized bounding box metadata
    // for each detected face. Bounding boxes should be in 0-1 range (normalized).
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_bbox_test".to_string();
    let image_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();
    let query_image_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::BuffaloL.into(),
                    index_model: AiModel::BuffaloL.into(),
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

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    // Query to retrieve the stored faces with metadata
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::Image(query_image_bytes)),
            }),
            condition: None,
            closest_n: 10,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    assert!(
        !get_response.entries.is_empty(),
        "Should have at least one stored face"
    );

    // Verify bounding box metadata for each face
    let mut found_bbox_count = 0;
    for entry in &get_response.entries {
        if let Some(value) = &entry.value {
            // Check for bounding box coordinates
            if let (Some(bbox_x1), Some(bbox_y1), Some(bbox_x2), Some(bbox_y2), Some(confidence)) = (
                value.value.get("bbox_x1"),
                value.value.get("bbox_y1"),
                value.value.get("bbox_x2"),
                value.value.get("bbox_y2"),
                value.value.get("confidence"),
            ) {
                found_bbox_count += 1;

                // Extract and verify coordinates are normalized (0-1 range)
                let x1 = extract_float_from_metadata(bbox_x1);
                let y1 = extract_float_from_metadata(bbox_y1);
                let x2 = extract_float_from_metadata(bbox_x2);
                let y2 = extract_float_from_metadata(bbox_y2);

                assert!(
                    (0.0..=1.0).contains(&x1),
                    "bbox_x1 should be normalized (0-1), got: {}",
                    x1
                );
                assert!(
                    (0.0..=1.0).contains(&y1),
                    "bbox_y1 should be normalized (0-1), got: {}",
                    y1
                );
                assert!(
                    (0.0..=1.0).contains(&x2),
                    "bbox_x2 should be normalized (0-1), got: {}",
                    x2
                );
                assert!(
                    (0.0..=1.0).contains(&y2),
                    "bbox_y2 should be normalized (0-1), got: {}",
                    y2
                );

                // Verify bounding box makes sense (x2 > x1, y2 > y1)
                assert!(x2 > x1, "x2 ({}) should be greater than x1 ({})", x2, x1);
                assert!(y2 > y1, "y2 ({}) should be greater than y1 ({})", y2, y1);

                // Verify confidence exists and is in valid range
                let conf = extract_float_from_metadata(confidence);
                assert!(
                    (0.0..=1.0).contains(&conf),
                    "Confidence should be in range [0,1], got: {}",
                    conf
                );

                println!(
                    "✓ Face bbox: [{:.3}, {:.3}, {:.3}, {:.3}], confidence: {:.3}",
                    x1, y1, x2, y2, conf
                );
            }
        }
    }

    assert_eq!(
        found_bbox_count,
        get_response.entries.len(),
        "All faces should have bounding box metadata"
    );
    println!(
        "✓ Successfully verified {} faces with normalized bounding box metadata",
        found_bbox_count
    );
}

// Helper function to extract f32 from MetadataValue
fn extract_float_from_metadata(metadata: &MetadataValue) -> f32 {
    if let Some(metadata_value::Value::RawString(s)) = &metadata.value {
        s.parse::<f32>()
            .unwrap_or_else(|_| panic!("Failed to parse float from: {}", s))
    } else {
        panic!("Expected RawString metadata value");
    }
}

fn extract_i32_from_metadata(metadata: &MetadataValue) -> i32 {
    if let Some(metadata_value::Value::RawString(s)) = &metadata.value {
        s.parse::<i32>()
            .unwrap_or_else(|_| panic!("Failed to parse i32 from: {}", s))
    } else {
        panic!("Expected RawString metadata value");
    }
}

#[tokio::test]
async fn test_buffalo_l_gender_age_metadata() {
    // Scenario: Verify that Buffalo_L returns gender/age attributes metadata
    // for each detected face with valid format (probabilities in [0,1], sum to 1.0, etc.).
    // Uses single_face.jpg (Jennifer Aniston) as test input.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_gender_age_test".to_string();
    let image_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::BuffaloL.into(),
                    index_model: AiModel::BuffaloL.into(),
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
                            value: Some(Value::Image(image_bytes.clone())),
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    }],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: genderage_params(),
                })),
            },
        ],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    // Query to retrieve the stored faces with metadata
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::Image(image_bytes)),
            }),
            condition: None,
            closest_n: 10,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: genderage_params(),
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    assert!(
        !get_response.entries.is_empty(),
        "Should have at least one stored face"
    );

    // Verify gender/age metadata for each face
    let mut found_attrs_count = 0;
    for entry in &get_response.entries {
        if let Some(value) = &entry.value {
            // Check for gender/age attributes
            if let (Some(gender_female), Some(gender_male), Some(age)) = (
                value.value.get("gender_female_prob"),
                value.value.get("gender_male_prob"),
                value.value.get("age"),
            ) {
                found_attrs_count += 1;

                // Extract and verify gender probabilities
                let female_prob = extract_float_from_metadata(gender_female);
                let male_prob = extract_float_from_metadata(gender_male);

                assert!(
                    (0.0..=1.0).contains(&female_prob),
                    "gender_female_prob should be in range [0,1], got: {}",
                    female_prob
                );
                assert!(
                    (0.0..=1.0).contains(&male_prob),
                    "gender_male_prob should be in range [0,1], got: {}",
                    male_prob
                );

                // Probabilities should sum to ~1.0 (allow small float error)
                let sum = female_prob + male_prob;
                assert!(
                    (sum - 1.0).abs() < 0.01,
                    "Gender probabilities should sum to 1.0, got: {}",
                    sum
                );

                // Sanity check: At least one gender should be > 0.3 (not completely uncertain)
                assert!(
                    female_prob > 0.3 || male_prob > 0.3,
                    "Model should have some gender confidence, got F:{} M:{}",
                    female_prob,
                    male_prob
                );

                // Extract and verify age
                let age_val = extract_i32_from_metadata(age);
                assert!(
                    (0..=120).contains(&age_val),
                    "Age should be reasonable, got: {}",
                    age_val
                );

                // Sanity check: Age should be realistic for a typical photo
                assert!(
                    (5..=100).contains(&age_val),
                    "Age should be realistic for a human face photo, got: {}",
                    age_val
                );

                println!(
                    "✓ Face attributes: gender=(F:{:.3}, M:{:.3}), age={}",
                    female_prob, male_prob, age_val
                );
            }
        }
    }

    assert_eq!(
        found_attrs_count,
        get_response.entries.len(),
        "All faces should have gender/age attribute metadata"
    );
    println!(
        "✓ Successfully verified {} faces with gender/age attributes",
        found_attrs_count
    );
}

#[tokio::test]
async fn test_buffalo_l_gender_age_multi_face() {
    // Semantic validation test: Verifies actual gender/age predictions on known test image.
    // Uses faces_multiple.jpg (Friends cast: 3M, 3F expected, model achieves ~83% accuracy)
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_multi_face_test".to_string();
    let image_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();
    let query_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::BuffaloL.into(),
                    index_model: AiModel::BuffaloL.into(),
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
                            value: HashMap::new(),
                        }),
                    }],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: genderage_params(),
                })),
            },
        ],
    };

    client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    // Query to retrieve all stored faces with metadata
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::Image(query_bytes)),
            }),
            condition: None,
            closest_n: 10,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: genderage_params(),
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    // Should detect 6-7 faces (sometimes detects duplicate)
    assert!(
        get_response.entries.len() >= 6,
        "Should detect at least 6 faces, got {}",
        get_response.entries.len()
    );

    println!("\nBuffaloL Gender/Age Predictions:");
    println!("================================");

    let mut male_count = 0;
    let mut female_count = 0;
    let mut ages = Vec::new();

    for (idx, entry) in get_response.entries.iter().enumerate() {
        if let Some(value) = &entry.value
            && let (Some(gender_female), Some(gender_male), Some(age)) = (
                value.value.get("gender_female_prob"),
                value.value.get("gender_male_prob"),
                value.value.get("age"),
            )
        {
            let female_prob = extract_float_from_metadata(gender_female);
            let male_prob = extract_float_from_metadata(gender_male);
            let age_val = extract_i32_from_metadata(age);

            // Verify probabilities are valid and sum to ~1.0
            assert!(
                (0.0..=1.0).contains(&female_prob),
                "Female probability should be in [0,1], got {}",
                female_prob
            );
            assert!(
                (0.0..=1.0).contains(&male_prob),
                "Male probability should be in [0,1], got {}",
                male_prob
            );
            let prob_sum = female_prob + male_prob;
            assert!(
                (prob_sum - 1.0).abs() < 0.01,
                "Gender probabilities should sum to ~1.0, got {}",
                prob_sum
            );

            // Verify age is reasonable for adults
            assert!(
                (18..=100).contains(&age_val),
                "Age should be reasonable (18-100), got {}",
                age_val
            );

            ages.push(age_val);

            let predicted_gender = if female_prob > male_prob {
                female_count += 1;
                "Female"
            } else {
                male_count += 1;
                "Male"
            };
            let confidence = if female_prob > male_prob {
                female_prob
            } else {
                male_prob
            };

            println!(
                "Face {}: {} ({:.1}% confident), Age: {}",
                idx + 1,
                predicted_gender,
                confidence * 100.0,
                age_val
            );
        }
    }

    // Verify gender distribution: Friends cast has 3M, 3F
    // Model achieves ~83% accuracy, so we expect at least 2 of each gender
    assert!(
        male_count >= 2,
        "Should predict at least 2 males (got {}), actual distribution: {}M, {}F",
        male_count,
        male_count,
        female_count
    );
    assert!(
        female_count >= 2,
        "Should predict at least 2 females (got {}), actual distribution: {}M, {}F",
        female_count,
        male_count,
        female_count
    );

    // Verify we got gender predictions for all detected faces
    assert_eq!(
        male_count + female_count,
        get_response.entries.len(),
        "All faces should have gender predictions"
    );

    println!(
        "\n✓ Gender distribution: {} males, {} females",
        male_count, female_count
    );
    println!(
        "✓ Age range: {}-{}",
        ages.iter().min().unwrap(),
        ages.iter().max().unwrap()
    );
}

#[tokio::test]
#[ignore] // Run manually with: cargo test test_buffalo_l_visualize_attributes -- --ignored --nocapture
async fn test_buffalo_l_visualize_attributes() {
    // Visual verification test: Draws bboxes, keypoints, and gender/age on test image
    // Output saved to: /tmp/buffalo_l_attributes_debug.jpg
    use image::Rgba;

    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_visualize".to_string();
    let image_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();
    let query_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();

    // Load original image for drawing
    let img = image::load_from_memory(&image_bytes).expect("Failed to load image");
    let mut output_img = img.to_rgba8();
    let (img_width, img_height) = output_img.dimensions();
    eprintln!(
        "DEBUG: Original image dimensions: {}x{}",
        img_width, img_height
    );

    // Index the image
    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::BuffaloL.into(),
                    index_model: AiModel::BuffaloL.into(),
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
                            value: Some(Value::Image(image_bytes.clone())),
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
        .expect("Failed to execute pipeline");

    // Retrieve all stored faces with metadata
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::Image(query_bytes)),
            }),
            condition: None,
            closest_n: 20,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    println!("Found {} faces", get_response.entries.len());

    // Draw each face
    for (idx, entry) in get_response.entries.iter().enumerate() {
        if let Some(value) = &entry.value {
            // Extract gender/age
            let female_prob = value
                .value
                .get("gender_female_prob")
                .and_then(|v| {
                    extract_float_from_metadata(v)
                        .to_string()
                        .parse::<f32>()
                        .ok()
                })
                .unwrap_or(0.5);
            let male_prob = value
                .value
                .get("gender_male_prob")
                .and_then(|v| {
                    extract_float_from_metadata(v)
                        .to_string()
                        .parse::<f32>()
                        .ok()
                })
                .unwrap_or(0.5);
            let age = value
                .value
                .get("age")
                .and_then(|v| extract_i32_from_metadata(v).to_string().parse::<i32>().ok())
                .unwrap_or(0);

            let gender_label = if female_prob > male_prob {
                format!("F({:.0}%)", female_prob * 100.0)
            } else {
                format!("M({:.0}%)", male_prob * 100.0)
            };

            // Extract bbox
            if let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
                value.value.get("bbox_x1"),
                value.value.get("bbox_y1"),
                value.value.get("bbox_x2"),
                value.value.get("bbox_y2"),
            ) {
                let x1_norm = extract_float_from_metadata(x1);
                let y1_norm = extract_float_from_metadata(y1);
                let x2_norm = extract_float_from_metadata(x2);
                let y2_norm = extract_float_from_metadata(y2);

                // Convert normalized coords to pixels
                let x1_px = (x1_norm * img_width as f32) as i32;
                let y1_px = (y1_norm * img_height as f32) as i32;
                let x2_px = (x2_norm * img_width as f32) as i32;
                let y2_px = (y2_norm * img_height as f32) as i32;

                // Print bbox coordinates for manual verification
                println!(
                    "Face {}: {} Age:{} bbox=({},{}) to ({},{})",
                    idx, gender_label, age, x1_px, y1_px, x2_px, y2_px
                );

                // Draw thick bbox (3 pixels wide) in cyan
                let cyan = Rgba([0, 255, 255, 255]);
                for thickness in 0..3 {
                    // Top
                    for x in (x1_px - thickness)..(x2_px + thickness) {
                        if x >= 0
                            && x < img_width as i32
                            && (y1_px - thickness) >= 0
                            && (y1_px - thickness) < img_height as i32
                        {
                            output_img.put_pixel(x as u32, (y1_px - thickness) as u32, cyan);
                        }
                    }
                    // Bottom
                    for x in (x1_px - thickness)..(x2_px + thickness) {
                        if x >= 0
                            && x < img_width as i32
                            && (y2_px + thickness) >= 0
                            && (y2_px + thickness) < img_height as i32
                        {
                            output_img.put_pixel(x as u32, (y2_px + thickness) as u32, cyan);
                        }
                    }
                    // Left
                    for y in (y1_px - thickness)..(y2_px + thickness) {
                        if (x1_px - thickness) >= 0
                            && (x1_px - thickness) < img_width as i32
                            && y >= 0
                            && y < img_height as i32
                        {
                            output_img.put_pixel((x1_px - thickness) as u32, y as u32, cyan);
                        }
                    }
                    // Right
                    for y in (y1_px - thickness)..(y2_px + thickness) {
                        if (x2_px + thickness) >= 0
                            && (x2_px + thickness) < img_width as i32
                            && y >= 0
                            && y < img_height as i32
                        {
                            output_img.put_pixel((x2_px + thickness) as u32, y as u32, cyan);
                        }
                    }
                }

                // Draw simple text label (manually rendered)
                // We'll draw a background box and put the label in the console instead
                let _label = format!("#{}", idx);
                let label_bg_x = x1_px.max(0);
                let label_bg_y = (y1_px - 25).max(0);

                // Draw black background rectangle (20x20 pixels)
                for ty in label_bg_y..(label_bg_y + 20).min(img_height as i32) {
                    for tx in label_bg_x..(label_bg_x + 40).min(img_width as i32) {
                        output_img.put_pixel(tx as u32, ty as u32, Rgba([0, 0, 0, 200]));
                    }
                }

                // Draw white text "#{idx}" using simple pixel art (just the number)
                // For simplicity, we'll just put a colored square with the index
                let index_color = match idx % 7 {
                    0 => Rgba([255, 0, 0, 255]),     // Red
                    1 => Rgba([0, 255, 0, 255]),     // Green
                    2 => Rgba([0, 0, 255, 255]),     // Blue
                    3 => Rgba([255, 255, 0, 255]),   // Yellow
                    4 => Rgba([255, 0, 255, 255]),   // Magenta
                    5 => Rgba([0, 255, 255, 255]),   // Cyan
                    _ => Rgba([255, 255, 255, 255]), // White
                };

                for ty in (label_bg_y + 2)..(label_bg_y + 18).min(img_height as i32) {
                    for tx in (label_bg_x + 2)..(label_bg_x + 18).min(img_width as i32) {
                        output_img.put_pixel(tx as u32, ty as u32, index_color);
                    }
                }
            }
        }
    }

    // Save annotated image
    output_img
        .save("/tmp/buffalo_l_attributes_debug.png")
        .expect("Failed to save image");
    println!("✓ Saved annotated image to /tmp/buffalo_l_attributes_debug.png");
    println!("  Review the image to verify:");
    println!("  - Green boxes around faces");
    println!("  - Gender and age printed in console above");
}

#[tokio::test]
async fn test_buffalo_l_without_genderage() {
    // Scenario: Verify default behavior when genderage attributes are NOT requested.
    // Should still detect faces with bbox/confidence but NO gender/age metadata.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_no_genderage".to_string();
    let image_bytes = include_bytes!("../../test_data/faces_multiple.jpg").to_vec();
    let query_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::BuffaloL.into(),
                    index_model: AiModel::BuffaloL.into(),
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
                            value: HashMap::new(),
                        }),
                    }],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: optimized_face_params(), // No attributes=genderage
                })),
            },
        ],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    // Query to retrieve the stored faces with metadata
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::Image(query_bytes)),
            }),
            condition: None,
            closest_n: 10,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: optimized_face_params(), // No attributes=genderage
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    assert!(
        !get_response.entries.is_empty(),
        "Should detect at least one face"
    );

    // Verify first face has bbox/confidence but NO gender/age
    let first_entry = &get_response.entries[0];
    let metadata = first_entry
        .value
        .as_ref()
        .expect("Expected value")
        .value
        .clone();

    // Verify bbox and confidence ARE present
    assert!(
        metadata.contains_key("bbox_x1"),
        "Should have bbox_x1 without genderage"
    );
    assert!(
        metadata.contains_key("bbox_y1"),
        "Should have bbox_y1 without genderage"
    );
    assert!(
        metadata.contains_key("bbox_x2"),
        "Should have bbox_x2 without genderage"
    );
    assert!(
        metadata.contains_key("bbox_y2"),
        "Should have bbox_y2 without genderage"
    );
    assert!(
        metadata.contains_key("confidence"),
        "Should have confidence without genderage"
    );

    // Verify gender/age fields are NOT present
    assert!(
        !metadata.contains_key("gender_female_prob"),
        "Should NOT have gender_female_prob without attributes=genderage"
    );
    assert!(
        !metadata.contains_key("gender_male_prob"),
        "Should NOT have gender_male_prob without attributes=genderage"
    );
    assert!(
        !metadata.contains_key("age"),
        "Should NOT have age without attributes=genderage"
    );

    println!(
        "✓ Successfully verified face detection without genderage: {} faces detected",
        get_response.entries.len()
    );
    println!("✓ Metadata includes bbox/confidence but NOT gender/age fields");
}

#[tokio::test]
async fn test_buffalo_l_genderage_opt_in() {
    // Scenario: Verify explicit opt-in behavior when genderage attributes ARE requested.
    // With attributes=genderage, should detect faces with ALL metadata: bbox, confidence, gender, age.
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "buffalo_l_with_genderage".to_string();
    let image_bytes = include_bytes!("../../test_data/single_face.jpg").to_vec();

    let pipeline_request = ai_pipeline::AiRequestPipeline {
        queries: vec![
            ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store_name.clone(),
                    query_model: AiModel::BuffaloL.into(),
                    index_model: AiModel::BuffaloL.into(),
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
                            value: Some(Value::Image(image_bytes.clone())),
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    }],
                    preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                    execution_provider: None,
                    model_params: genderage_params(), // WITH attributes=genderage
                })),
            },
        ],
    };

    let response = client
        .pipeline(tonic::Request::new(pipeline_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    // Query to retrieve the stored faces with metadata
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::Image(image_bytes)),
            }),
            condition: None,
            closest_n: 10,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: genderage_params(), // WITH attributes=genderage
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    assert!(
        !get_response.entries.is_empty(),
        "Should detect at least one face"
    );

    // Verify first face has ALL metadata fields (bbox + confidence + gender + age)
    let first_entry = &get_response.entries[0];
    let metadata = first_entry
        .value
        .as_ref()
        .expect("Expected value")
        .value
        .clone();

    // Verify bbox and confidence ARE present
    assert!(metadata.contains_key("bbox_x1"), "Should have bbox_x1");
    assert!(metadata.contains_key("bbox_y1"), "Should have bbox_y1");
    assert!(metadata.contains_key("bbox_x2"), "Should have bbox_x2");
    assert!(metadata.contains_key("bbox_y2"), "Should have bbox_y2");
    assert!(
        metadata.contains_key("confidence"),
        "Should have confidence"
    );

    // Verify gender/age fields ARE present with attributes=genderage
    assert!(
        metadata.contains_key("gender_female_prob"),
        "Should have gender_female_prob with attributes=genderage"
    );
    assert!(
        metadata.contains_key("gender_male_prob"),
        "Should have gender_male_prob with attributes=genderage"
    );
    assert!(
        metadata.contains_key("age"),
        "Should have age with attributes=genderage"
    );

    // Verify values are in reasonable ranges
    let female_prob = extract_float_from_metadata(&metadata["gender_female_prob"]);
    let male_prob = extract_float_from_metadata(&metadata["gender_male_prob"]);
    let age = extract_i32_from_metadata(&metadata["age"]);

    assert!(
        (0.0..=1.0).contains(&female_prob),
        "Female probability should be 0-1, got {}",
        female_prob
    );
    assert!(
        (0.0..=1.0).contains(&male_prob),
        "Male probability should be 0-1, got {}",
        male_prob
    );
    assert!(
        (0.0..=120.0).contains(&(age as f32)),
        "Age should be 0-120, got {}",
        age
    );

    // Probabilities should sum to ~1.0
    let prob_sum = female_prob + male_prob;
    assert!(
        (0.99..=1.01).contains(&prob_sum),
        "Gender probabilities should sum to ~1.0, got {}",
        prob_sum
    );

    println!(
        "✓ Successfully verified face detection WITH genderage: {} faces detected",
        get_response.entries.len()
    );
    println!("✓ Metadata includes bbox, confidence, AND gender/age fields");
    println!(
        "✓ Values: female={:.3}, male={:.3}, age={}",
        female_prob, male_prob, age
    );
}
