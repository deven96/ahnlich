use std::{sync::Arc, time::Duration};

use grpc_types::services::db_service::{
    db_service_client::DbServiceClient, db_service_server::DbServiceServer,
};
use once_cell::sync::Lazy;
use tonic::transport::Channel;
use utils::connection_layer::{trace_with_parent, RequestTrackerLayer};

use crate::{cli::ServerConfig, server::handler::Server};

static CONFIG: Lazy<ServerConfig> =
    Lazy::new(|| ServerConfig::default().os_select_port().enable_tracing());

#[tokio::test]
async fn test_grpc_ping_test() {
    let server = Server::new(&CONFIG).await.expect("Failed to create server");
    let address = "127.0.0.1:3000";

    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("Cannot bind");

    let listener_stream = tokio_stream::wrappers::TcpListenerStream::new(listener);

    let request_tracker = RequestTrackerLayer::new(Arc::clone(&server.client_handler()));
    let db_service = DbServiceServer::new(server);

    tokio::spawn(async move {
        let res = tonic::transport::Server::builder()
            .layer(request_tracker)
            .trace_fn(trace_with_parent)
            .add_service(db_service)
            .serve_with_incoming(listener_stream)
            .await;
        println!("{res:?}");
    });

    let address = format!("http://{}", address);

    println!("address: {address:?}");

    tokio::time::sleep(Duration::from_secs(3)).await;
    let channel = Channel::from_shared(address).expect("Faild to get channel");

    let mut client = DbServiceClient::connect(channel).await.expect("Failure");

    let response = client
        .ping(tonic::Request::new(grpc_types::db::query::Ping {}))
        .await
        .expect("Failed to ping");

    println!("Response: {response:?}");
}
