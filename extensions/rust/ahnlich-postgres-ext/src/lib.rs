use pgrx::prelude::*;

::pgrx::pg_module_magic!(name, version);

use ahnlich_client_rs::ai::AiClient;
use tokio;

use ahnlich_ai_proxy::cli::AIProxyConfig;
use ahnlich_ai_proxy::server::handler::AIProxyServer;

use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;

use once_cell::sync::Lazy;

use std::net::SocketAddr;

use tokio::time::Duration;
use utils::server::AhnlichServerUtils;

static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());
static AI_CONFIG: Lazy<AIProxyConfig> = Lazy::new(|| {
    let mut ai_proxy = AIProxyConfig::default().os_select_port();
    ai_proxy.db_port = CONFIG.port.clone();
    ai_proxy.db_host = CONFIG.common.host.clone();
    ai_proxy
});

async fn provision_test_servers() -> SocketAddr {
    let server = Server::new(&CONFIG)
        .await
        .expect("Could not initialize server");

    let db_port = server.local_addr().unwrap().port();

    let mut config = AI_CONFIG.clone();

    config.db_port = db_port;

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not initialize ai proxy");

    let ai_address = ai_server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });
    // start up ai proxy
    let _ = tokio::spawn(async move { ai_server.start().await });
    // Allow some time for the servers to start
    tokio::time::sleep(Duration::from_millis(200)).await;

    ai_address
}

#[pg_extern]
fn hello_ahnlich_postgres_ext() -> &'static str {
    "Hello, ahnlich_postgres_ext"
}

#[pg_extern]
#[tokio::main]
async fn ping() -> bool {
    let address = provision_test_servers().await;
    let ai_client = AiClient::new(address.to_string())
        .await
        .expect("Could not initialize client");

    ai_client.ping(None).await.is_ok()
}

#[pg_extern]
#[tokio::main]
async fn run_query() -> &'static str {
    let address = provision_test_servers().await;
    let ai_client = AiClient::new(address.to_string())
        .await
        .expect("Could not initialize client");

    if ai_client.ping(None).await.is_ok() {
        "Ping successful!"
    } else {
        "Ping failed!"
    }
}

#[pg_extern]
#[tokio::main]
async fn run_query_with_args(i: i32) -> String {
    let address = provision_test_servers().await;
    let ai_client = AiClient::new(address.to_string())
        .await
        .expect("Could not initialize client");

    let res = ai_client.ping(None).await;

    if res.is_ok() {
        format!("Ping successful! Args: {}, Response: {:?}", i, res.unwrap())
    } else {
        format!("Ping failed! Args: {}, Response: {:?}", i, res.unwrap())
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pg_schema]
mod tests {
    use pgrx::prelude::*;

    #[pg_test]
    fn test_hello_ahnlich_postgres_ext() {
        assert_eq!(
            "Hello, ahnlich_postgres_ext",
            crate::hello_ahnlich_postgres_ext()
        );
    }
}

/// This module is required by `cargo pgrx test` invocations.
/// It must be visible at the root of your extension crate.
#[cfg(test)]
pub mod pg_test {
    pub fn setup(_options: Vec<&str>) {
        // perform one-off initialization when the pg_test framework starts
    }

    #[must_use]
    pub fn postgresql_conf_options() -> Vec<&'static str> {
        // return any postgresql.conf settings that are required for your tests
        vec![]
    }
}
