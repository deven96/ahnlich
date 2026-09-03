use std::collections::HashMap;

use ahnlich_types::db::{
    pipeline::{DbQuery, DbRequestPipeline, db_query::Query, db_server_response::Response},
    query,
};
use ahnlich_types::keyval::{DbStoreEntry, StoreKey, StoreValue};
use ahnlich_types::metadata::{MetadataValue, metadata_value::Value as MetadataValueKind};
use ahnlich_types::services::db_service::db_service_client::DbServiceClient;
use tonic::transport::{Channel, Error as TransportError};
use tonic::{Code, Request};
use utils::server::AhnlichServerUtils;

use crate::cli::ServerConfig;
use crate::server::handler::Server;

fn raw_string(value: &str) -> MetadataValue {
    MetadataValue {
        value: Some(MetadataValueKind::RawString(value.to_owned())),
    }
}

fn entry(embedding: Vec<f32>, label: &str) -> DbStoreEntry {
    DbStoreEntry {
        key: Some(StoreKey { key: embedding }),
        value: Some(StoreValue {
            value: HashMap::from([
                ("label".to_owned(), raw_string(label)),
                ("category".to_owned(), raw_string("document")),
            ]),
        }),
    }
}

fn entries() -> Vec<DbStoreEntry> {
    vec![
        entry(vec![1.0, 0.0, 0.0], "one"),
        entry(vec![2.0, 0.0, 0.0], "two"),
        entry(vec![3.0, 0.0, 0.0], "three"),
    ]
}

async fn connected_client() -> DbServiceClient<Channel> {
    let config = ServerConfig::default().os_select_port();
    let server = Server::new(&config)
        .await
        .expect("server creation should succeed");
    let address = server
        .local_addr()
        .expect("server address should be available");

    tokio::spawn(async move {
        server.start().await.expect("test server should run");
    });

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    DbServiceClient::connect(format!("http://{address}"))
        .await
        .unwrap_or_else(|error: TransportError| panic!("client connection should succeed: {error}"))
}

#[tokio::test]
async fn grpc_lists_pages_and_clears_store() {
    let mut client = connected_client().await;
    let store = "grpc-documents";

    client
        .create_store(Request::new(query::CreateStore {
            store: store.to_owned(),
            dimension: 3,
            create_predicates: vec!["category".to_owned()],
            non_linear_indices: vec![],
            error_if_exists: true,
            schema: None,
        }))
        .await
        .expect("store creation should succeed");

    client
        .set(Request::new(query::Set {
            store: store.to_owned(),
            inputs: entries(),
            schema: None,
        }))
        .await
        .expect("setting entries should succeed");

    let first_page = client
        .list_store_entries(Request::new(query::ListStoreEntries {
            store: store.to_owned(),
            cursor: None,
            limit: Some(2),
            condition: None,
            schema: None,
        }))
        .await
        .expect("first page should succeed")
        .into_inner();

    assert_eq!(first_page.entries.len(), 2);

    let second_page = client
        .list_store_entries(Request::new(query::ListStoreEntries {
            store: store.to_owned(),
            cursor: first_page.next_cursor,
            limit: Some(2),
            condition: None,
            schema: None,
        }))
        .await
        .expect("second page should succeed")
        .into_inner();

    assert_eq!(second_page.entries.len(), 1);
    assert!(second_page.next_cursor.is_none());

    let clear_response = client
        .clear_store(Request::new(query::ClearStore {
            store: store.to_owned(),
            schema: None,
        }))
        .await
        .expect("clearing the store should succeed")
        .into_inner();

    assert_eq!(clear_response.deleted_count, 3);

    let store_info = client
        .get_store(Request::new(query::GetStore {
            store: store.to_owned(),
            schema: None,
        }))
        .await
        .expect("cleared store should still exist")
        .into_inner();

    assert_eq!(store_info.len, 0);
    assert_eq!(store_info.dimension, 3);
    assert_eq!(store_info.predicate_indices, vec!["category".to_owned()],);

    let empty_page = client
        .list_store_entries(Request::new(query::ListStoreEntries {
            store: store.to_owned(),
            cursor: None,
            limit: Some(10),
            condition: None,
            schema: None,
        }))
        .await
        .expect("listing the cleared store should succeed")
        .into_inner();

    assert!(empty_page.entries.is_empty());
    assert!(empty_page.next_cursor.is_none());
}

#[tokio::test]
async fn pipeline_supports_listing_and_clearing() {
    let mut client = connected_client().await;
    let store = "pipeline-documents";

    let response = client
        .pipeline(Request::new(DbRequestPipeline {
            queries: vec![
                DbQuery {
                    query: Some(Query::CreateStore(query::CreateStore {
                        store: store.to_owned(),
                        dimension: 3,
                        create_predicates: vec![],
                        non_linear_indices: vec![],
                        error_if_exists: true,
                        schema: None,
                    })),
                },
                DbQuery {
                    query: Some(Query::Set(query::Set {
                        store: store.to_owned(),
                        inputs: entries(),
                        schema: None,
                    })),
                },
                DbQuery {
                    query: Some(Query::ListStoreEntries(query::ListStoreEntries {
                        store: store.to_owned(),
                        cursor: None,
                        limit: Some(1),
                        condition: None,
                        schema: None,
                    })),
                },
                DbQuery {
                    query: Some(Query::ClearStore(query::ClearStore {
                        store: store.to_owned(),
                        schema: None,
                    })),
                },
                DbQuery {
                    query: Some(Query::ListStoreEntries(query::ListStoreEntries {
                        store: store.to_owned(),
                        cursor: None,
                        limit: Some(10),
                        condition: None,
                        schema: None,
                    })),
                },
            ],
        }))
        .await
        .expect("pipeline should succeed")
        .into_inner();

    assert_eq!(response.responses.len(), 5);

    assert!(matches!(
        response.responses[0].response.as_ref(),
        Some(Response::Unit(_))
    ));

    assert!(matches!(
        response.responses[1].response.as_ref(),
        Some(Response::Set(_))
    ));

    let Some(Response::ListStoreEntries(first_page)) = response.responses[2].response.as_ref()
    else {
        panic!("third response should contain a store-entry page");
    };

    assert_eq!(first_page.entries.len(), 1);
    assert!(first_page.next_cursor.is_some());

    let Some(Response::Del(clear_response)) = response.responses[3].response.as_ref() else {
        panic!("fourth response should contain a deletion result");
    };

    assert_eq!(clear_response.deleted_count, 3);

    let Some(Response::ListStoreEntries(empty_page)) = response.responses[4].response.as_ref()
    else {
        panic!("fifth response should contain a store-entry page");
    };

    assert!(empty_page.entries.is_empty());
    assert!(empty_page.next_cursor.is_none());
}

#[tokio::test]
async fn grpc_maps_endpoint_errors_to_status_codes() {
    let mut client = connected_client().await;

    let list_error = client
        .list_store_entries(Request::new(query::ListStoreEntries {
            store: "missing".to_owned(),
            cursor: Some("invalid".to_owned()),
            limit: Some(10),
            condition: None,
            schema: None,
        }))
        .await
        .expect_err("invalid cursor should fail");

    assert_eq!(list_error.code(), Code::InvalidArgument);

    let clear_error = client
        .clear_store(Request::new(query::ClearStore {
            store: "missing".to_owned(),
            schema: None,
        }))
        .await
        .expect_err("missing store should fail");

    assert_eq!(clear_error.code(), Code::NotFound);
}
