//! Integration tests for CLAP audio and text embeddings.
//!
//! Audio files are sourced from Wikimedia Commons (CC BY-SA):
//!   - dog_bark.ogg: "Barking of a dog" by Yannickteejay
//!   - cat_meow.ogg: "Meow" by Mramoeba
//!   - rain.ogg:     "Rain" by Bertof

use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;
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
        server::GetSimNEntry,
    },
    algorithm::algorithms::Algorithm,
    keyval::{AiStoreEntry, StoreInput, StoreValue, store_input::Value},
    metadata::{MetadataValue, metadata_value},
    services::ai_service::ai_service_client::AiServiceClient,
};

async fn provision_clap_servers() -> SocketAddr {
    let db_config = ServerConfig::default().os_select_port();
    let server = Server::new(&db_config)
        .await
        .expect("Failed to create DB server");
    let db_port = server.local_addr().unwrap().port();
    tokio::spawn(async move { server.start().await });

    let mut ai_config = AIProxyConfig::default()
        .os_select_port()
        .set_supported_models(vec![SupportedModels::ClapAudio, SupportedModels::ClapText]);
    ai_config.db_port = db_port;

    let ai_server = AIProxyServer::new(ai_config)
        .await
        .expect("Could not initialize AI proxy");
    let ai_address = ai_server.local_addr().expect("Could not get local addr");
    tokio::spawn(async move { ai_server.start().await });

    tokio::time::sleep(Duration::from_millis(200)).await;
    ai_address
}

async fn connect(addr: SocketAddr) -> AiServiceClient<Channel> {
    let channel = Channel::from_shared(format!("http://{addr}")).expect("Failed to create channel");
    AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect")
}

fn raw_string_entry(text: &str, label: &str) -> AiStoreEntry {
    AiStoreEntry {
        key: Some(StoreInput {
            value: Some(Value::RawString(text.to_string())),
        }),
        value: Some(label_value(label)),
    }
}

fn audio_entry(bytes: Vec<u8>, label: &str) -> AiStoreEntry {
    AiStoreEntry {
        key: Some(StoreInput {
            value: Some(Value::Audio(bytes)),
        }),
        value: Some(label_value(label)),
    }
}

fn label_value(label: &str) -> StoreValue {
    StoreValue {
        value: HashMap::from([(
            "label".to_string(),
            MetadataValue {
                value: Some(metadata_value::Value::RawString(label.to_string())),
            },
        )]),
    }
}

fn extract_label(entry: &GetSimNEntry) -> &str {
    entry
        .value
        .as_ref()
        .and_then(|v| v.value.get("label"))
        .and_then(|m| m.value.as_ref())
        .and_then(|v| {
            if let metadata_value::Value::RawString(s) = v {
                Some(s.as_str())
            } else {
                None
            }
        })
        .unwrap_or("unknown")
}

/// Index audio with ClapAudio, query with ClapText. Asserts each text description
/// returns the semantically correct audio clip as the top result.
#[tokio::test]
async fn test_clap_cross_modal_audio_indexed_text_queried() {
    let addr = provision_clap_servers().await;
    let mut client = connect(addr).await;
    let store = "clap_cross_modal_store".to_string();

    let dog_bytes = include_bytes!("../../test_data/audio/dog_bark.ogg").to_vec();
    let cat_bytes = include_bytes!("../../test_data/audio/cat_meow.ogg").to_vec();
    let rain_bytes = include_bytes!("../../test_data/audio/rain.ogg").to_vec();

    let pipeline = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store.clone(),
                query_model: AiModel::ClapText.into(),
                index_model: AiModel::ClapAudio.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: false,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store.clone(),
                inputs: vec![
                    audio_entry(dog_bytes, "dog"),
                    audio_entry(cat_bytes, "cat"),
                    audio_entry(rain_bytes, "rain"),
                ],
                preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    let resp = client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: pipeline,
        }))
        .await
        .expect("pipeline failed")
        .into_inner()
        .responses;

    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_resp)),
    }) = resp.get(1)
    {
        assert_eq!(set_resp.upsert.as_ref().unwrap().inserted, 3);
    } else {
        panic!("Expected Set response at index 1");
    }

    for (query_text, expected_label) in [
        ("a dog barking loudly", "dog"),
        ("a cat meowing", "cat"),
        ("heavy rain falling", "rain"),
    ] {
        let entries = client
            .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
                store: store.clone(),
                search_input: Some(StoreInput {
                    value: Some(Value::RawString(query_text.into())),
                }),
                closest_n: 1,
                algorithm: Algorithm::CosineSimilarity.into(),
                condition: None,
                preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                execution_provider: None,
            }))
            .await
            .expect("GetSimN failed")
            .into_inner()
            .entries;

        assert_eq!(entries.len(), 1, "query '{query_text}'");
        assert_eq!(
            extract_label(&entries[0]),
            expected_label,
            "query '{query_text}'"
        );
    }
}

/// Index audio with ClapAudio, query with ClapAudio. Asserts each audio clip is
/// most similar to itself (identity retrieval).
#[tokio::test]
async fn test_clap_audio_to_audio_retrieval() {
    let addr = provision_clap_servers().await;
    let mut client = connect(addr).await;
    let store = "clap_audio_audio_store".to_string();

    let dog_bytes = include_bytes!("../../test_data/audio/dog_bark.ogg").to_vec();
    let cat_bytes = include_bytes!("../../test_data/audio/cat_meow.ogg").to_vec();
    let rain_bytes = include_bytes!("../../test_data/audio/rain.ogg").to_vec();

    let pipeline = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store.clone(),
                query_model: AiModel::ClapAudio.into(),
                index_model: AiModel::ClapAudio.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: false,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store.clone(),
                inputs: vec![
                    audio_entry(dog_bytes.clone(), "dog"),
                    audio_entry(cat_bytes.clone(), "cat"),
                    audio_entry(rain_bytes.clone(), "rain"),
                ],
                preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: pipeline,
        }))
        .await
        .expect("pipeline failed");

    for (query_bytes, expected_label) in
        [(dog_bytes, "dog"), (cat_bytes, "cat"), (rain_bytes, "rain")]
    {
        let entries = client
            .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
                store: store.clone(),
                search_input: Some(StoreInput {
                    value: Some(Value::Audio(query_bytes)),
                }),
                closest_n: 1,
                algorithm: Algorithm::CosineSimilarity.into(),
                condition: None,
                preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                execution_provider: None,
            }))
            .await
            .expect("GetSimN failed")
            .into_inner()
            .entries;

        assert_eq!(entries.len(), 1, "label '{expected_label}'");
        assert_eq!(
            extract_label(&entries[0]),
            expected_label,
            "audio query for '{expected_label}' should return itself"
        );
    }
}

/// Index text with ClapText, query with ClapText. Asserts each description is
/// most similar to itself (identity retrieval).
#[tokio::test]
async fn test_clap_text_to_text_retrieval() {
    let addr = provision_clap_servers().await;
    let mut client = connect(addr).await;
    let store = "clap_text_text_store".to_string();

    let pipeline = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store.clone(),
                query_model: AiModel::ClapText.into(),
                index_model: AiModel::ClapText.into(),
                predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                store_original: false,
            })),
        },
        ai_pipeline::AiQuery {
            query: Some(Query::Set(ai_query_types::Set {
                store: store.clone(),
                inputs: vec![
                    raw_string_entry("a dog barking", "dog"),
                    raw_string_entry("a cat meowing", "cat"),
                    raw_string_entry("heavy rain falling", "rain"),
                ],
                preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: pipeline,
        }))
        .await
        .expect("pipeline failed");

    for (query_text, expected_label) in [
        ("a dog barking", "dog"),
        ("a cat meowing", "cat"),
        ("heavy rain falling", "rain"),
    ] {
        let entries = client
            .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
                store: store.clone(),
                search_input: Some(StoreInput {
                    value: Some(Value::RawString(query_text.into())),
                }),
                closest_n: 1,
                algorithm: Algorithm::CosineSimilarity.into(),
                condition: None,
                preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                execution_provider: None,
            }))
            .await
            .expect("GetSimN failed")
            .into_inner()
            .entries;

        assert_eq!(entries.len(), 1, "query '{query_text}'");
        assert_eq!(
            extract_label(&entries[0]),
            expected_label,
            "query '{query_text}' should return '{expected_label}'"
        );
    }
}

// ---------------------------------------------------------------------------
// Helper: build a minimal valid WAV file with silence at 48 kHz mono.
// The WAV header is exactly 44 bytes; samples are 16-bit PCM zeros.
// ---------------------------------------------------------------------------
fn make_silent_wav(duration_secs: f32) -> Vec<u8> {
    let sample_rate: u32 = 48_000;
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let num_samples = (sample_rate as f32 * duration_secs).ceil() as u32;
    let data_size = num_samples * u32::from(bits_per_sample / 8);
    let file_size = 36 + data_size; // RIFF chunk size

    let mut buf = Vec::with_capacity(44 + data_size as usize);
    // RIFF header
    buf.extend_from_slice(b"RIFF");
    buf.extend_from_slice(&file_size.to_le_bytes());
    buf.extend_from_slice(b"WAVE");
    // fmt sub-chunk
    buf.extend_from_slice(b"fmt ");
    buf.extend_from_slice(&16u32.to_le_bytes()); // sub-chunk size
    buf.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    buf.extend_from_slice(&num_channels.to_le_bytes());
    buf.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample / 8);
    buf.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = num_channels * (bits_per_sample / 8);
    buf.extend_from_slice(&block_align.to_le_bytes());
    buf.extend_from_slice(&bits_per_sample.to_le_bytes());
    // data sub-chunk
    buf.extend_from_slice(b"data");
    buf.extend_from_slice(&data_size.to_le_bytes());
    buf.extend(std::iter::repeat_n(0u8, data_size as usize)); // silence
    buf
}

/// NoPreprocessing is rejected immediately for ClapAudio — the caller cannot
/// supply a mel spectrogram over the wire; raw bytes always require the pipeline.
#[tokio::test]
async fn test_clap_audio_no_preprocessing_rejected() {
    let ai_address = provision_clap_servers().await;
    let mut client = connect(ai_address).await;

    let store = "clap_no_preprocess_store".to_string();
    let dog_bytes = include_bytes!("../../test_data/audio/dog_bark.ogg").to_vec();

    // Create the store first
    client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: vec![ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store.clone(),
                    query_model: AiModel::ClapAudio.into(),
                    index_model: AiModel::ClapAudio.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: false,
                    store_original: false,
                })),
            }],
        }))
        .await
        .expect("pipeline failed");

    let result = client
        .set(tonic::Request::new(ai_query_types::Set {
            store: store.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Audio(dog_bytes)),
                }),
                value: None,
            }],
            preprocess_action: PreprocessAction::NoPreprocessing.into(),
            execution_provider: None,
        }))
        .await;

    assert!(
        result.is_err(),
        "Expected error for NoPreprocessing on audio"
    );
    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::InvalidArgument,
        "Expected InvalidArgument, got {:?}: {}",
        status.code(),
        status.message()
    );
    assert!(
        status
            .message()
            .contains("NoPreprocessing is not supported for audio"),
        "Unexpected error message: {}",
        status.message()
    );
}

/// Audio clips longer than 10 seconds are rejected with a clear error so callers
/// know to trim before indexing rather than getting a silently truncated embedding.
#[tokio::test]
async fn test_clap_audio_too_long_rejected() {
    let ai_address = provision_clap_servers().await;
    let mut client = connect(ai_address).await;

    let store = "clap_too_long_store".to_string();

    // Build a silent WAV that is 11 seconds — just over the 10s limit.
    let long_audio = make_silent_wav(11.0);

    client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: vec![ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store.clone(),
                    query_model: AiModel::ClapAudio.into(),
                    index_model: AiModel::ClapAudio.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: false,
                    store_original: false,
                })),
            }],
        }))
        .await
        .expect("pipeline failed");

    let result = client
        .set(tonic::Request::new(ai_query_types::Set {
            store: store.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Audio(long_audio)),
                }),
                value: None,
            }],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
        }))
        .await;

    assert!(result.is_err(), "Expected error for audio > 10s");
    let status = result.unwrap_err();
    assert_eq!(
        status.code(),
        tonic::Code::InvalidArgument,
        "Expected InvalidArgument, got {:?}: {}",
        status.code(),
        status.message()
    );
    assert!(
        status.message().contains("Audio input is too long"),
        "Unexpected error message: {}",
        status.message()
    );
    assert!(
        status.message().contains("10000ms"),
        "Error message should state the 10000ms limit, got: {}",
        status.message()
    );
}

/// Audio clips shorter than 10 seconds should still work — they are padded via
/// the repeatpad strategy rather than zero-padded.
#[tokio::test]
async fn test_clap_short_audio_accepted() {
    let ai_address = provision_clap_servers().await;
    let mut client = connect(ai_address).await;

    let store = "clap_short_audio_store".to_string();

    // 3-second silent WAV — well under the 10s limit.
    let short_audio = make_silent_wav(3.0);

    let result = client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: vec![
                ai_pipeline::AiQuery {
                    query: Some(Query::CreateStore(ai_query_types::CreateStore {
                        store: store.clone(),
                        query_model: AiModel::ClapAudio.into(),
                        index_model: AiModel::ClapAudio.into(),
                        predicates: vec![],
                        non_linear_indices: vec![],
                        error_if_exists: false,
                        store_original: false,
                    })),
                },
                ai_pipeline::AiQuery {
                    query: Some(Query::Set(ai_query_types::Set {
                        store: store.clone(),
                        inputs: vec![AiStoreEntry {
                            key: Some(StoreInput {
                                value: Some(Value::Audio(short_audio)),
                            }),
                            value: None,
                        }],
                        preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                        execution_provider: None,
                    })),
                },
            ],
        }))
        .await
        .expect("pipeline failed")
        .into_inner();

    // Both CreateStore and Set should succeed
    assert_eq!(result.responses.len(), 2);
    assert!(
        matches!(
            result.responses[1].response,
            Some(ai_pipeline::ai_server_response::Response::Set(_))
        ),
        "Expected Set response, got: {:?}",
        result.responses[1]
    );
}
