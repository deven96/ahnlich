use server::cli::ServerConfig;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use types::query::deserialize_queries;
use types::query::Query;
use types::query::SerializedQuery;

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
    let message = vec![Query::InfoServer, Query::Ping];
    let serialized_message = SerializedQuery::from_queries(&message).unwrap();

    // Send the message
    reader.write_all(serialized_message.data()).await.unwrap();

    let mut response = vec![0u8; serialized_message.length()];

    timeout(Duration::from_secs(1), reader.read_exact(&mut response))
        .await
        .unwrap()
        .unwrap();

    let deserialized = deserialize_queries(&response).unwrap();

    assert_eq!(message, deserialized);
}
