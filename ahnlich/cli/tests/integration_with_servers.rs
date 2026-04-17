use ahnlich_ai_proxy::cli::AIProxyConfig;
use ahnlich_ai_proxy::server::handler::AIProxyServer;
use ahnlich_client_rs::{ai::AiClient, db::DbClient};
use ahnlich_db::cli::ServerConfig;
use ahnlich_db::server::handler::Server;
use std::io::Write;
use std::process::{Command, Stdio};
use tokio::time::{Duration, timeout};
use utils::server::AhnlichServerUtils;

/// Polls the server with ping until it responds, or panics after the deadline.
async fn wait_for_db_ready(port: u16) {
    let addr = format!("http://127.0.0.1:{port}");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        if tokio::time::Instant::now() > deadline {
            panic!("DB server on port {port} did not become ready within 15s");
        }
        if let Ok(client) = DbClient::new(addr.clone()).await {
            if client.ping(None).await.is_ok() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Polls the server with ping until it responds, or panics after the deadline.
async fn wait_for_ai_ready(port: u16) {
    let addr = format!("http://127.0.0.1:{port}");
    let deadline = tokio::time::Instant::now() + Duration::from_secs(15);
    loop {
        if tokio::time::Instant::now() > deadline {
            panic!("AI server on port {port} did not become ready within 15s");
        }
        if let Ok(client) = AiClient::new(addr.clone()).await {
            if client.ping(None).await.is_ok() {
                return;
            }
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

/// Provisions a DB server on an OS-selected port and returns the port number
async fn provision_db_server() -> u16 {
    let config = ServerConfig::default().os_select_port();
    let server = Server::new(&config)
        .await
        .expect("Failed to create DB server");
    let port = server.local_addr().unwrap().port();
    tokio::spawn(async move { server.start().await });
    wait_for_db_ready(port).await;
    port
}

/// Provisions an AI server (with DB backend) on an OS-selected port and returns the AI port number
/// Note: AI server requires DB connection for most commands (PING, INFOSERVER, etc.)
async fn provision_ai_server() -> u16 {
    // First provision DB server (AI needs it)
    let db_config = ServerConfig::default().os_select_port();
    let db_server = Server::new(&db_config)
        .await
        .expect("Failed to create DB server");
    let db_port = db_server.local_addr().unwrap().port();
    tokio::spawn(async move { db_server.start().await });
    wait_for_db_ready(db_port).await;

    // Then provision AI server connected to DB
    let mut ai_config = AIProxyConfig::default().os_select_port();
    ai_config.db_port = db_port;
    let ai_server = AIProxyServer::new(ai_config)
        .await
        .expect("Failed to create AI server");
    let ai_port = ai_server.local_addr().unwrap().port();
    tokio::spawn(async move { ai_server.start().await });
    wait_for_ai_ready(ai_port).await;

    ai_port
}

/// Helper to run CLI command and return output with timeout protection.
///
/// Uses `spawn_blocking` to properly handle the blocking `Command::wait_with_output()` call
/// within an async context, preventing the CLI process from being terminated prematurely.
///
/// The stdin is explicitly closed via `take()` to signal EOF to the CLI's non-interactive mode.
async fn run_cli_command(agent: &str, port: u16, input: &str) -> std::process::Output {
    // Convert to owned strings before moving into closure
    let agent = agent.to_string();
    let input = input.to_string();

    let result = timeout(
        Duration::from_secs(60),
        tokio::task::spawn_blocking(move || {
            let mut child = Command::new("cargo")
                .args(&[
                    "run",
                    "--quiet",
                    "-p",
                    "cli",
                    "--",
                    "ahnlich",
                    "--agent",
                    &agent,
                    "--host",
                    "127.0.0.1",
                    "--port",
                    &port.to_string(),
                    "--no-interactive",
                ])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to spawn CLI");

            // Write to stdin and explicitly close it to signal EOF
            {
                let mut stdin = child.stdin.take().expect("Failed to open stdin");
                stdin
                    .write_all(input.as_bytes())
                    .expect("Failed to write to stdin");
                // stdin is dropped here, signaling EOF
            }

            // Wait for the child process to complete
            child.wait_with_output().expect("Failed to wait on child")
        }),
    )
    .await;

    match result {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => panic!("Task failed: {}", e),
        Err(_) => panic!("CLI command timed out after 15 seconds"),
    }
}

#[tokio::test]
async fn test_db_ping() {
    let port = provision_db_server().await;
    let output = run_cli_command("db", port, "PING").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(output.status.success(), "Command should exit with 0");

    assert!(
        stdout.contains("Pong"),
        "Output should contain Pong response, got: {}",
        stdout
    );

    // Verify no ANSI codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}

#[tokio::test]
async fn test_ai_ping() {
    let port = provision_ai_server().await;
    let output = run_cli_command("ai", port, "PING").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(output.status.success(), "Command should exit with 0");

    assert!(
        stdout.contains("Pong"),
        "Output should contain Pong response, got: {}",
        stdout
    );

    // Verify no ANSI codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}

#[tokio::test]
async fn test_ai_infoserver() {
    let port = provision_ai_server().await;
    let output = run_cli_command("ai", port, "INFOSERVER").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(output.status.success(), "Command should exit with 0");

    assert!(
        stdout.contains("InfoServer"),
        "Output should contain InfoServer response, got: {}",
        stdout
    );

    // Verify no ANSI codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}

#[tokio::test]
async fn test_ai_liststores() {
    let port = provision_ai_server().await;
    let output = run_cli_command("ai", port, "LISTSTORES").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(output.status.success(), "Command should exit with 0");

    assert!(
        stdout.contains("StoreList"),
        "Output should contain StoreList response, got: {}",
        stdout
    );

    // Verify no ANSI codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}

#[tokio::test]
async fn test_ai_multiple_commands() {
    let port = provision_ai_server().await;
    let output = run_cli_command("ai", port, "PING; INFOSERVER").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(output.status.success(), "Command should exit with 0");

    assert!(
        stdout.contains("Pong"),
        "Output should contain Pong response, got: {}",
        stdout
    );

    assert!(
        stdout.contains("InfoServer"),
        "Output should contain InfoServer response, got: {}",
        stdout
    );

    // Verify no ANSI codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}

#[tokio::test]
async fn test_connection_refused() {
    let invalid_port = 54321;
    let output = run_cli_command("db", invalid_port, "PING").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(
        !output.status.success(),
        "Command should fail with non-zero exit code"
    );

    let stderr_lower = stderr.to_lowercase();
    assert!(
        stderr_lower.contains("refused")
            || stderr_lower.contains("could not connect")
            || stderr_lower.contains("connection")
            || stderr_lower.contains("transport"),
        "Stderr should contain connection error message, got: {}",
        stderr
    );
}

#[tokio::test]
async fn test_invalid_command() {
    let port = provision_db_server().await;
    let output = run_cli_command("db", port, "INVALIDCOMMAND").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(
        !output.status.success(),
        "Command should fail with non-zero exit code"
    );

    assert!(
        !stderr.is_empty(),
        "Stderr should contain error message for invalid command"
    );
}

#[tokio::test]
async fn test_db_infoserver() {
    let port = provision_db_server().await;
    let output = run_cli_command("db", port, "INFOSERVER").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(output.status.success(), "Command should exit with 0");

    assert!(
        stdout.contains("InfoServer"),
        "Output should contain InfoServer response, got: {}",
        stdout
    );

    // Verify no ANSI codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}

#[tokio::test]
async fn test_db_liststores() {
    let port = provision_db_server().await;
    let output = run_cli_command("db", port, "LISTSTORES").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(output.status.success(), "Command should exit with 0");

    assert!(
        stdout.contains("StoreList"),
        "Output should contain StoreList response, got: {}",
        stdout
    );

    // Verify no ANSI codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}

#[tokio::test]
async fn test_db_multiple_commands() {
    let port = provision_db_server().await;
    let output = run_cli_command("db", port, "PING; INFOSERVER").await;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("Exit status: {:?}", output.status);
    eprintln!("STDOUT: {}", stdout);
    eprintln!("STDERR: {}", stderr);

    assert!(output.status.success(), "Command should exit with 0");

    assert!(
        stdout.contains("Pong"),
        "Output should contain Pong response, got: {}",
        stdout
    );

    assert!(
        stdout.contains("InfoServer"),
        "Output should contain InfoServer response, got: {}",
        stdout
    );

    // Verify no ANSI codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}
