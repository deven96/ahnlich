use std::io::Write;
use std::process::{Command, Stdio};

#[test]
#[ignore] // Requires running ahnlich server
fn test_non_interactive_ping_success() {
    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "-p",
            "cli",
            "--",
            "ahnlich",
            "--agent",
            "db",
            "--host",
            "127.0.0.1",
            "--port",
            "1369",
            "--no-interactive",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    // Write PING to stdin
    {
        let stdin = child.stdin.as_mut().expect("Failed to open stdin");
        stdin.write_all(b"PING").expect("Failed to write to stdin");
    }

    let output = child.wait_with_output().expect("Failed to wait on child");

    assert!(output.status.success(), "Command should exit with 0");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Success"), "Output should contain Success");

    // Verify no ANSI codes in output
    assert!(
        !stdout.contains("\x1b["),
        "Output should not contain ANSI codes"
    );
}

#[test]
fn test_non_interactive_empty_input() {
    let mut child = Command::new("cargo")
        .args(&[
            "run",
            "-p",
            "cli",
            "--",
            "ahnlich",
            "--agent",
            "db",
            "--host",
            "127.0.0.1",
            "--port",
            "1369",
            "--no-interactive",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    // Close stdin without writing anything
    drop(child.stdin.take());

    let output = child.wait_with_output().expect("Failed to wait on child");

    assert!(
        !output.status.success(),
        "Command should exit with non-zero"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("No input provided"),
        "Error should mention no input"
    );
}
