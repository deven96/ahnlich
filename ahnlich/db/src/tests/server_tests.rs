use crate::server::handler::Server;
use crate::{cli::ServerConfig, errors::ServerError};
use ahnlich_types::algorithm::algorithms::Algorithm;
use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
use ahnlich_types::keyval::{StoreEntry, StoreKey, StoreValue};
use ahnlich_types::metadata::metadata_value::Value as MetadataValueEnum;
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::predicates::{
    self, predicate::Kind as PredicateKind, predicate_condition::Kind as PredicateConditionKind,
    Predicate, PredicateCondition,
};
use ahnlich_types::server_types::ServerType;
use ahnlich_types::shared::info::StoreUpsert;
use ahnlich_types::similarity::Similarity;
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
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
        .list_stores(tonic::Request::new(ahnlich_types::db::query::ListStores {}))
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
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
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
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::CreateStore(db_query_types::CreateStore {
                store: "Main".to_string(),
                dimension: 2,
                create_predicates: vec![],
                non_linear_indices: vec![],
                error_if_exists: true,
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
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
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
                    StoreEntry {
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
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
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
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
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
                    entries: vec![StoreEntry {
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
            })),
        },
        // Should not error but delete nothing (empty store)
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 1.1, 1.2, 1.3],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                    StoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.1, 1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                ],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2], // 3 dimensions vs store's 4
                }],
            })),
        },
        // Should work - delete existing key
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
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
            })),
        },
        // Should not error but delete nothing
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.0, 1.1, 1.2, 1.3],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                    StoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.1, 1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
                                MetadataValue {
                                    value: Some(MetadataValueEnum::Image(vec![1, 2, 3])),
                                },
                            )]),
                        }),
                    },
                ],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2],
                }],
            })),
        },
        // Should delete existing key
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.0, 1.1, 1.2, 1.3],
                }],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
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
                        size_in_bytes: 1232,
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
                        size_in_bytes: 1184,
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
    // let file_metadata = std::fs::metadata(&*PERSISTENCE_FILE).unwrap();

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
            })),
        },
        // Should not error as store exists
        db_pipeline::DbQuery {
            query: Some(Query::DelKey(db_query_types::DelKey {
                store: "Main".to_string(),
                keys: vec![],
            })),
        },
        // Should get persisted data
        db_pipeline::DbQuery {
            query: Some(Query::GetKey(db_query_types::GetKey {
                store: "Main".to_string(),
                keys: vec![StoreKey {
                    key: vec![1.1, 1.2, 1.3, 1.4],
                }],
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
                    entries: vec![StoreEntry {
                        key: Some(StoreKey {
                            key: vec![1.1, 1.2, 1.3, 1.4],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::from_iter([(
                                "medal".into(),
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
            })),
        },
        // Valid set operation
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![StoreEntry {
                    key: Some(StoreKey {
                        key: vec![1.23, 1.0, 0.2],
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                }],
            })),
        },
        // Should error due to dimension mismatch
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![StoreEntry {
                    key: Some(StoreKey {
                        key: vec![2.1], // 1 dimension vs store's 3
                    }),
                    value: Some(StoreValue {
                        value: HashMap::new(),
                    }),
                }],
            })),
        },
        // Upsert operation
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
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
                    StoreEntry {
                        key: Some(StoreKey {
                            key: vec![0.03, 5.1, 3.23],
                        }),
                        value: Some(StoreValue {
                            value: HashMap::new(),
                        }),
                    },
                ],
            })),
        },
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
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
                        size_in_bytes: 1320,
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
                non_linear_indices: vec![NonLinearAlgorithm::KdTree.into()],
                error_if_exists: true,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
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
                    StoreEntry {
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
                    StoreEntry {
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
            })),
        },
        // Remove KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::DropNonLinearAlgorithmIndex(
                db_query_types::DropNonLinearAlgorithmIndex {
                    store: "Main".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                    error_if_not_exists: false,
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
            })),
        },
        // Recreate KDTree index
        db_pipeline::DbQuery {
            query: Some(Query::CreateNonLinearAlgorithmIndex(
                db_query_types::CreateNonLinearAlgorithmIndex {
                    store: "Main".to_string(),
                    non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
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
                non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
                error_if_exists: true,
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
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
                    StoreEntry {
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
                    StoreEntry {
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
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
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
                    StoreEntry {
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
                    StoreEntry {
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
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
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
                    StoreEntry {
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
                    entries: vec![StoreEntry {
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
                    entries: vec![StoreEntry {
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
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
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
                    StoreEntry {
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
                        key: vec![0.2, 0.3, 0.4, 0.6],
                    },
                    StoreKey {
                        key: vec![0.4, 0.6],
                    },
                ],
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
        input_dimension: 4,
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
                        StoreEntry {
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
                        StoreEntry {
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
            })),
        },
        // Insert test data
        db_pipeline::DbQuery {
            query: Some(Query::Set(db_query_types::Set {
                store: "Main".into(),
                inputs: vec![
                    StoreEntry {
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
                    StoreEntry {
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
            })),
        },
        // Should return 0 (no new indexes created)
        db_pipeline::DbQuery {
            query: Some(Query::CreatePredIndex(db_query_types::CreatePredIndex {
                store: "Main".to_string(),
                predicates: vec!["galaxy".into()],
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
            })),
        },
        // Create indexes for technology and life-form (should return 2)
        db_pipeline::DbQuery {
            query: Some(Query::CreatePredIndex(db_query_types::CreatePredIndex {
                store: "Main".to_string(),
                predicates: vec!["technology".into(), "life-form".into(), "galaxy".into()],
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
                    entries: vec![StoreEntry {
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
                    entries: vec![StoreEntry {
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
                    entries: vec![StoreEntry {
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
                    entries: vec![StoreEntry {
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
                    entries: vec![StoreEntry {
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
            })),
        },
        // Should not error (error_if_not_exists=false)
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: false,
                predicates: vec!["planet".into()],
            })),
        },
        // Should error (predicate doesn't exist)
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: true,
                predicates: vec!["planet".into()],
            })),
        },
        // Should succeed (galaxy predicate exists)
        db_pipeline::DbQuery {
            query: Some(Query::DropPredIndex(db_query_types::DropPredIndex {
                store: "Main".to_string(),
                error_if_not_exists: true,
                predicates: vec!["galaxy".into()],
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
            })),
        },
        // List stores
        db_pipeline::DbQuery {
            query: Some(Query::ListStores(db_query_types::ListStores {})),
        },
        // Should succeed (store exists)
        db_pipeline::DbQuery {
            query: Some(Query::DropStore(db_query_types::DropStore {
                store: "Main".to_string(),
                error_if_not_exists: true,
            })),
        },
        // Should error (store doesn't exist)
        db_pipeline::DbQuery {
            query: Some(Query::DropStore(db_query_types::DropStore {
                store: "Main".to_string(),
                error_if_not_exists: true,
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
