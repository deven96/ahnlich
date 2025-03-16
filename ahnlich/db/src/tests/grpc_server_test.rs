use std::time::Duration;

use grpc_types::services::db_service::db_service_client::DbServiceClient;
use once_cell::sync::Lazy;
use tonic::transport::Channel;
use utils::server::AhnlichServerUtilsV2;

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

    let response = client
        .ping(tonic::Request::new(grpc_types::db::query::Ping {}))
        .await
        .expect("Failed to ping");

    println!("Response: {response:?}");
}
