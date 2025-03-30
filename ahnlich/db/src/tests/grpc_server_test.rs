use crate::server::handler::Server;
use crate::{cli::ServerConfig, errors::ServerError};
use grpc_types::keyval::store_input::Value as StoreInputValue;
use grpc_types::keyval::{StoreEntry, StoreKey, StoreValue};
use grpc_types::metadata::metadata_value::Value as MetadataValueEnum;
use grpc_types::metadata::MetadataValue;
use grpc_types::predicates::{
    self, predicate::Kind as PredicateKind, predicate_condition::Kind as PredicateConditionKind,
    Predicate, PredicateCondition,
};
use grpc_types::shared::info::StoreUpsert;
use once_cell::sync::Lazy;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::time::Duration;
use utils::server::AhnlichServerUtils;

use grpc_types::{
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

    tokio::spawn(async move {
        server.task_manager().spawn_blocking(server).await;
    });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_secs(3)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    // maximum_message_size => DbServiceServer(server).max_decoding_message_size
    // maximum_clients => At this point yet to figure out but it might be manually implementing
    // Server/Interceptor as shown in https://chatgpt.com/share/67abdf0b-72a8-8008-b203-bc8e65b02495
    // maximum_concurrency_per_client => we just set this with `concurrency_limit_per_connection`.
    // for creating trace functions, we can add `trace_fn` and extract our header from `Request::header` and return the span
    let response = client
        .ping(tonic::Request::new(grpc_types::db::query::Ping {}))
        .await
        .expect("Failed to ping");

    let expected = db_response_types::Pong {};
    println!("Response: {response:?}");
    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_maximum_client_restriction_works() {
    todo!()
}

// FIXME: failing: check out dbclient connection with_origin
#[tokio::test]
async fn test_server_client_info() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move {
        server.task_manager().spawn_blocking(server).await;
    });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_secs(3)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let response = client
        .list_clients(tonic::Request::new(grpc_types::db::query::ListClients {}))
        .await
        .expect("Failed to list clients");

    let expected = db_response_types::ClientList { clients: vec![] };
    println!("Response: {response:?}");
    assert_eq!(expected, response.into_inner());
}

#[tokio::test]
async fn test_simple_stores_list() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = server.local_addr().expect("Could not get local addr");

    tokio::spawn(async move {
        server.task_manager().spawn_blocking(server).await;
    });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_secs(3)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let response = client
        .list_stores(tonic::Request::new(grpc_types::db::query::ListStores {}))
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

    tokio::spawn(async move {
        server.task_manager().spawn_blocking(server).await;
    });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_secs(3)).await;
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
                grpc_types::shared::info::ErrorResponse {
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
                        size_in_bytes: 1720,
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

    tokio::spawn(async move {
        server.task_manager().spawn_blocking(server).await;
    });

    let address = format!("http://{}", address);

    tokio::time::sleep(Duration::from_secs(3)).await;
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
                                    grpc_types::metadata::metadata_value::Value::RawString(
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
                                    grpc_types::metadata::metadata_value::Value::RawString(
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
                        value: Some(grpc_types::keyval::StoreValue {
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
                        value: Some(grpc_types::keyval::StoreValue {
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
                                    grpc_types::metadata::metadata_value::Value::RawString(
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
                                    grpc_types::metadata::metadata_value::Value::RawString(
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
                grpc_types::shared::info::ErrorResponse {
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
                        size_in_bytes: 1928,
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
                        size_in_bytes: 1720,
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
