use crate::engine::store::StoreHandler;
use crate::server::handler::Server;
use crate::{cli::ServerConfig, errors::ServerError};
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::{
    self, HnswConfig, KdTreeConfig, NonLinearAlgorithm, non_linear_index,
};
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::metadata::metadata_value::Value as MetadataValueEnum;
use ahnlich_types::predicates::{
    self, Predicate, PredicateCondition, predicate::Kind as PredicateKind,
    predicate_condition::Kind as PredicateConditionKind,
};
use ahnlich_types::schema::Schema;
use ahnlich_types::server_types::ServerType;
use ahnlich_types::shared::info::StoreUpsert;
use ahnlich_types::similarity::Similarity;
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::time::Duration;
use utils::server::AhnlichServerUtils;

use ahnlich_types::{
    db::{
        pipeline::{self as db_pipeline, db_query::Query},
        query as db_query_types, server as db_response_types,
    },
    keyval::StoreName,
    services::db_service::db_service_client::DbServiceClient,
};
use tonic::transport::Channel;

static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());

static CONFIG_WITH_MAX_CLIENTS: Lazy<ServerConfig> =
    Lazy::new(|| ServerConfig::default().os_select_port().maximum_clients(2));

static PERSISTENCE_FILE: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ahnlich.dat"));

static CONFIG_WITH_PERSISTENCE: Lazy<ServerConfig> = Lazy::new(|| {
    ServerConfig::default()
        .os_select_port()
        .persistence_interval(200)
        .persist_location((*PERSISTENCE_FILE).clone())
});

static HNSW_PERSISTENCE_FILE: Lazy<PathBuf> =
    Lazy::new(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ahnlich_hnsw.dat"));

static CONFIG_WITH_HNSW_PERSISTENCE: Lazy<ServerConfig> = Lazy::new(|| {
    ServerConfig::default()
        .os_select_port()
        .persistence_interval(200)
        .persist_location((*HNSW_PERSISTENCE_FILE).clone())
});

#[tokio::test]
async fn test_grpc_ping_test() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let response = client
        .ping(tonic::Request::new(ahnlich_types::db::query::Ping {}))
        .await
        .expect("Failed to ping");

    let expected = db_response_types::Pong {};
    println!("Response: {response:?}");
    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_maximum_client_restriction_works() {
    let server = Server::new(&CONFIG_WITH_MAX_CLIENTS)
        .await
        .expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let mut first_client = DbServiceClient::connect(channel).await.expect("Failure");
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let mut second_client = DbServiceClient::connect(channel).await.expect("Failure");
    let response = second_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");
    assert_eq!(response.into_inner().clients.len(), 2);
    let response = first_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");

    assert_eq!(response.into_inner().clients.len(), 2);
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let mut third_client = DbServiceClient::connect(channel).await.expect("Failure");
    third_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect_err("third client failed to error with a max of 2");
    let response = second_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");
    assert_eq!(response.into_inner().clients.len(), 2);
    drop(first_client);
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let mut fourth_client = DbServiceClient::connect(channel).await.expect("Failure");
    let response = fourth_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");
    // The third client never connected so we expect only 2
    assert_eq!(response.into_inner().clients.len(), 2);
}

#[tokio::test]
async fn test_server_client_info() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");

    let mut first_client = DbServiceClient::connect(channel).await.expect("Failure");

    let channel = Channel::from_shared(address.clone()).expect("Faild to get channel");
    let second_client = DbServiceClient::connect(channel).await.expect("Failure");

    let response = first_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");

    assert_eq!(response.into_inner().clients.len(), 2);
    drop(second_client);

    let response = first_client
        .list_clients(tonic::Request::new(
            ahnlich_types::db::query::ListClients {},
        ))
        .await
        .expect("Failed to list clients");
    assert_eq!(response.into_inner().clients.len(), 1);
}

#[tokio::test]
async fn test_simple_stores_list() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let response = client
        .list_stores(tonic::Request::new(ahnlich_types::db::query::ListStores {
            schema: None,
        }))
        .await
        .expect("Failed to get store's list");

    let expected = db_response_types::StoreList { stores: vec![] };
    println!("Response: {response:?}");
    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_create_stores() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let queries = vec![
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to create stores");

    let error_response = ServerError::StoreAlreadyExists(StoreName {
        value: "Main".to_string(),
    });

    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: error_response.to_string(),
                    code: 6,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 0,
                        size_in_bytes: 1056,
                        non_linear_indices: vec![],
                        predicate_indices: vec![],
                        dimension: 3,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_del_pred() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let queries = vec![
        db_pipeline::DbQuery {
            query: Some(Query::DelPred(db_query_types::DelPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "planet".into(),
                            value: Some(MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "earth".to_string(),
                                    ),
                                ),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Should not error as it is correct query
        // but should delete nothing as nothing matches predicate
        db_pipeline::DbQuery {
            query: Some(Query::DelPred(db_query_types::DelPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "planet".into(),
                            value: Some(MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "earth".to_string(),
                                    ),
                                ),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.4, 1.5],
                        }),
                        value: Some(ahnlich_types::keyval::StoreValue {
                            value: HashMap::from_iter([(
                                "planet".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("jupiter".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(ahnlich_types::keyval::StoreValue {
                            value: HashMap::from_iter([(
                                "planet".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("mars".into())),
                                },
                            )]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::DelPred(db_query_types::DelPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "planet".into(),
                            value: Some(MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "mars".to_string(),
                                    ),
                                ),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![
                    StoreKey {
                        key: vec![1.4, 1.5],
                    },
                    StoreKey {
                        key: vec![1.6, 1.7],
                    },
                ],
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::DelPred(db_query_types::DelPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "planet".into(),
                            value: Some(MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        "mars".to_string(),
                                    ),
                                ),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to create stores");

    let error_response = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: error_response.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 2,
                        size_in_bytes: 1264,
                        non_linear_indices: vec![],
                        predicate_indices: vec![],
                        dimension: 2,
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "planet".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("mars".into())),
                                },
                            )]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 0,
                        size_in_bytes: 1056,
                        non_linear_indices: vec![],
                        predicate_indices: vec![],
                        dimension: 2,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_del_key() {
    // Server setup (same as your gRPC examples)
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![],
                schema: None,
            })),
        },
        // Create the store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 4,
                create_predicates: vec!["role".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Should not error but delete nothing (empty store)
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 1.1, 1.2, 1.3],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.1, 1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2], // 3 dimensions vs store's 4
                }],
                schema: None,
            })),
        },
        // Should work - delete existing key
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 4,
        input_dimension: 3,
    };

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 2,
                        size_in_bytes: 1176,
                        non_linear_indices: vec![],
                        predicate_indices: vec!["role".to_string()],
                        dimension: 4,
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 1,
                        size_in_bytes: 1128,
                        non_linear_indices: vec![],
                        predicate_indices: vec!["role".to_string()],
                        dimension: 4,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_server_with_persistence() {
    // First server instance
    //
    // Clean up - delete persistence file
    let _ = std::fs::remove_file(&*PERSISTENCE_FILE);

    let server = Server::new(&CONFIG_WITH_PERSISTENCE)
        .await
        .expect("Failed to create server");
    let write_flag = server.write_flag();
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // First set of operations
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![],
                schema: None,
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 4,
                create_predicates: vec!["role".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Should not error but delete nothing
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 1.1, 1.2, 1.3],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.1, 1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "role".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::Image(vec![1, 2, 3])),
                                },
                            )]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2],
                }],
                schema: None,
            })),
        },
        // Should delete existing key
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 4,
        input_dimension: 3,
    };

    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 2,
                        size_in_bytes: 1304,
                        non_linear_indices: vec![],
                        predicate_indices: vec!["role".to_string()],
                        dimension: 4,
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3, // DimensionMismatch
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 1,
                        size_in_bytes: 1256,
                        non_linear_indices: vec![],
                        predicate_indices: vec!["role".to_string()],
                        dimension: 4,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
    assert!(write_flag.load(Ordering::SeqCst));

    // Wait for persistence
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Second server instance
    let server = Server::new(&CONFIG_WITH_PERSISTENCE)
        .await
        .expect("Failed to create server");
    let write_flag = server.write_flag();
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    // Verify persistence file exists and is not empty
    let file_metadata = std::fs::metadata(
        &CONFIG_WITH_PERSISTENCE
            .common
            .persist_location
            .clone()
            .unwrap(),
    )
    .unwrap();
    assert!(file_metadata.len() > 0, "The persistence file is empty");

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(200)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Second set of operations to verify persistence
    let queries = vec![
        // Should error as store exists from persistence
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec!["role".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Should not error as store exists
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![],
                schema: None,
            })),
        },
        // Should get persisted data
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.1, 1.2, 1.3, 1.4],
                }],
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let already_exists_error = ServerError::StoreAlreadyExists(StoreName {
        value: "Main".into(),
    });

    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: already_exists_error.to_string(),
                    code: 6,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.1, 1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "role".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::Image(vec![1, 2, 3])),
                                },
                            )]),
                        }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
    assert!(!write_flag.load(Ordering::SeqCst));

    // Clean up - delete persistence file
    let _ = std::fs::remove_file(&*PERSISTENCE_FILE);
}

#[tokio::test]
async fn test_set_in_store() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![],
                schema: None,
            })),
        },
        // Create the store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["role".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Valid set operation
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![DbStoreEntry {
                    key: Some(StoreKey {
                        key: vec![1.23, 1.0, 0.2],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                }],
                schema: None,
            })),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![DbStoreEntry {
                    key: Some(StoreKey {
                        key: vec![2.1], // 1 dimension vs store's 3
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                }],
                schema: None,
            })),
        },
        // Upsert operation
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.23, 1.0, 0.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "role".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("headmaster".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![0.03, 5.1, 3.23],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 3,
        input_dimension: 1,
    };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 1,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 1,
                        updated: 1,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 2,
                        size_in_bytes: 1304,
                        non_linear_indices: vec![],
                        predicate_indices: vec!["role".to_string()],
                        dimension: 3,
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_remove_non_linear_indices() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Create store with KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["medal".into()],

                non_linear_indices: vec![nonlinear::NonLinearIndex {
                    index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
                }],

                error_if_exists: true,
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![5.0, 5.1, 5.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        // Get similar items using KDTree
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree.into(),
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0, 3.0],
                }),
                condition: None,
                schema: None,
            })),
        },
        // Remove KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::DropNonLinearAlgorithmIndex(
                db_query_types::DropNonLinearAlgorithmIndex {
                    store: "Main".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                    error_if_not_exists: false,
                    schema: None,
                },
            )),
        },
        // Should error as index doesn't exist (with error_if_not_exists=true)
        db_pipeline::DbQuery {
            query: Some(Query::DropNonLinearAlgorithmIndex(
                db_query_types::DropNonLinearAlgorithmIndex {
                    store: "Main".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                    error_if_not_exists: true,
                    schema: None,
                },
            )),
        },
        // Should error as KDTree no longer exists
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree as i32,
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0, 3.0],
                }),
                condition: None,
                schema: None,
            })),
        },
        // Recreate KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::CreateNonLinearAlgorithmIndex(
                db_query_types::CreateNonLinearAlgorithmIndex {
                    store: "Main".to_string(),

                    non_linear_indices: vec![nonlinear::NonLinearIndex {
                        index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
                    }],
                    schema: None,
                },
            )),
        },
        // Should succeed as index exists again
        db_pipeline::DbQuery {
            query: Some(Query::DropNonLinearAlgorithmIndex(
                db_query_types::DropNonLinearAlgorithmIndex {
                    store: "Main".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                    error_if_not_exists: true,
                    schema: None,
                },
            )),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let non_linear_index_err = ServerError::NonLinearIndexNotFound(NonLinearAlgorithm::KdTree);

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 3,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![2.0, 2.1, 2.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("gold".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 1.4599998 }),
                        },
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![1.2, 1.3, 1.4],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("silver".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 3.0600002 }),
                        },
                    ],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: non_linear_index_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: non_linear_index_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::CreateIndex(
                db_response_types::CreateIndex { created_indexes: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_get_sim_n_non_linear() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Create store with KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["medal".into()],
                non_linear_indices: vec![nonlinear::NonLinearIndex {
                    index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
                }],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![5.0, 5.1, 5.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        // Get 2 closest matches without condition
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree as i32,
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0, 3.0],
                }),
                condition: None,
                schema: None,
            })),
        },
        // return just 1 entry regardless of closest_n
        // due to precondition satisfying just one
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree as i32,
                search_input: Some(StoreKey {
                    key: vec![5.0, 2.1, 2.2],
                }),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 3,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![2.0, 2.1, 2.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("gold".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 1.4599998 }),
                        },
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![1.2, 1.3, 1.4],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("silver".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 3.0600002 }),
                        },
                    ],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![db_response_types::GetSimNEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                        similarity: Some(Similarity { value: 9.0 }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_get_sim_n() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::CosineSimilarity as i32,
                search_input: Some(StoreKey { key: vec![] }),
                condition: None,
                schema: None,
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["medal".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![5.0, 5.1, 5.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        // Error due to non-linear algorithm not existing
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::KdTree as i32,
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0, 3.0],
                }),
                condition: None,
                schema: None,
            })),
        },
        // Error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::EuclideanDistance as i32,
                search_input: Some(StoreKey {
                    key: vec![1.1, 2.0],
                }),
                condition: None,
                schema: None,
            })),
        },
        // Get with condition (should return 1 match)
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::CosineSimilarity as i32,
                search_input: Some(StoreKey {
                    key: vec![5.0, 2.1, 2.2],
                }),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        // Get closest 2 with DotProduct
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::DotProductSimilarity as i32,
                search_input: Some(StoreKey {
                    key: vec![1.0, 2.1, 2.2],
                }),
                condition: None,
                schema: None,
            })),
        },
        // Get closest 2 with EuclideanDistance
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 2,
                algorithm: Algorithm::EuclideanDistance as i32,
                search_input: Some(StoreKey {
                    key: vec![1.0, 2.1, 2.2],
                }),
                condition: None,
                schema: None,
            })),
        },
        // Get closest 1 where medal is not gold
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "Main".to_string(),
                closest_n: 1,
                algorithm: Algorithm::CosineSimilarity as i32,
                search_input: Some(StoreKey {
                    key: vec![5.0, 2.1, 2.2],
                }),

                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 3,
        input_dimension: 2,
    };
    let non_linear_index_err = ServerError::NonLinearIndexNotFound(NonLinearAlgorithm::KdTree);

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 3,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: non_linear_index_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![db_response_types::GetSimNEntry {
                        key: Some(StoreKey {
                            key: vec![2.0, 2.1, 2.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("gold".into())),
                                },
                            )]),
                        }),
                        similarity: Some(Similarity {
                            value: 0.9036338825194858,
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![5.0, 5.1, 5.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("bronze".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 27.149998 }),
                        },
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![2.0, 2.1, 2.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("gold".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 11.25 }),
                        },
                    ],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![2.0, 2.1, 2.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("gold".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity { value: 1.0 }),
                        },
                        db_response_types::GetSimNEntry {
                            key: Some(StoreKey {
                                key: vec![1.2, 1.3, 1.4],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "medal".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("silver".into())),
                                    },
                                )]),
                            }),
                            similarity: Some(Similarity {
                                value: 1.1489125293076061,
                            }),
                        },
                    ],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::GetSimN(
                db_response_types::GetSimN {
                    entries: vec![db_response_types::GetSimNEntry {
                        key: Some(StoreKey {
                            key: vec![5.0, 5.1, 5.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                        similarity: Some(Similarity {
                            value: 0.9119372494019118,
                        }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_get_pred() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["medal".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.3, 1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        // Should return empty (no matches)
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::In(predicates::In {
                            key: "medal".into(),
                            values: vec![MetadataValue {
                                value: Some(MetadataValueEnum::RawString("gold".into())),
                            }],
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        // Get where medal != silver
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("silver".into())),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        // Get where medal != bronze
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotEquals(predicates::NotEquals {
                            key: "medal".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("bronze".into())),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get { entries: vec![] },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.3, 1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("bronze".into())),
                                },
                            )]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("silver".into())),
                                },
                            )]),
                        }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_get_key() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let socket_addr = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", socket_addr);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![],
                schema: None,
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 0.2],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "title".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("sorcerer".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.2, 0.3],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "title".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("elf".into())),
                                },
                            )]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![
                    StoreKey {
                        key: vec![0.2, 0.3, 0.4],
                    },
                    StoreKey {
                        key: vec![0.2, 0.3, 0.4],
                    },
                    StoreKey {
                        key: vec![0.4, 0.6],
                    },
                ],
                schema: None,
            })),
        },
        // Should return empty (keys don't exist)
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![
                    StoreKey {
                        key: vec![0.4, 0.6],
                    },
                    StoreKey {
                        key: vec![0.2, 0.5],
                    },
                ],
                schema: None,
            })),
        },
        // Get existing keys
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![
                    StoreKey {
                        key: vec![1.2, 0.3],
                    },
                    StoreKey {
                        key: vec![0.4, 0.6],
                    },
                    StoreKey {
                        key: vec![1.0, 0.2],
                    },
                ],
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    let dimensions_mismatch_err = ServerError::StoreDimensionMismatch {
        store_dimension: 2,
        input_dimension: 3,
    };

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: dimensions_mismatch_err.to_string(),
                    code: 3,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get { entries: vec![] },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![
                        DbStoreEntry {
                            key: Some(StoreKey {
                                key: vec![1.2, 0.3],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "title".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("elf".into())),
                                    },
                                )]),
                            }),
                        },
                        DbStoreEntry {
                            key: Some(StoreKey {
                                key: vec![1.0, 0.2],
                            }),
                            value: Some(StoreValue {
                                value: HashMap::from_iter([(
                                    "title".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "sorcerer".into(),
                                        )),
                                    },
                                )]),
                            }),
                        },
                    ],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());

    let response = client
        .info_server(tonic::Request::new(ahnlich_types::db::query::InfoServer {}))
        .await
        .expect("Failed call info server");

    let info_response = response.into_inner().info.unwrap();

    assert_eq!(info_response.address, socket_addr.to_string());
    assert_eq!(info_response.version, env!("CARGO_PKG_VERSION").to_string());
    assert_eq!(info_response.r#type, ServerType::Database as i32);
}

#[tokio::test]
async fn test_create_pred_index() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::CreatePredIndex(db_query_types::CreatePredIndex {
                store: "Main".to_string(),
                predicates: vec!["planet".into()],
                schema: None,
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec!["galaxy".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "andromeda".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "humanoid".into(),
                                        )),
                                    },
                                ),
                            ]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "milkyway".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("insects".into())),
                                    },
                                ),
                            ]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        // Should return 0 (no new indexes created)
        db_pipeline::DbQuery {
            query: Some(Query::CreatePredIndex(db_query_types::CreatePredIndex {
                store: "Main".to_string(),
                predicates: vec!["galaxy".into()],
                schema: None,
            })),
        },
        // Get with galaxy="milkyway"
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "galaxy".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("milkyway".into())),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        // Get with life-form="humanoid"
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "life-form".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("humanoid".into())),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        // Get with life-form IN ["insects"]
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::In(predicates::In {
                            key: "life-form".into(),
                            values: vec![MetadataValue {
                                value: Some(MetadataValueEnum::RawString("insects".into())),
                            }],
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        // Get with life-form NOT IN ["humanoid"]
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::NotIn(predicates::NotIn {
                            key: "life-form".into(),
                            values: vec![MetadataValue {
                                value: Some(MetadataValueEnum::RawString("humanoid".into())),
                            }],
                        })),
                    })),
                }),
                schema: None,
            })),
        },
        // Create indexes for technology and life-form (should return 2)
        db_pipeline::DbQuery {
            query: Some(Query::CreatePredIndex(db_query_types::CreatePredIndex {
                store: "Main".to_string(),
                predicates: vec!["technology".into(), "life-form".into(), "galaxy".into()],
                schema: None,
            })),
        },
        // Verify humanoid still works after indexing
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: "Main".to_string(),
                condition: Some(PredicateCondition {
                    kind: Some(PredicateConditionKind::Value(Predicate {
                        kind: Some(PredicateKind::Equals(predicates::Equals {
                            key: "life-form".into(),
                            value: Some(MetadataValue {
                                value: Some(MetadataValueEnum::RawString("humanoid".into())),
                            }),
                        })),
                    })),
                }),
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Set(
                db_response_types::Set {
                    upsert: Some(StoreUpsert {
                        inserted: 2,
                        updated: 0,
                    }),
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::CreateIndex(
                db_response_types::CreateIndex { created_indexes: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "milkyway".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("insects".into())),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "andromeda".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "humanoid".into(),
                                        )),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "milkyway".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("insects".into())),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.6, 1.7],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "milkyway".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString("insects".into())),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::CreateIndex(
                db_response_types::CreateIndex { created_indexes: 2 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Get(
                db_response_types::Get {
                    entries: vec![DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.4, 1.5],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([
                                (
                                    "galaxy".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "andromeda".into(),
                                        )),
                                    },
                                ),
                                (
                                    "life-form".into(),
                                    MetadataValue {
                                        value: Some(MetadataValueEnum::RawString(
                                            "humanoid".into(),
                                        )),
                                    },
                                ),
                            ]),
                        }),
                    }],
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_drop_pred_index() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should error as store does not exist
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: true,
                predicates: vec!["planet".into()],
                schema: None,
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec!["galaxy".into()],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Should not error (error_if_not_exists=false)
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: false,
                predicates: vec!["planet".into()],
                schema: None,
            })),
        },
        // Should error (predicate doesn't exist)
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: true,
                predicates: vec!["planet".into()],
                schema: None,
            })),
        },
        // Should succeed (galaxy predicate exists)
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: true,
                predicates: vec!["galaxy".into()],
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });
    let predicate_not_found_err = ServerError::PredicateNotFound("planet".into());

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: predicate_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_drop_stores() {
    // Server setup
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Build the pipeline request
    let queries = vec![
        // Should not error (error_if_not_exists=false)
        db_pipeline::DbQuery {
            query: Some(Query::DropStore(db_query_types::DropStore {
                store: "Main".to_string(),
                error_if_not_exists: false,
                schema: None,
            })),
        },
        // Create store
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 3,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // List stores
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
        // Should succeed (store exists)
        db_pipeline::DbQuery {
            query: Some(Query::DropStore(db_query_types::DropStore {
                store: "Main".to_string(),
                error_if_not_exists: true,
                schema: None,
            })),
        },
        // Should error (store doesn't exist)
        db_pipeline::DbQuery {
            query: Some(Query::DropStore(db_query_types::DropStore {
                store: "Main".to_string(),
                error_if_not_exists: true,
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let store_not_found_err = ServerError::StoreNotFound(StoreName {
        value: "Main".to_string(),
    });

    // Execute the request
    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    // Build expected responses
    let expected_responses = vec![
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 0 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Unit(
                db_response_types::Unit {},
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::StoreList(
                db_response_types::StoreList {
                    stores: vec![db_response_types::StoreInfo {
                        name: "Main".to_string(),
                        len: 0,
                        size_in_bytes: 1056,
                        non_linear_indices: vec![],
                        predicate_indices: vec![],
                        dimension: 3,
                    }],
                },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Del(
                db_response_types::Del { deleted_count: 1 },
            )),
        },
        db_pipeline::DbServerResponse {
            response: Some(db_pipeline::db_server_response::Response::Error(
                ahnlich_types::shared::info::ErrorResponse {
                    message: store_not_found_err.to_string(),
                    code: 5,
                },
            )),
        },
    ];

    let expected = db_pipeline::DbResponsePipeline {
        responses: expected_responses,
    };

    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_server_persistence_with_hnsw_index() {
    // Clean up - delete persistence file
    let _ = std::fs::remove_file(&*HNSW_PERSISTENCE_FILE);

    let server = Server::new(&CONFIG_WITH_HNSW_PERSISTENCE)
        .await
        .expect("Failed to create server");
    let write_flag = server.write_flag();
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // First set of operations: create store with HNSW index, insert data, query
    let queries = vec![
        // Create store with HNSW index
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "HnswStore".to_string(),
                dimension: 3,
                create_predicates: vec!["category".into()],
                non_linear_indices: vec![nonlinear::NonLinearIndex {
                    index: Some(non_linear_index::Index::Hnsw(HnswConfig::default())),
                }],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "HnswStore".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 2.0, 3.0],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "category".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("a".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![4.0, 5.0, 6.0],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "category".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("b".into())),
                                },
                            )]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        // Get similar items using HNSW
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "HnswStore".to_string(),
                closest_n: 1,
                algorithm: Algorithm::Hnsw.into(),
                search_input: Some(StoreKey {
                    key: vec![1.0, 2.0, 3.0],
                }),
                condition: None,
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;

    // CreateStore should succeed
    assert!(
        matches!(
            &responses[0].response,
            Some(db_pipeline::db_server_response::Response::Unit(_))
        ),
        "CreateStore failed: {:?}",
        responses[0]
    );
    // Set should succeed
    assert!(
        matches!(
            &responses[1].response,
            Some(db_pipeline::db_server_response::Response::Set(_))
        ),
        "Set failed: {:?}",
        responses[1]
    );
    // GetSimN should return results
    assert!(
        matches!(
            &responses[2].response,
            Some(db_pipeline::db_server_response::Response::GetSimN(_))
        ),
        "GetSimN failed: {:?}",
        responses[2]
    );

    assert!(write_flag.load(Ordering::SeqCst));

    // Wait for persistence
    tokio::time::sleep(Duration::from_millis(200)).await;

    // Second server instance - verify persistence
    let server = Server::new(&CONFIG_WITH_HNSW_PERSISTENCE)
        .await
        .expect("Failed to create server");
    let write_flag = server.write_flag();
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    // Verify persistence file exists and is not empty
    let file_metadata = std::fs::metadata(
        &CONFIG_WITH_HNSW_PERSISTENCE
            .common
            .persist_location
            .clone()
            .unwrap(),
    )
    .unwrap();
    assert!(file_metadata.len() > 0, "The persistence file is empty");

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(200)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Second set of operations to verify persistence
    let queries = vec![
        // Should error as store exists from persistence
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "HnswStore".to_string(),
                dimension: 3,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Should get persisted data
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "HnswStore".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 2.0, 3.0],
                }],
                schema: None,
            })),
        },
        // GetSimN should still work after deserialization
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "HnswStore".to_string(),
                closest_n: 1,
                algorithm: Algorithm::Hnsw.into(),
                search_input: Some(StoreKey {
                    key: vec![4.0, 5.0, 6.0],
                }),
                condition: None,
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;

    let already_exists_error = ServerError::StoreAlreadyExists(StoreName {
        value: "HnswStore".into(),
    });

    // CreateStore should error (already exists from persistence)
    assert_eq!(
        responses[0].response,
        Some(db_pipeline::db_server_response::Response::Error(
            ahnlich_types::shared::info::ErrorResponse {
                message: already_exists_error.to_string(),
                code: 6,
            },
        ))
    );

    // GetKey should return persisted data
    assert_eq!(
        responses[1].response,
        Some(db_pipeline::db_server_response::Response::Get(
            db_response_types::Get {
                entries: vec![DbStoreEntry {
                    key: Some(StoreKey {
                        key: vec![1.0, 2.0, 3.0],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::from_iter([(
                            "category".into(),
                            MetadataValue {
                                value: Some(MetadataValueEnum::RawString("a".into())),
                            },
                        )]),
                    }),
                }],
            },
        ))
    );

    // GetSimN should work after deserialization
    assert!(
        matches!(
            &responses[2].response,
            Some(db_pipeline::db_server_response::Response::GetSimN(_))
        ),
        "GetSimN after deserialization failed: {:?}",
        responses[2]
    );

    assert!(!write_flag.load(Ordering::SeqCst));

    // Clean up - delete persistence file
    let _ = std::fs::remove_file(&*HNSW_PERSISTENCE_FILE);
}

/// Test 1: Create a store with specific HNSW configuration and assert the config is applied.
#[tokio::test]
async fn test_create_store_with_hnsw_configuration() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let hnsw_config = HnswConfig {
        distance: Some(ahnlich_types::algorithm::algorithms::DistanceMetric::Euclidean as i32),
        ef_construction: Some(200),
        maximum_connections: Some(32),
        maximum_connections_zero: Some(64),
        extend_candidates: Some(true),
        keep_pruned_connections: Some(false),
    };

    let queries = vec![
        // Create store with specific HNSW configuration
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "HnswConfigStore".to_string(),
                dimension: 4,
                create_predicates: vec!["tag".into()],
                non_linear_indices: vec![nonlinear::NonLinearIndex {
                    index: Some(non_linear_index::Index::Hnsw(hnsw_config)),
                }],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "HnswConfigStore".into(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 0.0, 0.0, 0.0],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "tag".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("a".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![0.0, 1.0, 0.0, 0.0],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "tag".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("b".into())),
                                },
                            )]),
                        }),
                    },
                    DbStoreEntry {
                        key: Some(StoreKey {
                            key: vec![0.9, 0.1, 0.0, 0.0],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "tag".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::RawString("c".into())),
                                },
                            )]),
                        }),
                    },
                ],
                schema: None,
            })),
        },
        // Search using HNSW - nearest to [1.0, 0.0, 0.0, 0.0]
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: "HnswConfigStore".to_string(),
                closest_n: 2,
                algorithm: Algorithm::Hnsw.into(),
                search_input: Some(StoreKey {
                    key: vec![1.0, 0.0, 0.0, 0.0],
                }),
                condition: None,
                schema: None,
            })),
        },
        // List stores to verify config is reflected
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;

    // CreateStore should succeed
    assert!(
        matches!(
            &responses[0].response,
            Some(db_pipeline::db_server_response::Response::Unit(_))
        ),
        "CreateStore failed: {:?}",
        responses[0]
    );

    // Set should succeed with 3 inserts
    assert_eq!(
        responses[1].response,
        Some(db_pipeline::db_server_response::Response::Set(
            db_response_types::Set {
                upsert: Some(StoreUpsert {
                    inserted: 3,
                    updated: 0,
                }),
            },
        ))
    );

    // GetSimN should return results - nearest to [1,0,0,0] should be [1,0,0,0] and [0.9,0.1,0,0]
    if let Some(db_pipeline::db_server_response::Response::GetSimN(sim_result)) =
        &responses[2].response
    {
        assert_eq!(
            sim_result.entries.len(),
            2,
            "Should return 2 nearest neighbors"
        );
        // First result should be the exact match [1.0, 0.0, 0.0, 0.0]
        assert_eq!(
            sim_result.entries[0].key.as_ref().unwrap().key,
            vec![1.0, 0.0, 0.0, 0.0]
        );
        // Second should be the closest [0.9, 0.1, 0.0, 0.0]
        assert_eq!(
            sim_result.entries[1].key.as_ref().unwrap().key,
            vec![0.9, 0.1, 0.0, 0.0]
        );
    } else {
        panic!("GetSimN failed: {:?}", responses[2]);
    }

    // ListStores should return the store with HNSW config
    if let Some(db_pipeline::db_server_response::Response::StoreList(store_list)) =
        &responses[3].response
    {
        assert_eq!(store_list.stores.len(), 1);
        let store_info = &store_list.stores[0];
        assert_eq!(store_info.name, "HnswConfigStore");
        assert_eq!(store_info.len, 3);

        // Verify the non_linear_indices configuration is returned
        assert_eq!(store_info.non_linear_indices.len(), 1);
        let returned_index = &store_info.non_linear_indices[0];
        if let Some(non_linear_index::Index::Hnsw(returned_config)) = &returned_index.index {
            assert_eq!(
                returned_config.distance,
                Some(ahnlich_types::algorithm::algorithms::DistanceMetric::Euclidean as i32)
            );
            assert_eq!(returned_config.ef_construction, Some(200));
            assert_eq!(returned_config.maximum_connections, Some(32));
            assert_eq!(returned_config.maximum_connections_zero, Some(64));
            assert_eq!(returned_config.extend_candidates, Some(true));
            assert_eq!(returned_config.keep_pruned_connections, Some(false));
        } else {
            panic!(
                "Expected HNSW config in non_linear_indices, got: {:?}",
                returned_index
            );
        }
    } else {
        panic!("ListStores failed: {:?}", responses[3]);
    }
}

/// Test 2: Verify that a store cannot have the same nonlinear index type twice.
/// CreateNonLinearAlgorithmIndex is idempotent - adding an existing index returns 0 created.
#[tokio::test]
async fn test_duplicate_nonlinear_index_prevention() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let queries = vec![
        // Create store with HNSW index
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "DuplicateTest".to_string(),
                dimension: 3,
                create_predicates: vec![],
                non_linear_indices: vec![nonlinear::NonLinearIndex {
                    index: Some(non_linear_index::Index::Hnsw(HnswConfig::default())),
                }],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Try to create the same HNSW index again - should be idempotent, returns 0
        db_pipeline::DbQuery {
            query: Some(Query::CreateNonLinearAlgorithmIndex(
                db_query_types::CreateNonLinearAlgorithmIndex {
                    store: "DuplicateTest".to_string(),
                    non_linear_indices: vec![nonlinear::NonLinearIndex {
                        index: Some(non_linear_index::Index::Hnsw(HnswConfig {
                            ef_construction: Some(500),
                            maximum_connections: Some(100),
                            maximum_connections_zero: Some(200),
                            ..HnswConfig::default()
                        })),
                    }],
                    schema: None,
                },
            )),
        },
        // Try to create a KDTree index - should succeed (different type)
        db_pipeline::DbQuery {
            query: Some(Query::CreateNonLinearAlgorithmIndex(
                db_query_types::CreateNonLinearAlgorithmIndex {
                    store: "DuplicateTest".to_string(),
                    non_linear_indices: vec![nonlinear::NonLinearIndex {
                        index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
                    }],
                    schema: None,
                },
            )),
        },
        // Try to create both HNSW and KDTree again - both exist, should return 0
        db_pipeline::DbQuery {
            query: Some(Query::CreateNonLinearAlgorithmIndex(
                db_query_types::CreateNonLinearAlgorithmIndex {
                    store: "DuplicateTest".to_string(),
                    non_linear_indices: vec![
                        nonlinear::NonLinearIndex {
                            index: Some(non_linear_index::Index::Hnsw(HnswConfig::default())),
                        },
                        nonlinear::NonLinearIndex {
                            index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
                        },
                    ],
                    schema: None,
                },
            )),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;

    // CreateStore should succeed
    assert!(
        matches!(
            &responses[0].response,
            Some(db_pipeline::db_server_response::Response::Unit(_))
        ),
        "CreateStore failed: {:?}",
        responses[0]
    );

    // CreateNonLinearAlgorithmIndex with same HNSW should return 0 (already exists, not duplicated)
    assert_eq!(
        responses[1].response,
        Some(db_pipeline::db_server_response::Response::CreateIndex(
            db_response_types::CreateIndex { created_indexes: 0 },
        )),
        "Duplicate HNSW should not be created: {:?}",
        responses[1]
    );

    // CreateNonLinearAlgorithmIndex with KDTree should succeed (1 new index)
    assert_eq!(
        responses[2].response,
        Some(db_pipeline::db_server_response::Response::CreateIndex(
            db_response_types::CreateIndex { created_indexes: 1 },
        )),
        "KDTree index creation failed: {:?}",
        responses[2]
    );

    // CreateNonLinearAlgorithmIndex with both HNSW and KDTree - both exist, return 0
    assert_eq!(
        responses[3].response,
        Some(db_pipeline::db_server_response::Response::CreateIndex(
            db_response_types::CreateIndex { created_indexes: 0 },
        )),
        "Duplicate indices should not be created: {:?}",
        responses[3]
    );
}

/// Test 3: Demonstrate recall quality impact of HNSW configuration.
/// First creates a store with very low config showing poor recall,
/// then reconstructs with proper values showing improved recall.
#[tokio::test]
async fn test_hnsw_recall_with_config_reconstruction() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Generate a synthetic dataset: 200 vectors of dimension 16
    // Use deterministic pseudo-random generation for reproducibility
    let dimension = 16usize;
    let num_vectors = 200usize;
    let mut vectors: Vec<Vec<f32>> = Vec::with_capacity(num_vectors);
    for i in 0..num_vectors {
        let mut vec = Vec::with_capacity(dimension);
        for d in 0..dimension {
            // Simple deterministic generation using sine/cosine patterns
            let val = ((i as f32 * 0.1 + d as f32 * 0.3).sin() * 100.0).round() / 100.0;
            vec.push(val);
        }
        vectors.push(vec);
    }

    // Compute brute-force ground truth: top-10 nearest neighbors for a query vector
    let query_vec: Vec<f32> = (0..dimension)
        .map(|d| ((42.0_f32 * 0.1 + d as f32 * 0.3).sin() * 100.0).round() / 100.0)
        .collect();
    let k = 10usize;

    // Compute euclidean distances from query to all vectors
    let mut distances: Vec<(usize, f32)> = vectors
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let dist: f32 = v
                .iter()
                .zip(query_vec.iter())
                .map(|(a, b)| (a - b) * (a - b))
                .sum::<f32>()
                .sqrt();
            (i, dist)
        })
        .collect();
    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    let ground_truth_keys: Vec<Vec<f32>> = distances[..k]
        .iter()
        .map(|(i, _)| vectors[*i].clone())
        .collect();

    // === Phase 1: Create store with very low HNSW config ===
    let low_config = HnswConfig {
        distance: Some(ahnlich_types::algorithm::algorithms::DistanceMetric::Euclidean as i32),
        ef_construction: Some(5),
        maximum_connections: Some(2),
        maximum_connections_zero: Some(4),
        extend_candidates: Some(false),
        keep_pruned_connections: Some(false),
    };

    // Create store
    let create_response = client
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries: vec![db_pipeline::DbQuery {
                query: Some(Query::CreateStore(db_query_types::CreateStore {
                    store: "RecallTest".to_string(),
                    dimension: dimension as u32,
                    create_predicates: vec![],
                    non_linear_indices: vec![nonlinear::NonLinearIndex {
                        index: Some(non_linear_index::Index::Hnsw(low_config)),
                    }],
                    error_if_exists: true,
                    schema: None,
                })),
            }],
        }))
        .await
        .expect("Failed to create store");
    assert!(
        matches!(
            &create_response.into_inner().responses[0].response,
            Some(db_pipeline::db_server_response::Response::Unit(_))
        ),
        "CreateStore with low config failed"
    );

    // Insert all vectors
    let inputs: Vec<DbStoreEntry> = vectors
        .iter()
        .map(|v| DbStoreEntry {
            key: Some(StoreKey { key: v.clone() }),
            value: Some(StoreValue {
                value: HashMap::new(),
            }),
        })
        .collect();

    let set_response = client
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries: vec![db_pipeline::DbQuery {
                query: Some(Query::Set(db_query_types::Set {
                    store: "RecallTest".into(),
                    inputs,
                    schema: None,
                })),
            }],
        }))
        .await
        .expect("Failed to insert data");
    assert!(
        matches!(
            &set_response.into_inner().responses[0].response,
            Some(db_pipeline::db_server_response::Response::Set(_))
        ),
        "Set failed"
    );

    // Query with HNSW using low config
    let low_config_response = client
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries: vec![db_pipeline::DbQuery {
                query: Some(Query::GetSimN(db_query_types::GetSimN {
                    store: "RecallTest".to_string(),
                    closest_n: k as u64,
                    algorithm: Algorithm::Hnsw.into(),
                    search_input: Some(StoreKey {
                        key: query_vec.clone(),
                    }),
                    condition: None,
                    schema: None,
                })),
            }],
        }))
        .await
        .expect("Failed to query");

    let low_config_results = &low_config_response.into_inner().responses[0];
    let low_recall = if let Some(db_pipeline::db_server_response::Response::GetSimN(sim_result)) =
        &low_config_results.response
    {
        let returned_keys: Vec<Vec<f32>> = sim_result
            .entries
            .iter()
            .map(|e| e.key.as_ref().unwrap().key.clone())
            .collect();
        let overlap = ground_truth_keys
            .iter()
            .filter(|gt| returned_keys.contains(gt))
            .count();
        overlap as f32 / k as f32
    } else {
        panic!("GetSimN with low config failed: {:?}", low_config_results);
    };

    // === Phase 2: Reconstruct - remove the low config index and recreate with proper config ===

    // Drop the HNSW index
    let drop_response = client
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries: vec![db_pipeline::DbQuery {
                query: Some(Query::DropNonLinearAlgorithmIndex(
                    db_query_types::DropNonLinearAlgorithmIndex {
                        store: "RecallTest".to_string(),
                        non_linear_indices: vec![NonLinearAlgorithm::Hnsw as i32],
                        error_if_not_exists: true,
                        schema: None,
                    },
                )),
            }],
        }))
        .await
        .expect("Failed to drop index");
    assert!(
        matches!(
            &drop_response.into_inner().responses[0].response,
            Some(db_pipeline::db_server_response::Response::Del(_))
        ),
        "Drop index failed"
    );

    // Recreate HNSW index with proper configuration
    let good_config = HnswConfig {
        distance: Some(ahnlich_types::algorithm::algorithms::DistanceMetric::Euclidean as i32),
        ef_construction: Some(100),
        maximum_connections: Some(48),
        maximum_connections_zero: Some(96),
        extend_candidates: Some(false),
        keep_pruned_connections: Some(false),
    };

    let recreate_response = client
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries: vec![db_pipeline::DbQuery {
                query: Some(Query::CreateNonLinearAlgorithmIndex(
                    db_query_types::CreateNonLinearAlgorithmIndex {
                        store: "RecallTest".to_string(),
                        non_linear_indices: vec![nonlinear::NonLinearIndex {
                            index: Some(non_linear_index::Index::Hnsw(good_config)),
                        }],
                        schema: None,
                    },
                )),
            }],
        }))
        .await
        .expect("Failed to recreate index");
    assert!(
        matches!(
            &recreate_response.into_inner().responses[0].response,
            Some(db_pipeline::db_server_response::Response::CreateIndex(_))
        ),
        "Recreate index failed"
    );

    // Query with HNSW using good config
    let good_config_response = client
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries: vec![db_pipeline::DbQuery {
                query: Some(Query::GetSimN(db_query_types::GetSimN {
                    store: "RecallTest".to_string(),
                    closest_n: k as u64,
                    algorithm: Algorithm::Hnsw.into(),
                    search_input: Some(StoreKey {
                        key: query_vec.clone(),
                    }),
                    condition: None,
                    schema: None,
                })),
            }],
        }))
        .await
        .expect("Failed to query with good config");

    let good_config_results = &good_config_response.into_inner().responses[0];
    let good_recall = if let Some(db_pipeline::db_server_response::Response::GetSimN(sim_result)) =
        &good_config_results.response
    {
        let returned_keys: Vec<Vec<f32>> = sim_result
            .entries
            .iter()
            .map(|e| e.key.as_ref().unwrap().key.clone())
            .collect();
        let overlap = ground_truth_keys
            .iter()
            .filter(|gt| returned_keys.contains(gt))
            .count();
        overlap as f32 / k as f32
    } else {
        panic!("GetSimN with good config failed: {:?}", good_config_results);
    };

    // Assert that the good config has better or equal recall than the low config
    assert!(
        good_recall >= low_recall,
        "Good config recall ({}) should be >= low config recall ({})",
        good_recall,
        low_recall
    );

    // Assert that the good config achieves high recall (>= 90%)
    assert!(
        good_recall >= 0.9,
        "Good config should achieve at least 90% recall, got {}",
        good_recall
    );
}

/// Test 4: Verify that ListStores returns the HNSW configuration for stores with nonlinear indices.
#[tokio::test]
async fn test_list_stores_returns_nonlinear_config() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let hnsw_config = HnswConfig {
        distance: Some(ahnlich_types::algorithm::algorithms::DistanceMetric::Cosine as i32),
        ef_construction: Some(150),
        maximum_connections: Some(24),
        maximum_connections_zero: Some(48),
        extend_candidates: Some(true),
        keep_pruned_connections: Some(true),
    };

    let queries = vec![
        // Create store with both HNSW and KDTree indices
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "ConfigStore".to_string(),
                dimension: 3,
                create_predicates: vec![],
                non_linear_indices: vec![
                    nonlinear::NonLinearIndex {
                        index: Some(non_linear_index::Index::Hnsw(hnsw_config)),
                    },
                    nonlinear::NonLinearIndex {
                        index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
                    },
                ],
                error_if_exists: true,
                schema: None,
            })),
        },
        // Create a store without nonlinear indices
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "PlainStore".to_string(),
                dimension: 3,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        // List stores
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;

    // Both creates should succeed
    assert!(
        matches!(
            &responses[0].response,
            Some(db_pipeline::db_server_response::Response::Unit(_))
        ),
        "CreateStore ConfigStore failed: {:?}",
        responses[0]
    );
    assert!(
        matches!(
            &responses[1].response,
            Some(db_pipeline::db_server_response::Response::Unit(_))
        ),
        "CreateStore PlainStore failed: {:?}",
        responses[1]
    );

    // ListStores should contain both stores
    if let Some(db_pipeline::db_server_response::Response::StoreList(store_list)) =
        &responses[2].response
    {
        assert_eq!(store_list.stores.len(), 2);

        // Find ConfigStore and PlainStore (stores are sorted by name)
        let config_store = store_list
            .stores
            .iter()
            .find(|s| s.name == "ConfigStore")
            .expect("ConfigStore not found in list");
        let plain_store = store_list
            .stores
            .iter()
            .find(|s| s.name == "PlainStore")
            .expect("PlainStore not found in list");

        // PlainStore should have no nonlinear indices
        assert!(
            plain_store.non_linear_indices.is_empty(),
            "PlainStore should have no non_linear_indices, got: {:?}",
            plain_store.non_linear_indices
        );

        // ConfigStore should have 2 nonlinear indices (HNSW + KDTree)
        assert_eq!(
            config_store.non_linear_indices.len(),
            2,
            "ConfigStore should have 2 non_linear_indices"
        );

        // Find the HNSW config in the indices
        let hnsw_index = config_store
            .non_linear_indices
            .iter()
            .find(|idx| matches!(idx.index, Some(non_linear_index::Index::Hnsw(_))))
            .expect("HNSW index not found in ConfigStore");

        if let Some(non_linear_index::Index::Hnsw(returned_config)) = &hnsw_index.index {
            assert_eq!(
                returned_config.distance,
                Some(ahnlich_types::algorithm::algorithms::DistanceMetric::Cosine as i32),
                "Distance metric mismatch"
            );
            assert_eq!(
                returned_config.ef_construction,
                Some(150),
                "ef_construction mismatch"
            );
            assert_eq!(
                returned_config.maximum_connections,
                Some(24),
                "maximum_connections mismatch"
            );
            assert_eq!(
                returned_config.maximum_connections_zero,
                Some(48),
                "maximum_connections_zero mismatch"
            );
            assert_eq!(
                returned_config.extend_candidates,
                Some(true),
                "extend_candidates mismatch"
            );
            assert_eq!(
                returned_config.keep_pruned_connections,
                Some(true),
                "keep_pruned_connections mismatch"
            );
        } else {
            panic!("Expected HNSW config");
        }

        // Verify KDTree index exists
        let kdtree_index = config_store
            .non_linear_indices
            .iter()
            .find(|idx| matches!(idx.index, Some(non_linear_index::Index::Kdtree(_))))
            .expect("KDTree index not found in ConfigStore");
        assert!(
            matches!(kdtree_index.index, Some(non_linear_index::Index::Kdtree(_))),
            "Expected KDTree config"
        );
    } else {
        panic!("ListStores failed: {:?}", responses[2]);
    }
}

/// Test: GetStore returns not_found error for non-existent store
#[tokio::test]
async fn test_get_store_not_found() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let result = client
        .get_store(tonic::Request::new(db_query_types::GetStore {
            store: "NonExistent".to_string(),
            schema: None,
        }))
        .await;

    assert!(result.is_err());
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::NotFound);
}

/// Test: GetStore returns correct StoreInfo for an existing store
#[tokio::test]
async fn test_get_store_success() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Create a store with predicates and a nonlinear index
    let hnsw_config = HnswConfig {
        distance: Some(ahnlich_types::algorithm::algorithms::DistanceMetric::Cosine as i32),
        ef_construction: Some(150),
        maximum_connections: Some(24),
        maximum_connections_zero: Some(48),
        extend_candidates: Some(true),
        keep_pruned_connections: Some(true),
    };

    let create_req = db_query_types::CreateStore {
        store: "TestGetStore".to_string(),
        dimension: 5,
        create_predicates: vec!["author".to_string(), "category".to_string()],
        non_linear_indices: vec![nonlinear::NonLinearIndex {
            index: Some(non_linear_index::Index::Hnsw(hnsw_config)),
        }],
        error_if_exists: true,
        schema: None,
    };

    client
        .create_store(tonic::Request::new(create_req))
        .await
        .expect("CreateStore failed");

    // Insert some data
    let set_req = db_query_types::Set {
        store: "TestGetStore".to_string(),
        inputs: vec![DbStoreEntry {
            key: Some(StoreKey {
                key: vec![1.0, 2.0, 3.0, 4.0, 5.0],
            }),
            value: Some(StoreValue {
                value: HashMap::from_iter([(
                    "author".to_string(),
                    MetadataValue {
                        value: Some(MetadataValueEnum::RawString("alice".to_string())),
                    },
                )]),
            }),
        }],
        schema: None,
    };

    client
        .set(tonic::Request::new(set_req))
        .await
        .expect("Set failed");

    // GetStore
    let store_info = client
        .get_store(tonic::Request::new(db_query_types::GetStore {
            store: "TestGetStore".to_string(),
            schema: None,
        }))
        .await
        .expect("GetStore failed")
        .into_inner();

    assert_eq!(store_info.name, "TestGetStore");
    assert_eq!(store_info.len, 1);
    assert_eq!(store_info.dimension, 5);
    assert_eq!(
        store_info.predicate_indices,
        vec!["author".to_string(), "category".to_string()]
    );
    assert_eq!(store_info.non_linear_indices.len(), 1);
    assert!(store_info.size_in_bytes > 0);
}

/// Test: GetStore works within a pipeline
#[tokio::test]
async fn test_get_store_in_pipeline() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let queries = vec![
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "PipelineStore".to_string(),
                dimension: 3,
                create_predicates: vec!["tag".to_string()],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::GetStore(db_query_types::GetStore {
                store: "PipelineStore".to_string(),
                schema: None,
            })),
        },
        // GetStore on non-existent store should return error in pipeline
        db_pipeline::DbQuery {
            query: Some(Query::GetStore(db_query_types::GetStore {
                store: "DoesNotExist".to_string(),
                schema: None,
            })),
        },
    ];

    let pipelined_request = db_pipeline::DbRequestPipeline { queries };

    let response = client
        .pipeline(tonic::Request::new(pipelined_request))
        .await
        .expect("Failed to execute pipeline");

    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 3);

    // CreateStore should succeed
    assert!(
        matches!(
            &responses[0].response,
            Some(db_pipeline::db_server_response::Response::Unit(_))
        ),
        "CreateStore failed: {:?}",
        responses[0]
    );

    // GetStore should return StoreInfo
    if let Some(db_pipeline::db_server_response::Response::StoreInfo(store_info)) =
        &responses[1].response
    {
        assert_eq!(store_info.name, "PipelineStore");
        assert_eq!(store_info.len, 0);
        assert_eq!(store_info.dimension, 3);
        assert_eq!(store_info.predicate_indices, vec!["tag".to_string()]);
        assert!(store_info.non_linear_indices.is_empty());
    } else {
        panic!("GetStore in pipeline failed: {:?}", responses[1]);
    }

    // GetStore on non-existent store should return error
    assert!(
        matches!(
            &responses[2].response,
            Some(db_pipeline::db_server_response::Response::Error(_))
        ),
        "Expected error for non-existent store, got: {:?}",
        responses[2]
    );
}

#[tokio::test]
async fn test_mmap_persistence_performance() {
    let mmap_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ahnlich_mmap_test.dat");
    let no_mmap_file = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ahnlich_no_mmap_test.dat");

    let _ = std::fs::remove_file(&mmap_file);
    let _ = std::fs::remove_file(&no_mmap_file);

    // Create a large store with lots of vectors to ensure file > 64KB threshold
    // Vector dimension: 128, Number of vectors: 1000 = ~500KB file
    let dimension = 128;
    let num_vectors = 1000;

    // First, create and persist a large store
    let config_with_mmap = ServerConfig::default()
        .os_select_port()
        .persistence_interval(200)
        .persist_location(mmap_file.clone());

    let server = Server::new(&config_with_mmap)
        .await
        .expect("Failed to create server");
    let write_flag = server.write_flag();
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Create store
    let create_query = db_pipeline::DbQuery {
        query: Some(Query::CreateStore(db_query_types::CreateStore {
            store: "LargeStore".to_string(),
            dimension,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: false,
            schema: None,
        })),
    };

    client
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries: vec![create_query],
        }))
        .await
        .expect("Failed to create store");

    // Insert many vectors in batches to create a large file
    let batch_size = 100;
    for batch_start in (0..num_vectors).step_by(batch_size) {
        let batch_end = (batch_start + batch_size).min(num_vectors);
        let mut inputs = Vec::new();

        for i in batch_start..batch_end {
            let key: Vec<f32> = (0..dimension)
                .map(|j| ((i * dimension as usize + j as usize) as f32) * 0.01)
                .collect();
            inputs.push(DbStoreEntry {
                key: Some(StoreKey { key }),
                value: Some(StoreValue {
                    value: HashMap::new(),
                }),
            });
        }

        let set_query = db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "LargeStore".into(),
                inputs,
                schema: None,
            })),
        };

        client
            .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
                queries: vec![set_query],
            }))
            .await
            .expect("Failed to insert batch");
    }

    // Trigger persistence by setting write flag and waiting
    write_flag.store(true, Ordering::SeqCst);
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Verify the file was created and check its size
    let file_metadata = std::fs::metadata(&mmap_file).expect("Persistence file not created");
    println!(
        "Persistence file size: {} KB ({} bytes)",
        file_metadata.len() / 1024,
        file_metadata.len()
    );

    // The file should be larger than the mmap threshold (64KB)
    assert!(
        file_metadata.len() > 64 * 1024,
        "File size ({} bytes) should be > 64KB for meaningful mmap test",
        file_metadata.len()
    );

    // Copy the file for the no-mmap test
    std::fs::copy(&mmap_file, &no_mmap_file).expect("Failed to copy persistence file");

    // Now test loading with mmap enabled (default)
    let config_with_mmap = ServerConfig::default()
        .os_select_port()
        .persist_location(mmap_file.clone());

    let server_mmap = Server::new(&config_with_mmap)
        .await
        .expect("Failed to create server with mmap");

    // Verify the store was loaded correctly
    let address = server_mmap.local_addr().expect("Could not get local addr");
    tokio::spawn(async move { server_mmap.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client_mmap = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let list_query = db_pipeline::DbQuery {
        query: Some(Query::ListStores(db_query_types::ListStores {
            schema: None,
        })),
    };

    let response = client_mmap
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries: vec![list_query],
        }))
        .await
        .expect("Failed to list stores");

    // Verify store exists
    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 1);
    if let Some(db_pipeline::db_server_response::Response::StoreList(store_list)) =
        &responses[0].response
    {
        assert_eq!(store_list.stores.len(), 1);
        assert_eq!(store_list.stores[0].name, "LargeStore");
    } else {
        panic!("Expected StoreList response");
    }

    // Now test loading with mmap disabled
    let config_no_mmap = ServerConfig::default()
        .os_select_port()
        .persist_location(no_mmap_file.clone())
        .disable_mmap();

    let server_no_mmap = Server::new(&config_no_mmap)
        .await
        .expect("Failed to create server without mmap");

    // Verify the store was loaded correctly
    let address = server_no_mmap
        .local_addr()
        .expect("Could not get local addr");
    tokio::spawn(async move { server_no_mmap.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client_no_mmap = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let list_query = db_pipeline::DbQuery {
        query: Some(Query::ListStores(db_query_types::ListStores {
            schema: None,
        })),
    };

    let response = client_no_mmap
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries: vec![list_query],
        }))
        .await
        .expect("Failed to list stores");

    // Verify store exists
    let responses = response.into_inner().responses;
    assert_eq!(responses.len(), 1);
    if let Some(db_pipeline::db_server_response::Response::StoreList(store_list)) =
        &responses[0].response
    {
        assert_eq!(store_list.stores.len(), 1);
        assert_eq!(store_list.stores[0].name, "LargeStore");
    } else {
        panic!("Expected StoreList response");
    }

    // Clean up
    let _ = std::fs::remove_file(&mmap_file);
    let _ = std::fs::remove_file(&no_mmap_file);
}

/// Test: Create stores in different schemas and list them with schema filtering
#[tokio::test]
async fn test_schema_create_and_list_in_schema() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Create a store in default schema
    client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "PublicStore".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: None,
        }))
        .await
        .expect("CreateStore in default schema failed");

    // Create a store in custom schema
    client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "CustomStore".to_string(),
            dimension: 5,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: Some("custom".to_string()),
        }))
        .await
        .expect("CreateStore in custom schema failed");

    // Create another store in custom schema
    client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "CustomStore2".to_string(),
            dimension: 7,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: Some("custom".to_string()),
        }))
        .await
        .expect("Create second store in custom schema failed");

    // List stores filtered by public schema
    let response = client
        .list_stores(tonic::Request::new(db_query_types::ListStores {
            schema: Some("public".to_string()),
        }))
        .await
        .expect("ListStores with public schema failed")
        .into_inner();
    assert_eq!(response.stores.len(), 1);
    assert_eq!(response.stores[0].name, "PublicStore");

    // List stores filtered by custom schema
    let response = client
        .list_stores(tonic::Request::new(db_query_types::ListStores {
            schema: Some("custom".to_string()),
        }))
        .await
        .expect("ListStores with custom schema failed")
        .into_inner();
    assert_eq!(response.stores.len(), 2);
    let names: Vec<&str> = response.stores.iter().map(|s| s.name.as_str()).collect();
    assert!(names.contains(&"CustomStore"));
    assert!(names.contains(&"CustomStore2"));

    // List stores with no schema filter - should default to public schema
    let response = client
        .list_stores(tonic::Request::new(db_query_types::ListStores {
            schema: None,
        }))
        .await
        .expect("ListStores without schema filter failed")
        .into_inner();
    assert_eq!(response.stores.len(), 1);
    assert_eq!(response.stores[0].name, "PublicStore");
}

/// Test: GetStore with schema parameter
#[tokio::test]
async fn test_schema_get_store_in_schema() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Create store in custom schema
    client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "SchemaGetStore".to_string(),
            dimension: 4,
            create_predicates: vec!["tag".to_string()],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: Some("myschema".to_string()),
        }))
        .await
        .expect("CreateStore failed");

    // GetStore with schema specified
    let store_info = client
        .get_store(tonic::Request::new(db_query_types::GetStore {
            store: "SchemaGetStore".to_string(),
            schema: Some("myschema".to_string()),
        }))
        .await
        .expect("GetStore with schema failed")
        .into_inner();

    assert_eq!(store_info.name, "SchemaGetStore");
    assert_eq!(store_info.dimension, 4);
    assert_eq!(store_info.predicate_indices, vec!["tag".to_string()]);

    // GetStore without schema should NOT find it (defaults to public schema only)
    let result = client
        .get_store(tonic::Request::new(db_query_types::GetStore {
            store: "SchemaGetStore".to_string(),
            schema: None,
        }))
        .await;
    assert!(
        result.is_err(),
        "GetStore without schema should fail for stores in non-public schema"
    );

    assert_eq!(store_info.name, "SchemaGetStore");
    assert_eq!(store_info.dimension, 4);
}

/// Test: DropStore with schema parameter
#[tokio::test]
async fn test_schema_drop_store_in_schema() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Create store in custom schema
    client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "DropInSchema".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: Some("dropschema".to_string()),
        }))
        .await
        .expect("CreateStore failed");

    // Verify store exists via GetStore
    client
        .get_store(tonic::Request::new(db_query_types::GetStore {
            store: "DropInSchema".to_string(),
            schema: Some("dropschema".to_string()),
        }))
        .await
        .expect("GetStore should succeed before drop");

    // Drop store with schema specified
    client
        .drop_store(tonic::Request::new(db_query_types::DropStore {
            store: "DropInSchema".to_string(),
            error_if_not_exists: true,
            schema: Some("dropschema".to_string()),
        }))
        .await
        .expect("DropStore failed");

    // Verify store is gone
    let result = client
        .get_store(tonic::Request::new(db_query_types::GetStore {
            store: "DropInSchema".to_string(),
            schema: Some("dropschema".to_string()),
        }))
        .await;

    assert!(result.is_err(), "GetStore should fail after drop");
}

/// Test: DropSchema to remove an entire non-public schema
#[tokio::test]
async fn test_schema_drop_schema() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Create two stores in a schema to drop
    client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "DropSchemaStore1".to_string(),
            dimension: 3,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: Some("tobedropped".to_string()),
        }))
        .await
        .expect("CreateStore 1 failed");

    client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "DropSchemaStore2".to_string(),
            dimension: 5,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: Some("tobedropped".to_string()),
        }))
        .await
        .expect("CreateStore 2 failed");

    // Also create a store in public schema to verify isolation
    client
        .create_store(tonic::Request::new(db_query_types::CreateStore {
            store: "PublicSurvivor".to_string(),
            dimension: 2,
            create_predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: None,
        }))
        .await
        .expect("CreateStore in public failed");

    // Drop the schema
    let response = client
        .drop_schema(tonic::Request::new(db_query_types::DropSchema {
            schema: "tobedropped".to_string(),
        }))
        .await
        .expect("DropSchema failed")
        .into_inner();
    assert_eq!(response.deleted_count, 2);

    // Verify stores in dropped schema are gone
    let result = client
        .get_store(tonic::Request::new(db_query_types::GetStore {
            store: "DropSchemaStore1".to_string(),
            schema: Some("tobedropped".to_string()),
        }))
        .await;
    assert!(result.is_err(), "Store in dropped schema should be gone");

    // Verify public schema store still exists
    let pub_store = client
        .get_store(tonic::Request::new(db_query_types::GetStore {
            store: "PublicSurvivor".to_string(),
            schema: None,
        }))
        .await
        .expect("Public store should still exist")
        .into_inner();
    assert_eq!(pub_store.name, "PublicSurvivor");

    // Verify list stores filtered by public schema shows only the survivor
    let response = client
        .list_stores(tonic::Request::new(db_query_types::ListStores {
            schema: Some("public".to_string()),
        }))
        .await
        .expect("ListStores failed")
        .into_inner();
    assert_eq!(response.stores.len(), 1);
    assert_eq!(response.stores[0].name, "PublicSurvivor");
}

/// Test: Dropping the "public" schema should fail
#[tokio::test]
async fn test_schema_drop_public_schema_fails() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    // Attempt to drop "public" schema
    let result = client
        .drop_schema(tonic::Request::new(db_query_types::DropSchema {
            schema: "public".to_string(),
        }))
        .await;

    assert!(
        result.is_err(),
        "Dropping public schema should return an error"
    );
    let status = result.unwrap_err();
    assert_eq!(status.code(), tonic::Code::InvalidArgument);
    assert!(
        status.message().contains("public"),
        "Error message should reference 'public': {}",
        status.message()
    );
}

#[tokio::test]
async fn test_schema_store_commands_use_custom_schema() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move { server.start().await });

    let address = format!("http://{}", address);
    tokio::time::sleep(Duration::from_millis(100)).await;
    let channel = Channel::from_shared(address).expect("Failed to get channel");
    let mut client = DbServiceClient::connect(channel)
        .await
        .expect("Failed to connect");

    let schema = Some("db_store_commands".to_string());
    let store_name = "DbSchemaCommandStore".to_string();
    let matching_metadatakey = "Brand".to_string();
    let nike_value = MetadataValue {
        value: Some(MetadataValueEnum::RawString("Nike".into())),
    };
    let adidas_value = MetadataValue {
        value: Some(MetadataValueEnum::RawString("Adidas".into())),
    };
    let nike_store_value = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), nike_value.clone())]),
    };
    let adidas_store_value = StoreValue {
        value: HashMap::from_iter([(matching_metadatakey.clone(), adidas_value.clone())]),
    };
    let jordan_key = StoreKey {
        key: vec![1.0, 1.1, 1.2],
    };
    let yeezy_key = StoreKey {
        key: vec![2.0, 2.1, 2.2],
    };
    let condition_nike = PredicateCondition {
        kind: Some(PredicateConditionKind::Value(Predicate {
            kind: Some(PredicateKind::Equals(predicates::Equals {
                key: matching_metadatakey.clone(),
                value: Some(nike_value),
            })),
        })),
    };
    let condition_adidas = PredicateCondition {
        kind: Some(PredicateConditionKind::Value(Predicate {
            kind: Some(PredicateKind::Equals(predicates::Equals {
                key: matching_metadatakey.clone(),
                value: Some(adidas_value),
            })),
        })),
    };

    let queries = vec![
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: store_name.clone(),
                dimension: 3,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::GetStore(db_query_types::GetStore {
                store: store_name.clone(),
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {
                schema: None,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::CreatePredIndex(db_query_types::CreatePredIndex {
                store: store_name.clone(),
                predicates: vec![matching_metadatakey.clone()],
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::CreateNonLinearAlgorithmIndex(
                db_query_types::CreateNonLinearAlgorithmIndex {
                    store: store_name.clone(),
                    non_linear_indices: vec![nonlinear::NonLinearIndex {
                        index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
                    }],
                    schema: schema.clone(),
                },
            )),
        },
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: store_name.clone(),
                inputs: vec![
                    DbStoreEntry {
                        key: Some(jordan_key.clone()),
                        value: Some(nike_store_value.clone()),
                    },
                    DbStoreEntry {
                        key: Some(yeezy_key.clone()),
                        value: Some(adidas_store_value.clone()),
                    },
                ],
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: store_name.clone(),
                keys: vec![jordan_key.clone()],
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::GetPred(db_query_types::GetPred {
                store: store_name.clone(),
                condition: Some(condition_nike.clone()),
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::GetSimN(db_query_types::GetSimN {
                store: store_name.clone(),
                search_input: Some(jordan_key.clone()),
                closest_n: 1,
                algorithm: Algorithm::DotProductSimilarity.into(),
                condition: Some(condition_nike.clone()),
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::DelPred(db_query_types::DelPred {
                store: store_name.clone(),
                condition: Some(condition_adidas),
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: store_name.clone(),
                keys: vec![jordan_key.clone()],
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: store_name.clone(),
                predicates: vec![matching_metadatakey.clone()],
                error_if_not_exists: true,
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::DropNonLinearAlgorithmIndex(
                db_query_types::DropNonLinearAlgorithmIndex {
                    store: store_name.clone(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree.into()],
                    error_if_not_exists: true,
                    schema: schema.clone(),
                },
            )),
        },
        db_pipeline::DbQuery {
            query: Some(Query::DropStore(db_query_types::DropStore {
                store: store_name.clone(),
                error_if_not_exists: true,
                schema: schema.clone(),
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores { schema })),
        },
    ];

    let result = client
        .pipeline(tonic::Request::new(db_pipeline::DbRequestPipeline {
            queries,
        }))
        .await
        .expect("Failed to send pipeline request")
        .into_inner();

    assert_eq!(result.responses.len(), 15);
    assert!(matches!(
        &result.responses[0].response,
        Some(db_pipeline::db_server_response::Response::Unit(_))
    ));
    assert!(matches!(
        &result.responses[1].response,
        Some(db_pipeline::db_server_response::Response::StoreInfo(info))
            if info.name == store_name
    ));
    assert!(matches!(
        &result.responses[2].response,
        Some(db_pipeline::db_server_response::Response::StoreList(list)) if list.stores.is_empty()
    ));
    assert!(matches!(
        &result.responses[3].response,
        Some(db_pipeline::db_server_response::Response::CreateIndex(index)) if index.created_indexes == 1
    ));
    assert!(matches!(
        &result.responses[4].response,
        Some(db_pipeline::db_server_response::Response::CreateIndex(index)) if index.created_indexes == 1
    ));
    assert!(matches!(
        &result.responses[5].response,
        Some(db_pipeline::db_server_response::Response::Set(set))
            if set.upsert == Some(StoreUpsert { inserted: 2, updated: 0 })
    ));
    assert!(matches!(
        &result.responses[6].response,
        Some(db_pipeline::db_server_response::Response::Get(get))
            if get.entries.len() == 1 && get.entries[0].key == Some(jordan_key.clone())
    ));
    assert!(matches!(
        &result.responses[7].response,
        Some(db_pipeline::db_server_response::Response::Get(get))
            if get.entries.len() == 1 && get.entries[0].value == Some(nike_store_value.clone())
    ));
    assert!(matches!(
        &result.responses[8].response,
        Some(db_pipeline::db_server_response::Response::GetSimN(get_sim_n))
            if get_sim_n.entries.len() == 1
    ));
    assert!(matches!(
        &result.responses[9].response,
        Some(db_pipeline::db_server_response::Response::Del(del)) if del.deleted_count == 1
    ));
    assert!(matches!(
        &result.responses[10].response,
        Some(db_pipeline::db_server_response::Response::Del(del)) if del.deleted_count == 1
    ));
    assert!(matches!(
        &result.responses[11].response,
        Some(db_pipeline::db_server_response::Response::Del(del)) if del.deleted_count == 1
    ));
    assert!(matches!(
        &result.responses[12].response,
        Some(db_pipeline::db_server_response::Response::Del(del)) if del.deleted_count == 1
    ));
    assert!(matches!(
        &result.responses[13].response,
        Some(db_pipeline::db_server_response::Response::Del(del)) if del.deleted_count == 1
    ));
    assert!(matches!(
        &result.responses[14].response,
        Some(db_pipeline::db_server_response::Response::StoreList(list)) if list.stores.is_empty()
    ));
}

#[test]
fn test_migrate_old_flat_snapshot() {
    // Create a store handler and populate a store under "public"
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let store_name = StoreName {
        value: "test_store".to_string(),
    };
    handler
        .create_store(
            store_name.clone(),
            &Schema::default(),
            NonZeroUsize::new(3).unwrap(),
            vec![],
            std::collections::HashSet::new(),
            true,
        )
        .expect("Failed to create store");

    // Serialize the inner stores (under "public") as the old flat format
    let stores = handler.get_stores();
    let guard = stores.guard();
    let inner = stores
        .get(&Schema::default(), &guard)
        .expect("No public schema");
    let pinned = inner.pin();
    let old_format_bytes = serde_json::to_vec(&pinned).expect("Failed to serialize old format");

    // Now simulate loading this old-format snapshot via migration
    let migrated = StoreHandler::load_snapshot(&old_format_bytes).expect("Migration failed");

    // Verify: migrated stores should contain the store under "public"
    let migrated_guard = migrated.guard();
    let migrated_inner = migrated
        .get(&Schema::default(), &migrated_guard)
        .expect("No public schema after migration");
    assert_eq!(
        migrated_inner.len(),
        1,
        "Expected 1 store under public schema"
    );
    let migrated_pinned = migrated_inner.pin();
    let (_key, _store) = migrated_pinned.iter().next().expect("No store in result");
}

#[test]
fn test_migrate_old_flat_snapshot_json_file() {
    // Create a real store handler and serialize its inner public stores as old-format JSON
    let handler = StoreHandler::new(Arc::new(AtomicBool::new(false)));
    let store_name = StoreName {
        value: "fixture_store".to_string(),
    };
    handler
        .create_store(
            store_name.clone(),
            &Schema::default(),
            NonZeroUsize::new(3).unwrap(),
            vec![],
            std::collections::HashSet::new(),
            true,
        )
        .expect("Failed to create store");

    // Get the flat (old-format) JSON: the inner HashMap under "public"
    let stores = handler.get_stores();
    let guard = stores.guard();
    let inner = stores
        .get(&Schema::default(), &guard)
        .expect("No public schema");
    let pinned = inner.pin();
    let json_bytes = serde_json::to_vec_pretty(&pinned).expect("Failed to serialize");

    // Write to fixture file in tests/fixtures/
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures");
    std::fs::create_dir_all(&fixture_dir).expect("Failed to create fixtures dir");
    let fixture_path = fixture_dir.join("db_old_flat_snapshot.json");
    std::fs::write(&fixture_path, &json_bytes).expect("Failed to write fixture");

    // Now read it back and verify it migrates correctly
    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");
    let migrated = StoreHandler::load_snapshot(&read_bytes).expect("Migration of fixture failed");

    let migrated_guard = migrated.guard();
    let migrated_inner = migrated
        .get(&Schema::default(), &migrated_guard)
        .expect("No public schema after migration");
    assert_eq!(
        migrated_inner.len(),
        1,
        "Expected 1 store under public schema"
    );
}

#[test]
fn test_migrate_from_committed_db_fixture() {
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("tests")
        .join("fixtures")
        .join("db_old_flat_snapshot.json");

    assert!(
        fixture_path.exists(),
        "Committed fixture not found: {:?}",
        fixture_path
    );

    let read_bytes = std::fs::read(&fixture_path).expect("Failed to read fixture");

    let migrated = StoreHandler::load_snapshot(&read_bytes).expect("Migration of fixture failed");

    let migrated_guard = migrated.guard();
    let migrated_inner = migrated
        .get(&Schema::default(), &migrated_guard)
        .expect("No public schema after migration");
    assert_eq!(
        migrated_inner.len(),
        1,
        "Expected 1 store under public schema"
    );
    let pinned = migrated_inner.pin();
    let (key, _) = pinned.iter().next().expect("No store in result");
    assert_eq!(
        key.value, "fixture_store",
        "Store name preserved after migration"
    );
}
