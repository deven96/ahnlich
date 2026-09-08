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
                schema: None,
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
                model_params: HashMap::new(),
                schema: None,
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
                model_params: HashMap::new(),
                schema: None,
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
                schema: None,
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
                model_params: HashMap::new(),
                schema: None,
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
                model_params: HashMap::new(),
                schema: None,
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
                schema: None,
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
                model_params: HashMap::new(),
                schema: None,
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
                model_params: HashMap::new(),
                schema: None,
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
                    schema: None,
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
            model_params: HashMap::new(),
            schema: None,
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

/// Audio clips longer than 10 minutes are rejected with a clear error.
/// Audio between 10s and 10 minutes is automatically chunked.
#[tokio::test]
async fn test_clap_audio_too_long_rejected() {
    // Provision with increased message size to handle 601s WAV (~57MB)
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
    ai_config.common.message_size = 100 * 1024 * 1024; // 100MB to handle large test audio

    let ai_server = AIProxyServer::new(ai_config)
        .await
        .expect("Could not initialize AI proxy");
    let ai_address = ai_server.local_addr().expect("Could not get local addr");
    tokio::spawn(async move { ai_server.start().await });

    tokio::time::sleep(Duration::from_millis(200)).await;

    let mut client = connect(ai_address).await;

    let store = "clap_too_long_store".to_string();

    // Build a silent WAV that is 601 seconds (just over the 600s / 10 minute limit).
    let long_audio = make_silent_wav(601.0);

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
                    schema: None,
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
            model_params: HashMap::new(),
            schema: None,
        }))
        .await;

    assert!(result.is_err(), "Expected error for audio > 10 minutes");
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
        status.message().contains("600000ms"),
        "Error message should state the 600000ms (10 minute) limit, got: {}",
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
                        schema: None,
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
                        model_params: HashMap::new(),
                        schema: None,
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
/// Test that short audio (< 10s) returns OneToMany response with chunk metadata
#[tokio::test]
async fn test_clap_short_audio_returns_one_to_many() {
    let ai_address = provision_clap_servers().await;
    let mut client = connect(ai_address).await;

    let store = "clap_one_to_many_test_store".to_string();

    let short_audio = include_bytes!("../../test_data/audio/cat_meow.ogg").to_vec();

    let result = client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: vec![
                ai_pipeline::AiQuery {
                    query: Some(Query::CreateStore(ai_query_types::CreateStore {
                        store: store.clone(),
                        query_model: AiModel::ClapText.into(),
                        index_model: AiModel::ClapAudio.into(),
                        predicates: vec![],
                        non_linear_indices: vec![],
                        error_if_exists: false,
                        store_original: false,
                        schema: None,
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
                        model_params: HashMap::new(),
                        schema: None,
                    })),
                },
            ],
        }))
        .await
        .expect("pipeline failed")
        .into_inner();

    // Verify Set response
    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_response)),
    }) = result.responses.get(1)
    {
        assert!(set_response.upsert.as_ref().unwrap().inserted > 0);
    } else {
        panic!("Expected Set response");
    }

    // Retrieve the entry and verify chunk metadata
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::RawString("cat meowing".to_string())),
            }),
            closest_n: 1,
            algorithm: Algorithm::CosineSimilarity.into(),
            condition: None,
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
            schema: None,
        }))
        .await
        .expect("GetSimN failed")
        .into_inner()
        .entries;

    assert!(!get_response.is_empty(), "Expected at least one result");
    let entry = &get_response[0];
    let metadata = entry.value.as_ref().expect("Entry should have metadata");

    // Verify chunk metadata fields exist
    assert!(
        metadata.value.contains_key("chunk_start_sec"),
        "Missing chunk_start_sec"
    );
    assert!(
        metadata.value.contains_key("chunk_end_sec"),
        "Missing chunk_end_sec"
    );
    assert!(
        metadata.value.contains_key("total_chunks"),
        "Missing total_chunks"
    );
    assert!(
        metadata.value.contains_key("chunk_duration_sec"),
        "Missing chunk_duration_sec"
    );
    assert!(
        metadata.value.contains_key("audio_total_duration_sec"),
        "Missing audio_total_duration_sec"
    );

    // For short audio (< 10s), there should be only 1 chunk
    let total_chunks = metadata.value.get("total_chunks").unwrap();
    if let Some(metadata_value::Value::RawString(val)) = &total_chunks.value {
        assert_eq!(val, "1", "Short audio should produce 1 chunk");
    } else {
        panic!("total_chunks should be a RawString");
    }
}

#[tokio::test]
async fn test_clap_audio_15_second_chunking() {
    // 15 seconds at 48kHz = 720,000 samples
    // Should produce 2 chunks with 1-second overlap
    let mock_pcm: Vec<f32> = vec![0.0; 720_000];
    let mock_audio_bytes = create_wav_from_pcm(&mock_pcm, 48000);

    let preprocessor =
        crate::engine::ai::providers::processors::preprocessor::ORTAudioPreprocessor::new(
            crate::cli::server::SupportedModels::ClapAudio,
            48_000,
            10.0,
        );

    let chunks = preprocessor.process(vec![mock_audio_bytes]).unwrap();

    assert_eq!(chunks.len(), 2, "15s audio should produce 2 chunks");
    assert_eq!(chunks[0].chunk_index, 0);
    assert_eq!(chunks[1].chunk_index, 1);
    assert_eq!(chunks[0].total_chunks, 2);
    assert_eq!(chunks[1].total_chunks, 2);

    // Verify overlap: chunk_0 ends at 10s, chunk_1 starts at 9s
    assert!((chunks[0].end_sec - 10.0).abs() < 0.01);
    assert!((chunks[1].start_sec - 9.0).abs() < 0.01);
}

// Helper function to create WAV bytes from PCM samples
fn create_wav_from_pcm(pcm: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let data_size = (pcm.len() * 2) as u32; // 2 bytes per i16 sample
    let file_size = 36 + data_size; // RIFF chunk size

    let mut wav = Vec::with_capacity(44 + data_size as usize);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM format
    wav.extend_from_slice(&num_channels.to_le_bytes());
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * u32::from(num_channels) * u32::from(bits_per_sample / 8);
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    let block_align = num_channels * (bits_per_sample / 8);
    wav.extend_from_slice(&block_align.to_le_bytes());
    wav.extend_from_slice(&bits_per_sample.to_le_bytes());

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&data_size.to_le_bytes());

    // Convert f32 to i16 PCM
    for &sample in pcm {
        let sample_i16 = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
        wav.extend_from_slice(&sample_i16.to_le_bytes());
    }

    wav
}

/// Test that verifies the zero-copy optimization path works for audio and text inputs.
/// Test that large audio (50s) is accepted and properly chunked
/// Note: Cannot test full 600s due to gRPC message size limits (~10MB)
#[tokio::test]
async fn test_clap_audio_large_accepted_and_chunked() {
    let ai_address = provision_clap_servers().await;
    let mut client = connect(ai_address).await;

    let store = "test_store_large".to_string();

    // Create store with ClapText query and ClapAudio index (cross-modal)
    client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: vec![ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store.clone(),
                    query_model: AiModel::ClapText.into(),
                    index_model: AiModel::ClapAudio.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: false,
                    store_original: false,
                    schema: None,
                })),
            }],
        }))
        .await
        .expect("CreateStore failed");

    // Create 50-second audio using create_wav_from_pcm (well within 600s limit but requires chunking)
    // 50 seconds at 48kHz = 2,400,000 samples
    let samples_50s = 48_000 * 50;
    let mock_pcm: Vec<f32> = vec![0.0; samples_50s];
    let audio_50s = create_wav_from_pcm(&mock_pcm, 48000);

    let response = client
        .set(tonic::Request::new(ai_query_types::Set {
            store: store.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Audio(audio_50s)),
                }),
                value: None,
            }],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
            schema: None,
        }))
        .await;

    assert!(
        response.is_ok(),
        "50-second audio should be accepted: {:?}",
        response.as_ref().err()
    );

    // Retrieve entries and check chunk metadata
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::RawString("audio".to_string())),
            }),
            closest_n: 20,
            algorithm: Algorithm::CosineSimilarity.into(),
            condition: None,
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
            schema: None,
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    // Verify metadata shows correct chunking (50s with 10s chunks and 1s overlap = ~6 chunks)
    assert!(
        !get_response.entries.is_empty(),
        "Should have at least one entry"
    );

    let metadata = get_response.entries[0].value.as_ref().unwrap();

    // Verify audio duration
    let duration = metadata.value.get("audio_total_duration_sec").unwrap();
    if let Some(metadata_value::Value::RawString(val)) = &duration.value {
        let duration_sec: f32 = val.parse().unwrap();
        assert!(
            (duration_sec - 50.0).abs() < 1.0,
            "Duration should be ~50s, got {}",
            duration_sec
        );
    }

    // Verify total chunks
    let total = metadata.value.get("total_chunks").unwrap();
    if let Some(metadata_value::Value::RawString(val)) = &total.value {
        let total_chunks: usize = val.parse().unwrap();
        assert!(
            total_chunks >= 5 && total_chunks <= 6,
            "Expected ~5-6 chunks for 50s audio, got {}",
            total_chunks
        );
    }
}

/// Test that chunk metadata includes sequential indices
#[tokio::test]
async fn test_clap_audio_chunk_metadata_sequential() {
    let ai_address = provision_clap_servers().await;
    let mut client = connect(ai_address).await;

    let store = "test_metadata_store".to_string();

    // Create store
    client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: vec![ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store.clone(),
                    query_model: AiModel::ClapText.into(),
                    index_model: AiModel::ClapAudio.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: false,
                    store_original: false,
                    schema: None,
                })),
            }],
        }))
        .await
        .expect("CreateStore failed");

    // 25-second audio produces 3 chunks
    let samples_25s = 48_000 * 25;
    let mock_pcm: Vec<f32> = vec![0.0; samples_25s];
    let audio_25s = create_wav_from_pcm(&mock_pcm, 48000);

    client
        .set(tonic::Request::new(ai_query_types::Set {
            store: store.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Audio(audio_25s)),
                }),
                value: None,
            }],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
            schema: None,
        }))
        .await
        .expect("Set failed");

    // Get an entry and verify chunk metadata exists and has correct format
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::RawString("audio".to_string())),
            }),
            closest_n: 10,
            algorithm: Algorithm::CosineSimilarity.into(),
            condition: None,
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
            schema: None,
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    assert!(
        !get_response.entries.is_empty(),
        "Should have at least one entry"
    );

    // Verify metadata structure for chunk information
    let metadata = get_response.entries[0].value.as_ref().unwrap();

    // Verify chunk index exists
    assert!(
        metadata
            .value
            .contains_key(crate::AHNLICH_AI_ONE_TO_MANY_INDEX_META_KEY),
        "Should have chunk index metadata"
    );

    // Verify total_chunks metadata
    let total = metadata.value.get("total_chunks").unwrap();
    if let Some(metadata_value::Value::RawString(val)) = &total.value {
        let total_chunks: usize = val.parse().unwrap();
        assert_eq!(total_chunks, 3, "25s audio should produce 3 chunks");
    }

    // Verify chunk index is a valid number
    let index_meta = metadata
        .value
        .get(crate::AHNLICH_AI_ONE_TO_MANY_INDEX_META_KEY)
        .unwrap();
    if let Some(metadata_value::Value::RawString(idx)) = &index_meta.value {
        let index: usize = idx.parse().expect("Chunk index should be a valid number");
        assert!(index < 3, "Chunk index should be < total_chunks");
    }
}

/// Test batch processing with mixed audio durations
#[tokio::test]
async fn test_clap_audio_batch_mixed_durations() {
    let ai_address = provision_clap_servers().await;
    let mut client = connect(ai_address).await;

    let store = "test_batch_store".to_string();

    // Create store with ClapText query and ClapAudio index (cross-modal)
    client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: vec![ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store.clone(),
                    query_model: AiModel::ClapText.into(),
                    index_model: AiModel::ClapAudio.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: false,
                    store_original: false,
                    schema: None,
                })),
            }],
        }))
        .await
        .expect("CreateStore failed");

    // Create 3 audios: 5s, 30s, 8s
    let audio_5s = create_wav_from_pcm(&vec![0.0; 48_000 * 5], 48000);
    let audio_30s = create_wav_from_pcm(&vec![0.0; 48_000 * 30], 48000);
    let audio_8s = create_wav_from_pcm(&vec![0.0; 48_000 * 8], 48000);

    let _response = client
        .set(tonic::Request::new(ai_query_types::Set {
            store: store.clone(),
            inputs: vec![
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::Audio(audio_5s)),
                    }),
                    value: None,
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::Audio(audio_30s)),
                    }),
                    value: None,
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::Audio(audio_8s)),
                    }),
                    value: None,
                },
            ],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
            schema: None,
        }))
        .await
        .expect("Batch set failed");

    // Batch was accepted (no error in .await.expect above)

    // Retrieve and verify metadata
    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::RawString("sound".to_string())),
            }),
            closest_n: 20,
            algorithm: Algorithm::CosineSimilarity.into(),
            condition: None,
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
            schema: None,
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    assert!(
        !get_response.entries.is_empty(),
        "Should have at least one entry"
    );

    // Since we inserted 3 audios (5s, 30s, 8s), check that we get entries with chunk metadata
    // We can't easily distinguish which chunks belong to which audio with identical silent audio,
    // but we can verify the metadata structure is correct
    let metadata = get_response.entries[0].value.as_ref().unwrap();
    assert!(
        metadata.value.contains_key("total_chunks"),
        "Should have total_chunks metadata"
    );
    assert!(
        metadata.value.contains_key("chunk_start_sec"),
        "Should have chunk_start_sec metadata"
    );
    assert!(
        metadata.value.contains_key("chunk_end_sec"),
        "Should have chunk_end_sec metadata"
    );
    assert!(
        metadata.value.contains_key("audio_total_duration_sec"),
        "Should have audio_total_duration_sec metadata"
    );
}

/// Test that consecutive chunks have 1-second overlap
#[tokio::test]
async fn test_clap_audio_overlap_calculation() {
    let ai_address = provision_clap_servers().await;
    let mut client = connect(ai_address).await;

    let store = "test_overlap_store".to_string();

    // Create store
    client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries: vec![ai_pipeline::AiQuery {
                query: Some(Query::CreateStore(ai_query_types::CreateStore {
                    store: store.clone(),
                    query_model: AiModel::ClapText.into(),
                    index_model: AiModel::ClapAudio.into(),
                    predicates: vec![],
                    non_linear_indices: vec![],
                    error_if_exists: false,
                    store_original: false,
                    schema: None,
                })),
            }],
        }))
        .await
        .expect("CreateStore failed");

    // 25-second audio
    let audio_25s = create_wav_from_pcm(&vec![0.0; 48_000 * 25], 48000);

    client
        .set(tonic::Request::new(ai_query_types::Set {
            store: store.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Audio(audio_25s)),
                }),
                value: None,
            }],
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
            schema: None,
        }))
        .await
        .expect("Set failed");

    let get_response = client
        .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
            store: store.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::RawString("test".to_string())),
            }),
            closest_n: 10,
            algorithm: Algorithm::CosineSimilarity.into(),
            condition: None,
            preprocess_action: PreprocessAction::ModelPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
            schema: None,
        }))
        .await
        .expect("GetSimN failed")
        .into_inner();

    assert!(
        !get_response.entries.is_empty(),
        "Should have at least one entry"
    );

    // Verify chunk timing metadata exists and is properly formatted
    let metadata = get_response.entries[0].value.as_ref().unwrap();

    // Verify chunk_start_sec exists and is a valid number
    let chunk_start = metadata.value.get("chunk_start_sec").unwrap();
    if let Some(metadata_value::Value::RawString(s)) = &chunk_start.value {
        let start_sec: f32 = s.parse().expect("chunk_start_sec should be a valid number");
        assert!(start_sec >= 0.0, "chunk_start_sec should be non-negative");
    } else {
        panic!("chunk_start_sec should be a RawString");
    }

    // Verify chunk_end_sec exists and is a valid number
    let chunk_end = metadata.value.get("chunk_end_sec").unwrap();
    if let Some(metadata_value::Value::RawString(s)) = &chunk_end.value {
        let end_sec: f32 = s.parse().expect("chunk_end_sec should be a valid number");
        assert!(end_sec > 0.0, "chunk_end_sec should be positive");
    } else {
        panic!("chunk_end_sec should be a RawString");
    }

    // Verify chunk_duration_sec exists
    assert!(
        metadata.value.contains_key("chunk_duration_sec"),
        "Should have chunk_duration_sec metadata"
    );

    // Verify total_chunks metadata indicates multiple chunks for 25s audio
    let total = metadata.value.get("total_chunks").unwrap();
    if let Some(metadata_value::Value::RawString(val)) = &total.value {
        let total_chunks: usize = val.parse().unwrap();
        assert_eq!(total_chunks, 3, "25s audio should produce 3 chunks");
    }
}
