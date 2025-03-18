use std::time::Duration;

use grpc_types::services::db_service::db_service_client::DbServiceClient;
use once_cell::sync::Lazy;
use tonic::transport::Channel;
use utils::server::AhnlichServerUtils;

use crate::{cli::ServerConfig, server::handler::Server};

static CONFIG: Lazy<ServerConfig> =
    Lazy::new(|| ServerConfig::default().os_select_port().enable_tracing());

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

    println!("Response: {response:?}");
}
