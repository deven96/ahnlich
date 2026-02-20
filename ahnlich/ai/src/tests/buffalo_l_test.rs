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
        // 3 identical images with 6 faces each = 6 unique + 12 updates
        assert_eq!(
            upsert.inserted, 6,
            "Expected 6 unique faces from 3 identical images"
        );
        assert_eq!(
            upsert.updated, 12,
            "Expected 12 updates from duplicate faces (6 faces × 2 extra copies)"
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
            model_params: HashMap::new(),
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
            model_params: HashMap::new(),
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
        // single_face.jpg has 1 face, faces_multiple.jpg has 6 faces = 7 total
        assert_eq!(upsert.inserted, 7, "Should insert 1 + 6 = 7 faces");
    }

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::GetSimN(get_sim_n_response)),
    }) = responses.get(2)
    {
        assert!(
            get_sim_n_response.entries.len() > 0,
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
            assert!(
                second_similarity < 0.7,
                "Different faces should have lower similarity (<0.7), got: {}",
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

    assert_eq!(
        default_face_count, 6,
        "Default threshold should detect all 6 faces"
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
