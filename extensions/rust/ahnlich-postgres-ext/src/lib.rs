use pgrx::{prelude::*, Json};

::pgrx::pg_module_magic!(name, version);

use ahnlich_ai_proxy::cli::AIProxyConfig;
use ahnlich_ai_proxy::server::handler::AIProxyServer;

use ahnlich_client_rs::ai::AiClient;

use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;

use ahnlich_types::ai::query::ConvertStoreInputToEmbeddings;
use ahnlich_types::ai::server::StoreInputToEmbeddingsList;

use once_cell::sync::Lazy;

use std::net::SocketAddr;

use tokio;
use tokio::runtime::Runtime;
use tokio::sync::OnceCell;
use tokio::time::Duration;

use utils::server::AhnlichServerUtils;

static TOKIO_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime!")
});

static DB_CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());
static AI_CONFIG: Lazy<AIProxyConfig> = Lazy::new(|| {
    let mut ai_proxy = AIProxyConfig::default().os_select_port();
    ai_proxy.db_port = DB_CONFIG.port.clone();
    ai_proxy.db_host = DB_CONFIG.common.host.clone();
    ai_proxy
});

// Global (per-process) singleton client instance
static CLIENT: OnceCell<AiClient> = OnceCell::const_new();

async fn provision_servers() -> SocketAddr {
    let server = Server::new(&DB_CONFIG)
        .await
        .expect("Could not initialize DB server");

    let db_port = server.local_addr().unwrap().port();

    let mut config = AI_CONFIG.clone();

    config.db_port = db_port;

    let ai_server = AIProxyServer::new(config)
        .await
        .expect("Could not initialize AI proxy");

    let ai_address = ai_server.local_addr().expect("Could not get local address");

    // Start up DB server
    let _ = tokio::spawn(async move { server.start().await });

    // Start up AI proxy
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
fn init_client(base_url: Option<&str>) -> &'static str {
    TOKIO_RUNTIME.block_on(async {
        match CLIENT
            .get_or_try_init(|| async {
                let ai_address = provision_servers().await;
                AiClient::new(ai_address.to_string()).await
            })
            .await
        {
            Ok(_) => "Client initialized successfully!",
            Err(_) => "Client initialization failed!",
        }
    })
}

// #[pg_extern]
// #[tokio::main]
// async fn init_client(base_url: Option<&str>) -> &'static str {
//     match CLIENT
//         .get_or_try_init(|| async {
//             let ai_address = provision_test_servers().await;
//             AiClient::new(ai_address.to_string()).await
//         })
//         .await
//     {
//         Ok(_) => "Client initialized successfully",
//         Err(_) => "Client initialization failed",
//     }
// }

#[pg_extern]
fn ping() -> &'static str {
    TOKIO_RUNTIME.block_on(async {
        match CLIENT.get() {
            Some(client) => {
                if client.ping(None).await.is_ok() {
                    "Ping successful!"
                } else {
                    "Ping failed!"
                }
            }
            None => "Could not find client!",
        }
    })
}

// #[pg_extern]
// #[tokio::main]
// async fn ping() -> &'static str {
//     let address = provision_servers().await;
//     let ai_client = AiClient::new(address.to_string())
//         .await
//         .expect("Could not initialize client");

//     if ai_client.ping(None).await.is_ok() {
//         "Ping successful!"
//     } else {
//         "Ping failed!"
//     }
// }

#[pg_extern]
fn ping_with_args(i: i32) -> String {
    TOKIO_RUNTIME.block_on(async {
        match CLIENT.get() {
            Some(client) => {
                let res = client.ping(None).await;

                if res.is_ok() {
                    format!("Ping successful! Args: {}, Response: {:?}", i, res.unwrap())
                } else {
                    format!("Ping failed! Args: {}, Response: {:?}", i, res.unwrap())
                }
            }
            None => format!("Could not find client! Args: {}", i,),
        }
    })
}

// #[pg_extern]
// #[tokio::main]
// async fn ping_with_args_optional(i: Option<i32>) -> String {
//     let address = provision_servers().await;
//     let ai_client = AiClient::new(address.to_string())
//         .await
//         .expect("Could not initialize client");

//     let res = ai_client.ping(None).await;

//     if res.is_ok() {
//         format!("Ping successful! Args: {}, Response: {:?}", i, res.unwrap())
//     } else {
//         format!("Ping failed! Args: {}, Response: {:?}", i, res.unwrap())
//     }
// }

#[pg_extern]
fn convert_store_input_to_embeddings(query: Json) -> StoreInputToEmbeddingsList {
    TOKIO_RUNTIME.block_on(async {
        match CLIENT.get() {
            Some(client) => {
                // Deserialize the JSON into your struct
                let query: ConvertStoreInputToEmbeddings = serde_json::from_value(query.0)
                    .expect("Invalid JSON for ConvertStoreInputToEmbeddings");

                let response = client
                    .convert_store_input_to_embeddings(query, None)
                    .await
                    .expect("Failed to convert store input to embeddings");

                response
            }
            None => StoreInputToEmbeddingsList { values: vec![] },
        }
    })
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
