use futures::future::join_all;
use server::cli::ServerConfig;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use types::bincode::BinCodeSerAndDeser;
use types::query::Query;
use types::query::ServerQuery;
use types::server::ServerResponse;
use types::server::ServerResult;

#[tokio::test]
async fn test_run_server_echos() {
    let server = server::Server::new(&ServerConfig::default())
        .await
        .expect("Could not initialize server");
    let address = server.local_addr().expect("Could not get local addr");
    let _ = tokio::spawn(async move { server.start().await });

    // Allow some time for the server to start
    tokio::time::sleep(Duration::from_millis(100)).await;
    let tasks = vec![
        tokio::spawn(async move {
            let message = ServerQuery::from_queries(&[Query::InfoServer, Query::Ping]);
            let mut expected = ServerResult::with_capacity(2);
            expected.push(Ok(ServerResponse::Unit));
            expected.push(Ok(ServerResponse::Pong));
            query_server_assert_result(address, message, expected).await
        }),
        tokio::spawn(async move {
            let message = ServerQuery::from_queries(&[Query::Ping, Query::InfoServer]);
            let mut expected = ServerResult::with_capacity(2);
            expected.push(Ok(ServerResponse::Pong));
            expected.push(Ok(ServerResponse::Unit));
            query_server_assert_result(address, message, expected).await
        }),
    ];
    join_all(tasks).await;
}

async fn query_server_assert_result(
    server_addr: SocketAddr,
    query: ServerQuery,
    expected_result: ServerResult,
) {
    // Connect to the server
    let stream = TcpStream::connect(server_addr).await.unwrap();
    let mut reader = BufReader::new(stream);

    // Message to send
    let serialized_message = query.serialize().unwrap();

    // Send the message
    reader.write_all(&serialized_message).await.unwrap();

    // get length of response
    let mut length_header = [0u8; types::bincode::LENGTH_HEADER_SIZE];
    timeout(
        Duration::from_secs(1),
        reader.read_exact(&mut length_header),
    )
    .await
    .unwrap()
    .unwrap();
    let data_length = u64::from_be_bytes(length_header);
    let mut response = vec![0u8; data_length as usize];

    timeout(Duration::from_secs(1), reader.read_exact(&mut response))
        .await
        .unwrap()
        .unwrap();

    let response = ServerResult::deserialize(false, &response).unwrap();

    assert_eq!(response, expected_result);
}
