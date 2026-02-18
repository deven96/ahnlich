use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;
use once_cell::sync::Lazy;
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
    services::ai_service::ai_service_client::AiServiceClient,
};

static DB_CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());

static CLAP_AI_CONFIG: Lazy<AIProxyConfig> = Lazy::new(|| {
    AIProxyConfig::default()
        .os_select_port()
        .set_supported_models(vec![SupportedModels::ClapAudio, SupportedModels::ClapText])
});

async fn provision_clap_servers() -> SocketAddr {
    let server = Server::new(&DB_CONFIG)
        .await
        .expect("Failed to create DB server");
    let db_port = server.local_addr().unwrap().port();
    tokio::spawn(async move { server.start().await });

    let mut config = CLAP_AI_CONFIG.clone();
    config.db_port = db_port;

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not initialize AI proxy");
    let ai_address = ai_server.local_addr().expect("Could not get local addr");
    tokio::spawn(async move { ai_server.start().await });

    tokio::time::sleep(Duration::from_millis(200)).await;
    ai_address
}

/// Indexes three real audio clips (dog bark, cat meow, rain) using ClapAudio,
/// then queries with natural language using ClapText and asserts the semantically
/// correct audio clip is returned as the top result.
///
/// Audio files are sourced from Wikimedia Commons (CC BY-SA):
///   - dog_bark.ogg: "Barking of a dog" by Yannickteejay
///   - cat_meow.ogg: "Meow" by Mramoeba
///   - rain.ogg:     "Rain" by Bertof
#[tokio::test]
async fn test_clap_cross_modal_retrieval() {
    let ai_address = provision_clap_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "clap_retrieval_store".to_string();

    let dog_bytes = include_bytes!("../../test_data/audio/dog_bark.ogg").to_vec();
    let cat_bytes = include_bytes!("../../test_data/audio/cat_meow.ogg").to_vec();
    let rain_bytes = include_bytes!("../../test_data/audio/rain.ogg").to_vec();

    let queries = vec![
        ai_pipeline::AiQuery {
            query: Some(Query::CreateStore(ai_query_types::CreateStore {
                store: store_name.clone(),
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
                store: store_name.clone(),
                inputs: vec![
                    AiStoreEntry {
                        key: Some(StoreInput {
                            value: Some(Value::Audio(dog_bytes)),
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from([(
                                "label".to_string(),
                                ahnlich_types::metadata::MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "dog".to_string(),
                                        ),
                                    ),
                                },
                            )]),
                        }),
                    },
                    AiStoreEntry {
                        key: Some(StoreInput {
                            value: Some(Value::Audio(cat_bytes)),
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from([(
                                "label".to_string(),
                                ahnlich_types::metadata::MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "cat".to_string(),
                                        ),
                                    ),
                                },
                            )]),
                        }),
                    },
                    AiStoreEntry {
                        key: Some(StoreInput {
                            value: Some(Value::Audio(rain_bytes)),
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from([(
                                "label".to_string(),
                                ahnlich_types::metadata::MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            "rain".to_string(),
                                        ),
                                    ),
                                },
                            )]),
                        }),
                    },
                ],
                preprocess_action: PreprocessAction::ModelPreprocessing.into(),
                execution_provider: None,
            })),
        },
    ];

    let set_response = client
        .pipeline(tonic::Request::new(ai_pipeline::AiRequestPipeline {
            queries,
        }))
        .await
        .expect("Failed to index audio clips");

    let set_responses = set_response.into_inner().responses;
    if let Some(ai_pipeline::AiServerResponse {
        response: Some(ai_pipeline::ai_server_response::Response::Set(set_resp)),
    }) = set_responses.get(1)
    {
        assert_eq!(
            set_resp.upsert.as_ref().unwrap().inserted,
            3,
            "Expected 3 audio clips indexed"
        );
    } else {
        panic!("Expected Set response");
    }

    // Each query should retrieve the semantically matching clip as its top result.
    let test_cases = [
        ("a dog barking loudly", "dog"),
        ("a cat meowing", "cat"),
        ("heavy rain falling", "rain"),
    ];

    for (query_text, expected_label) in test_cases {
        let sim_response = client
            .get_sim_n(tonic::Request::new(ai_query_types::GetSimN {
                store: store_name.clone(),
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
            .expect("GetSimN failed");

        let entries = sim_response.into_inner().entries;
        assert_eq!(
            entries.len(),
            1,
            "Expected 1 result for query '{query_text}'"
        );

        let top = &entries[0];
        let label = top
            .value
            .as_ref()
            .and_then(|v| v.value.get("label"))
            .and_then(|m| m.value.as_ref())
            .and_then(|v| {
                if let ahnlich_types::metadata::metadata_value::Value::RawString(s) = v {
                    Some(s.as_str())
                } else {
                    None
                }
            })
            .unwrap_or("unknown");

        assert_eq!(
            label, expected_label,
            "Query '{query_text}' should return '{expected_label}' but got '{label}'"
        );
    }
}
