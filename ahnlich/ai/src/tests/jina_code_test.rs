use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::{collections::HashMap, net::SocketAddr};
use tokio::time::Duration;
use tonic::transport::Channel;
use utils::server::AhnlichServerUtils;

use crate::{cli::AIProxyConfig, server::handler::AIProxyServer};

use ahnlich_types::{
    ai::{
        models::AiModel,
        pipeline::{self as ai_pipeline, ai_query::Query},
        preprocess::PreprocessAction,
        query as ai_query_types,
    },
    algorithm::algorithms::Algorithm,
    keyval::{AiStoreEntry, StoreInput, StoreValue, store_input::Value},
    metadata::{MetadataValue, metadata_value::Value as MValue},
    services::ai_service::ai_service_client::AiServiceClient,
    shared::info::StoreUpsert,
};

static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());
static AI_CONFIG: Lazy<AIProxyConfig> = Lazy::new(|| AIProxyConfig::default().os_select_port());

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
async fn test_jina_code_model_loads_and_produces_embeddings() {
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "code_store".to_string();

    // Python factorial function
    let python_code = r#"def factorial(n):
    if n == 0:
        return 1
    else:
        return n * factorial(n-1)"#;

    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::JinaEmbeddingsV2BaseCode.into(),
            index_model: AiModel::JinaEmbeddingsV2BaseCode.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        })),
    };

    let set_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::RawString(python_code.to_string())),
                }),
                value: Some(StoreValue {
                    value: HashMap::new(),
                }),
            }],
            preprocess_action: PreprocessAction::NoPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        })),
    };

    let pipelined_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 2);

    // Verify CreateStore succeeded
    match &responses[0].response {
        Some(ai_pipeline::ai_server_response::Response::Unit(_)) => {}
        _ => panic!("Expected Unit response for CreateStore"),
    }

    // Verify Set succeeded with 1 insertion
    match &responses[1].response {
        Some(ai_pipeline::ai_server_response::Response::Set(set_response)) => {
            assert_eq!(
                set_response.upsert,
                Some(StoreUpsert {
                    inserted: 1,
                    updated: 0,
                })
            );
        }
        _ => panic!("Expected Set response"),
    }
}

#[tokio::test]
async fn test_jina_code_semantic_similarity_across_languages() {
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "cross_lang_store".to_string();

    // Code snippets
    let python_factorial = r#"def factorial(n):
    if n == 0:
        return 1
    else:
        return n * factorial(n-1)"#;

    let js_factorial = r#"function factorial(n) {
    if (n === 0) {
        return 1;
    }
    return n * factorial(n - 1);
}"#;

    let rust_factorial = r#"fn factorial(n: u32) -> u32 {
    if n == 0 {
        1
    } else {
        n * factorial(n - 1)
    }
}"#;

    let unrelated_http_server = r#"const express = require('express');
const app = express();
app.get('/health', (req, res) => {
    res.json({ status: 'ok' });
});
app.listen(3000);"#;

    // Create store
    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::JinaEmbeddingsV2BaseCode.into(),
            index_model: AiModel::JinaEmbeddingsV2BaseCode.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        })),
    };

    // Insert code snippets
    let set_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
            inputs: vec![
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString(python_factorial.to_string())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from([(
                            "lang".to_string(),
                            MetadataValue {
                                value: Some(MValue::RawString("python".to_string())),
                            },
                        )]),
                    }),
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString(js_factorial.to_string())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from([(
                            "lang".to_string(),
                            MetadataValue {
                                value: Some(MValue::RawString("javascript".to_string())),
                            },
                        )]),
                    }),
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString(rust_factorial.to_string())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from([(
                            "lang".to_string(),
                            MetadataValue {
                                value: Some(MValue::RawString("rust".to_string())),
                            },
                        )]),
                    }),
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString(unrelated_http_server.to_string())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from([(
                            "lang".to_string(),
                            MetadataValue {
                                value: Some(MValue::RawString("javascript".to_string())),
                            },
                        )]),
                    }),
                },
            ],
            preprocess_action: PreprocessAction::NoPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        })),
    };

    // Search with Python factorial
    let get_sim_n_query = ai_pipeline::AiQuery {
        query: Some(Query::GetSimN(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::RawString(python_factorial.to_string())),
            }),
            condition: None,
            closest_n: 4,
            preprocess_action: PreprocessAction::NoPreprocessing.into(),
            algorithm: Algorithm::CosineSimilarity.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        })),
    };

    let pipelined_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_query, get_sim_n_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 3);

    // Verify search results
    match &responses[2].response {
        Some(ai_pipeline::ai_server_response::Response::GetSimN(sim_response)) => {
            assert_eq!(sim_response.entries.len(), 4);

            // First result should be Python factorial (self)
            if let Some(Value::RawString(s)) =
                sim_response.entries[0].key.as_ref().unwrap().value.as_ref()
            {
                assert!(s.contains("def factorial"));
            } else {
                panic!("Expected RawString value");
            }

            // Second and third should be JS and Rust factorial (similar semantics)
            let second = if let Some(Value::RawString(s)) =
                sim_response.entries[1].key.as_ref().unwrap().value.as_ref()
            {
                s.clone()
            } else {
                panic!("Expected RawString value");
            };
            let third = if let Some(Value::RawString(s)) =
                sim_response.entries[2].key.as_ref().unwrap().value.as_ref()
            {
                s.clone()
            } else {
                panic!("Expected RawString value");
            };
            assert!(second.contains("factorial") || third.contains("factorial"));

            // HTTP server should rank last (most dissimilar)
            if let Some(Value::RawString(s)) =
                sim_response.entries[3].key.as_ref().unwrap().value.as_ref()
            {
                assert!(s.contains("express") || s.contains("health"));
            } else {
                panic!("Expected RawString value");
            }
        }
        _ => panic!("Expected GetSimN response"),
    }
}

#[tokio::test]
async fn test_jina_code_natural_language_query() {
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "nl_query_store".to_string();

    // Code snippets
    let enumerate_example = r#"for i, item in enumerate(items):
    print(f"Index {i}: {item}")"#;

    let map_example = r#"result = list(map(lambda x: x * 2, numbers))"#;

    let filter_example = r#"evens = [x for x in numbers if x % 2 == 0]"#;

    // Create store
    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::JinaEmbeddingsV2BaseCode.into(),
            index_model: AiModel::JinaEmbeddingsV2BaseCode.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        })),
    };

    // Insert code snippets
    let set_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
            inputs: vec![
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString(enumerate_example.to_string())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from([(
                            "desc".to_string(),
                            MetadataValue {
                                value: Some(MValue::RawString("enumerate".to_string())),
                            },
                        )]),
                    }),
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString(map_example.to_string())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from([(
                            "desc".to_string(),
                            MetadataValue {
                                value: Some(MValue::RawString("map".to_string())),
                            },
                        )]),
                    }),
                },
                AiStoreEntry {
                    key: Some(StoreInput {
                        value: Some(Value::RawString(filter_example.to_string())),
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from([(
                            "desc".to_string(),
                            MetadataValue {
                                value: Some(MValue::RawString("filter".to_string())),
                            },
                        )]),
                    }),
                },
            ],
            preprocess_action: PreprocessAction::NoPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        })),
    };

    // Natural language query
    let nl_query = "How do I access the index while iterating";
    let get_sim_n_query = ai_pipeline::AiQuery {
        query: Some(Query::GetSimN(ai_query_types::GetSimN {
            store: store_name.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::RawString(nl_query.to_string())),
            }),
            condition: None,
            closest_n: 3,
            preprocess_action: PreprocessAction::NoPreprocessing.into(),
            algorithm: Algorithm::CosineSimilarity.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        })),
    };

    let pipelined_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_query, get_sim_n_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 3);

    // Verify enumerate example ranks first
    match &responses[2].response {
        Some(ai_pipeline::ai_server_response::Response::GetSimN(sim_response)) => {
            assert_eq!(sim_response.entries.len(), 3);

            // First result should be enumerate example (matches "access index while iterating")
            if let Some(Value::RawString(s)) =
                sim_response.entries[0].key.as_ref().unwrap().value.as_ref()
            {
                assert!(s.contains("enumerate"));
            } else {
                panic!("Expected RawString value");
            }
        }
        _ => panic!("Expected GetSimN response"),
    }
}

#[tokio::test]
async fn test_jina_code_embedding_dimensions() {
    let ai_address = provision_test_servers().await;
    let channel =
        Channel::from_shared(format!("http://{}", ai_address)).expect("Failed to create channel");
    let mut client = AiServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let store_name = "dimension_test_store".to_string();

    let sample_code = r#"def hello():
    print("Hello, World!")"#;

    // Create store
    let create_store_query = ai_pipeline::AiQuery {
        query: Some(Query::CreateStore(ai_query_types::CreateStore {
            store: store_name.clone(),
            query_model: AiModel::JinaEmbeddingsV2BaseCode.into(),
            index_model: AiModel::JinaEmbeddingsV2BaseCode.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            store_original: true,
        })),
    };

    // Insert code
    let set_query = ai_pipeline::AiQuery {
        query: Some(Query::Set(ai_query_types::Set {
            store: store_name.clone(),
            inputs: vec![AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::RawString(sample_code.to_string())),
                }),
                value: Some(StoreValue {
                    value: HashMap::new(),
                }),
            }],
            preprocess_action: PreprocessAction::NoPreprocessing.into(),
            execution_provider: None,
            model_params: HashMap::new(),
        })),
    };

    // List stores to check dimensions
    let list_stores_query = ai_pipeline::AiQuery {
        query: Some(Query::ListStores(ai_query_types::ListStores {})),
    };

    let pipelined_request = ai_pipeline::AiRequestPipeline {
        queries: vec![create_store_query, set_query, list_stores_query],
    };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to send pipeline request");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 3);

    // Verify 768 dimensions
    match &responses[2].response {
        Some(ai_pipeline::ai_server_response::Response::StoreList(store_list)) => {
            let store = store_list
                .stores
                .iter()
                .find(|s| s.name == store_name)
                .expect("Store not found");

            assert_eq!(
                store.embedding_size, 768,
                "Jina Code embeddings should be 768-dimensional"
            );
        }
        _ => panic!("Expected StoreList response"),
    }
}
