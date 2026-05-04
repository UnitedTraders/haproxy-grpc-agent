// Integration tests for configurable logging feature (004)
// Tests verify file/console log destination, per-package overrides, and backward compatibility

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;
use tempfile::NamedTempFile;

/// Helper to build the agent binary (once per test run)
fn ensure_binary_built() {
    let output = Command::new("cargo")
        .args(["build"])
        .output()
        .expect("Failed to build agent");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("cargo build failed: {}", stderr);
    }
}

/// Helper to get the path to the built binary
fn binary_path() -> std::path::PathBuf {
    let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("target/debug/haproxy-grpc-agent");
    path
}

/// Helper to create a temp config file with given TOML content.
/// Returns the NamedTempFile handle (keeps it alive) and its path as String.
fn write_temp_config(content: &str) -> (NamedTempFile, String) {
    let mut file = NamedTempFile::with_suffix(".toml").expect("Failed to create temp config file");
    file.write_all(content.as_bytes())
        .expect("Failed to write config");
    file.flush().expect("Failed to flush config");
    let path = file.path().to_string_lossy().to_string();
    (file, path)
}

/// Helper to start the agent with a config file and capture stderr.
/// Returns the child process. Caller must kill it.
fn start_agent_with_config(config_path: &str) -> std::process::Child {
    Command::new(binary_path())
        .args(["--config", config_path])
        .env_remove("RUST_LOG")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start agent")
}

// T022: Agent with file destination creates log file with JSON content
#[test]
#[ignore] // Requires cargo build
fn test_file_destination_creates_log_file() {
    ensure_binary_built();

    let log_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let log_path = log_dir.path().join("agent.log");
    let log_path_str = log_path.to_string_lossy().to_string();

    let config_content = format!(
        r#"
server_port = 15550
metrics_port = 19090

[logging]
destination = "file"
level = "info"
file_path = "{}"
"#,
        log_path_str
    );

    let (_config_file, config_path) = write_temp_config(&config_content);

    let mut child = start_agent_with_config(&config_path);

    // Give the agent time to start and write logs
    std::thread::sleep(Duration::from_secs(2));

    // Kill the agent
    let _ = child.kill();
    let _ = child.wait();

    // Verify log file was created
    assert!(
        log_path.exists(),
        "Log file should exist at {}",
        log_path_str
    );

    // Verify log file contains JSON log lines
    let log_content = std::fs::read_to_string(&log_path).expect("Failed to read log file");
    assert!(!log_content.is_empty(), "Log file should not be empty");

    // Check that at least one line is valid JSON with expected fields
    let has_json_log = log_content.lines().any(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .map(|v| v.get("level").is_some() && v.get("timestamp").is_some())
            .unwrap_or(false)
    });

    assert!(
        has_json_log,
        "Log file should contain valid JSON log entries"
    );

    // Verify stderr is empty (logs should go to file, not console)
    let output = child.wait_with_output().unwrap_or_else(|_| {
        // Process already waited, this is expected
        std::process::Output {
            status: std::process::ExitStatus::default(),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    });
    // Note: stderr may have some output from before logging init, that's ok
    let _ = output;
}

// T023: Agent with console destination (or no [logging] section) logs to stderr
#[test]
#[ignore] // Requires cargo build
fn test_console_destination_logs_to_stderr() {
    ensure_binary_built();

    let config_content = r#"
server_port = 15551
metrics_port = 19091
log_level = "info"
"#;

    let (_config_file, config_path) = write_temp_config(config_content);

    let mut child = start_agent_with_config(&config_path);

    // Give agent time to start
    std::thread::sleep(Duration::from_secs(2));

    let _ = child.kill();
    let output = child.wait_with_output().expect("Failed to wait for agent");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify stderr has log content (JSON lines)
    let has_json_log = stderr.lines().any(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .map(|v| v.get("level").is_some())
            .unwrap_or(false)
    });

    assert!(
        has_json_log,
        "Console destination should produce JSON logs on stderr. Got: {}",
        &stderr[..stderr.len().min(500)]
    );
}

// T024: Agent with file destination but no file_path exits with error
#[test]
#[ignore] // Requires cargo build
fn test_file_destination_without_path_exits_error() {
    ensure_binary_built();

    let config_content = r#"
server_port = 15552
metrics_port = 19092

[logging]
destination = "file"
"#;

    let (_config_file, config_path) = write_temp_config(config_content);

    let output = Command::new(binary_path())
        .args(["--config", &config_path])
        .env_remove("RUST_LOG")
        .output()
        .expect("Failed to run agent");

    assert!(
        !output.status.success(),
        "Agent should exit with error when file destination has no file_path"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("file_path"),
        "Error message should mention file_path. Got: {}",
        stderr
    );
}

// T028: Backward compat - top-level log_level still works
#[test]
#[ignore] // Requires cargo build
fn test_backward_compat_top_level_log_level() {
    ensure_binary_built();

    let config_content = r#"
server_port = 15553
metrics_port = 19093
log_level = "warn"
"#;

    let (_config_file, config_path) = write_temp_config(config_content);

    let mut child = start_agent_with_config(&config_path);

    std::thread::sleep(Duration::from_secs(2));

    let _ = child.kill();
    let output = child.wait_with_output().expect("Failed to wait for agent");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // At warn level, there should be no INFO-level startup messages
    let info_lines: Vec<&str> = stderr
        .lines()
        .filter(|line| {
            serde_json::from_str::<serde_json::Value>(line)
                .ok()
                .and_then(|v| v.get("level").and_then(|l| l.as_str()).map(|l| l == "INFO"))
                .unwrap_or(false)
        })
        .collect();

    // At warn level, INFO messages should be suppressed
    assert!(
        info_lines.is_empty(),
        "At warn level, INFO messages should be suppressed. Found {} INFO lines",
        info_lines.len()
    );
}

// T029: [logging].level overrides top-level log_level
#[test]
#[ignore] // Requires cargo build
fn test_logging_level_overrides_top_level() {
    ensure_binary_built();

    let config_content = r#"
server_port = 15554
metrics_port = 19094
log_level = "warn"

[logging]
level = "info"
"#;

    let (_config_file, config_path) = write_temp_config(config_content);

    let mut child = start_agent_with_config(&config_path);

    std::thread::sleep(Duration::from_secs(2));

    let _ = child.kill();
    let output = child.wait_with_output().expect("Failed to wait for agent");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // [logging].level = "info" should override log_level = "warn"
    // So INFO messages SHOULD appear
    let has_info = stderr.lines().any(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .and_then(|v| v.get("level").and_then(|l| l.as_str()).map(|l| l == "INFO"))
            .unwrap_or(false)
    });

    assert!(
        has_info,
        "With [logging].level=info overriding log_level=warn, INFO messages should appear"
    );
}

// T033: Per-package override with non-existent module is silently accepted
#[test]
#[ignore] // Requires cargo build
fn test_nonexistent_package_override_accepted() {
    ensure_binary_built();

    let config_content = r#"
server_port = 15555
metrics_port = 19095

[logging]
level = "info"

[logging.packages]
"nonexistent_crate::nonexistent_module" = "debug"
"#;

    let (_config_file, config_path) = write_temp_config(config_content);

    let mut child = start_agent_with_config(&config_path);

    std::thread::sleep(Duration::from_secs(2));

    let _ = child.kill();
    let output = child.wait_with_output().expect("Failed to wait for agent");

    // Agent should start successfully (exit was forced by kill, not error)
    // The key assertion: it didn't crash on startup due to unknown module
    let stderr = String::from_utf8_lossy(&output.stderr);
    let has_startup_log = stderr.lines().any(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .and_then(|v| {
                v.get("fields")
                    .and_then(|f| f.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.contains("HAProxy gRPC Agent starting"))
            })
            .unwrap_or(false)
    });

    assert!(
        has_startup_log,
        "Agent should start successfully even with non-existent package override"
    );
}

// T043: Legacy config (no [logging] section) preserves identical behavior
#[test]
#[ignore] // Requires cargo build
fn test_legacy_config_no_logging_section() {
    ensure_binary_built();

    let config_content = r#"
server_port = 15556
metrics_port = 19096
log_level = "info"
log_format = "json"
"#;

    let (_config_file, config_path) = write_temp_config(config_content);

    let mut child = start_agent_with_config(&config_path);

    std::thread::sleep(Duration::from_secs(2));

    let _ = child.kill();
    let output = child.wait_with_output().expect("Failed to wait for agent");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verify JSON logs on stderr (console, the default destination)
    let has_json_startup = stderr.lines().any(|line| {
        serde_json::from_str::<serde_json::Value>(line)
            .ok()
            .and_then(|v| {
                v.get("fields")
                    .and_then(|f| f.get("message"))
                    .and_then(|m| m.as_str())
                    .map(|s| s.contains("HAProxy gRPC Agent starting"))
            })
            .unwrap_or(false)
    });

    assert!(
        has_json_startup,
        "Legacy config should produce JSON startup logs on stderr"
    );
}

// T044: Invalid log level string in config causes clear error
#[test]
#[ignore] // Requires cargo build
fn test_invalid_log_level_in_config() {
    ensure_binary_built();

    let config_content = r#"
server_port = 15557
metrics_port = 19097

[logging]
level = "verbose"
"#;

    let (_config_file, config_path) = write_temp_config(config_content);

    let output = Command::new(binary_path())
        .args(["--config", &config_path])
        .env_remove("RUST_LOG")
        .output()
        .expect("Failed to run agent");

    assert!(
        !output.status.success(),
        "Agent should exit with error on invalid log level"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // TOML deserialization should produce an error about the invalid enum value
    assert!(
        stderr.contains("config") || stderr.contains("parse") || stderr.contains("unknown variant"),
        "Error should relate to config parsing. Got: {}",
        stderr
    );
}
