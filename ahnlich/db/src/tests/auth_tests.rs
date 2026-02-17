use crate::cli::ServerConfig;
use crate::server::handler::Server;
use ahnlich_client_rs::db::DbClient;
use ahnlich_types::db::query::Ping;
use ahnlich_types::services::db_service::db_service_client::DbServiceClient;
use once_cell::sync::Lazy;
use rcgen::generate_simple_self_signed;
use std::collections::HashMap;
use std::path::PathBuf;
use tempfile::TempDir;
use tokio::time::Duration;
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use utils::auth::hash_api_key;
use utils::server::AhnlichServerUtils;

static BASE_CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());

fn generate_test_tls(dir: &TempDir) -> (PathBuf, PathBuf, String) {
    let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
    let cert = generate_simple_self_signed(subject_alt_names).expect("Failed to generate cert");

    let cert_pem = cert.cert.pem();
    let key_pem = cert.key_pair.serialize_pem();

    let cert_path = dir.path().join("server.crt");
    let key_path = dir.path().join("server.key");

    std::fs::write(&cert_path, &cert_pem).expect("Failed to write cert");
    std::fs::write(&key_path, &key_pem).expect("Failed to write key");

    (cert_path, key_path, cert_pem)
}

fn write_auth_config(dir: &TempDir, users: HashMap<&str, &str>) -> PathBuf {
    let path = dir.path().join("auth.toml");
    let mut content = String::from("[users]\n");
    for (username, api_key) in users {
        let hashed = hash_api_key(api_key);
        content.push_str(&format!("{} = \"{}\"\n", username, hashed));
    }
    content.push_str("\n[security]\nmin_key_length = 8\n");
    std::fs::write(&path, content).expect("Failed to write auth config");
    path
}

async fn tls_channel(addr: std::net::SocketAddr, cert_pem: &str) -> Channel {
    let ca = Certificate::from_pem(cert_pem);
    let tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("localhost");
    Channel::from_shared(format!("https://localhost:{}", addr.port()))
        .expect("Invalid address")
        .tls_config(tls)
        .expect("Invalid TLS config")
        .connect()
        .await
        .expect("Failed to connect")
}

#[tokio::test]
async fn test_unauthenticated_request_rejected() {
    let tmp = TempDir::new().unwrap();
    let (cert_path, key_path, cert_pem) = generate_test_tls(&tmp);
    let auth_config = write_auth_config(&tmp, [("alice", "alicepass")].into());

    let config = BASE_CONFIG
        .clone()
        .with_auth(auth_config, cert_path, key_path);

    let server = Server::new(&config).await.expect("Failed to create server");
    let addr = server.local_addr().unwrap();
    tokio::spawn(async move { server.start().await });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let channel = tls_channel(addr, &cert_pem).await;
    let mut client = DbServiceClient::new(channel);

    let result = client.ping(tonic::Request::new(Ping {})).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
}

#[tokio::test]
async fn test_wrong_credentials_rejected() {
    let tmp = TempDir::new().unwrap();
    let (cert_path, key_path, cert_pem) = generate_test_tls(&tmp);
    let auth_config = write_auth_config(&tmp, [("alice", "alicepass")].into());

    let config = BASE_CONFIG
        .clone()
        .with_auth(auth_config, cert_path, key_path);

    let server = Server::new(&config).await.expect("Failed to create server");
    let addr = server.local_addr().unwrap();
    tokio::spawn(async move { server.start().await });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let channel = tls_channel(addr, &cert_pem).await;
    let mut client = DbServiceClient::new(channel);

    let mut req = tonic::Request::new(Ping {});
    req.metadata_mut().insert(
        "authorization",
        "Bearer alice:wrongpassword".parse().unwrap(),
    );

    let result = client.ping(req).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().code(), tonic::Code::Unauthenticated);
}

#[tokio::test]
async fn test_valid_credentials_accepted() {
    let tmp = TempDir::new().unwrap();
    let (cert_path, key_path, cert_pem) = generate_test_tls(&tmp);
    let auth_config = write_auth_config(&tmp, [("alice", "alicepass")].into());

    let config = BASE_CONFIG
        .clone()
        .with_auth(auth_config, cert_path, key_path);

    let server = Server::new(&config).await.expect("Failed to create server");
    let addr = server.local_addr().unwrap();
    tokio::spawn(async move { server.start().await });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let channel = tls_channel(addr, &cert_pem).await;
    let mut client = DbServiceClient::new(channel);

    let mut req = tonic::Request::new(Ping {});
    req.metadata_mut()
        .insert("authorization", "Bearer alice:alicepass".parse().unwrap());

    let result = client.ping(req).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_multiple_users_auth() {
    let tmp = TempDir::new().unwrap();
    let (cert_path, key_path, cert_pem) = generate_test_tls(&tmp);
    let auth_config =
        write_auth_config(&tmp, [("alice", "alicepass"), ("bob", "bobspass1")].into());

    let config = BASE_CONFIG
        .clone()
        .with_auth(auth_config, cert_path, key_path);

    let server = Server::new(&config).await.expect("Failed to create server");
    let addr = server.local_addr().unwrap();
    tokio::spawn(async move { server.start().await });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let channel = tls_channel(addr, &cert_pem).await;

    let mut client = DbServiceClient::new(channel.clone());
    let mut req = tonic::Request::new(Ping {});
    req.metadata_mut()
        .insert("authorization", "Bearer alice:alicepass".parse().unwrap());
    assert!(client.ping(req).await.is_ok());

    let mut client = DbServiceClient::new(channel.clone());
    let mut req = tonic::Request::new(Ping {});
    req.metadata_mut()
        .insert("authorization", "Bearer bob:bobspass1".parse().unwrap());
    assert!(client.ping(req).await.is_ok());

    let mut client = DbServiceClient::new(channel);
    let mut req = tonic::Request::new(Ping {});
    req.metadata_mut().insert(
        "authorization",
        "Bearer charlie:charliepass".parse().unwrap(),
    );
    assert_eq!(
        client.ping(req).await.unwrap_err().code(),
        tonic::Code::Unauthenticated
    );
}

#[tokio::test]
async fn test_db_client_with_auth_token() {
    let tmp = TempDir::new().unwrap();
    let (cert_path, key_path, cert_pem) = generate_test_tls(&tmp);
    let auth_config = write_auth_config(&tmp, [("alice", "alicepass")].into());

    let config = BASE_CONFIG
        .clone()
        .with_auth(auth_config, cert_path, key_path);

    let server = Server::new(&config).await.expect("Failed to create server");
    let addr = server.local_addr().unwrap();
    tokio::spawn(async move { server.start().await });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let ca = Certificate::from_pem(&cert_pem);
    let tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("localhost");

    let channel = Channel::from_shared(format!("https://localhost:{}", addr.port()))
        .unwrap()
        .tls_config(tls)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let mut db_client = DbClient::new_with_channel(channel);
    db_client.set_auth_token("alice:alicepass".to_string());

    let result = db_client.list_stores(None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_db_client_without_auth_token_rejected() {
    let tmp = TempDir::new().unwrap();
    let (cert_path, key_path, cert_pem) = generate_test_tls(&tmp);
    let auth_config = write_auth_config(&tmp, [("alice", "alicepass")].into());

    let config = BASE_CONFIG
        .clone()
        .with_auth(auth_config, cert_path, key_path);

    let server = Server::new(&config).await.expect("Failed to create server");
    let addr = server.local_addr().unwrap();
    tokio::spawn(async move { server.start().await });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let ca = Certificate::from_pem(&cert_pem);
    let tls = ClientTlsConfig::new()
        .ca_certificate(ca)
        .domain_name("localhost");

    let channel = Channel::from_shared(format!("https://localhost:{}", addr.port()))
        .unwrap()
        .tls_config(tls)
        .unwrap()
        .connect()
        .await
        .unwrap();

    let db_client = DbClient::new_with_channel(channel);
    let result = db_client.list_stores(None).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_no_auth_server_accepts_requests_without_token() {
    let server = Server::new(&BASE_CONFIG)
        .await
        .expect("Failed to create server");
    let addr = server.local_addr().unwrap();
    tokio::spawn(async move { server.start().await });
    tokio::time::sleep(Duration::from_millis(100)).await;

    let db_client = DbClient::new(format!("http://{}", addr))
        .await
        .expect("Could not connect");

    let result = db_client.list_stores(None).await;
    assert!(result.is_ok());
}
