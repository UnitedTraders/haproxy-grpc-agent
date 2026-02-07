// Integration tests for HAProxy gRPC Agent using testcontainers
// Tests the full end-to-end flow: TCP server → gRPC client → backend

mod common;

use common::{cleanup_agent, send_check, send_raw_request, start_agent, start_mock_backend};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

// Test basic connectivity to agent
#[tokio::test]
async fn test_agent_connectivity() {
    let (_container, _backend_port) = start_mock_backend("SERVING").await;
    let (handle, agent_addr) = start_agent().await;

    let result = TcpStream::connect(agent_addr).await;
    assert!(result.is_ok(), "Should connect to agent TCP server");

    cleanup_agent(handle);
}

// Test health check with healthy backend
#[tokio::test]
async fn test_health_check_healthy_backend() {
    let (_container, backend_port) = start_mock_backend("SERVING").await;
    let (handle, agent_addr) = start_agent().await;

    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(response, "up", "Healthy backend should return 'up'");

    cleanup_agent(handle);
}

// Test health check with SSL flag (non-existent SSL backend)
#[tokio::test]
async fn test_health_check_with_ssl() {
    let (handle, agent_addr) = start_agent().await;

    let response = send_raw_request(agent_addr, "127.0.0.1 50052 ssl 127.0.0.1\n").await;
    assert_eq!(
        response, "down",
        "Non-existent SSL backend should return 'down'"
    );

    cleanup_agent(handle);
}

// Test protocol violation handling
#[tokio::test]
async fn test_protocol_violation() {
    let (handle, agent_addr) = start_agent().await;

    let response = send_raw_request(agent_addr, "invalid request\n").await;
    assert_eq!(response, "down", "Protocol violation should return 'down'");

    cleanup_agent(handle);
}

// Test persistent connection (multiple requests on same TCP stream)
#[tokio::test]
async fn test_persistent_connection() {
    let (_container, backend_port) = start_mock_backend("SERVING").await;
    let (handle, agent_addr) = start_agent().await;

    let stream = TcpStream::connect(agent_addr)
        .await
        .expect("Should connect to agent");

    let mut reader = BufReader::new(stream);

    for i in 0..3 {
        let request = format!("127.0.0.1 {} no-ssl 127.0.0.1\n", backend_port);
        reader
            .get_mut()
            .write_all(request.as_bytes())
            .await
            .expect("Should write request");
        reader.get_mut().flush().await.expect("Should flush");

        let mut response = String::new();
        reader
            .read_line(&mut response)
            .await
            .expect("Should read response");

        assert_eq!(
            response.trim(),
            "up",
            "Request {} should receive 'up' response",
            i + 1
        );
        response.clear();
    }

    cleanup_agent(handle);
}

// Test unreachable backend
#[tokio::test]
async fn test_unreachable_backend() {
    let (handle, agent_addr) = start_agent().await;

    let response = send_check(agent_addr, "nonexistent.example.com", 9999).await;
    assert_eq!(response, "down", "Unreachable backend should return 'down'");

    cleanup_agent(handle);
}
