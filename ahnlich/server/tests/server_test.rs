use server::cli::ServerConfig;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_run_server_echos() {
    let server = server::Server::new(&ServerConfig::default())
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });

    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Connect to the server
    let stream = TcpStream::connect(address).await.unwrap();
    let mut reader = BufReader::new(stream);

    // Message to send
    let message = b"Hello, world!\n";

    // Send the message
    reader.write_all(message).await.unwrap();

    let mut response = String::new();

    timeout(Duration::from_secs(1), reader.read_line(&mut response))
        .await
        .unwrap()
        .unwrap();

    assert_eq!(message, response.as_bytes());
}
