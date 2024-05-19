use server::ServerConfig;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_run_server_echos() {
    let server_config = ServerConfig::default();
    spawn_app(&server_config).await;

    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the server
    let stream = TcpStream::connect(format!("{}:{}", &server_config.host, &server_config.port))
        .await
        .unwrap();
    let mut reader = BufReader::new(stream);

    // Message to send
    let message = "Hello, world!\n";

    // Send the message
    reader.write_all(message.as_bytes()).await.unwrap();

    let mut response = String::new();

    timeout(Duration::from_secs(1), reader.read_line(&mut response))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(message, response);
}

async fn spawn_app(config: &ServerConfig) {
    let test_server = server::run_server(config.to_owned());

    let _ = tokio::spawn(test_server);
}
