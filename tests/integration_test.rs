// T052-T056: Integration tests for HAProxy gRPC Agent
// Tests the full end-to-end flow: TCP server → gRPC client → backend

use std::net::SocketAddr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::sleep;

// Helper function to connect to agent and send request
async fn send_agent_request(
    addr: SocketAddr,
    request: &str,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut stream = TcpStream::connect(addr).await?;

    // Send request
    stream.write_all(request.as_bytes()).await?;
    stream.flush().await?;

    // Read response
    let mut reader = BufReader::new(stream);
    let mut response = String::new();
    reader.read_line(&mut response).await?;

    Ok(response)
}

// T052: Test basic connectivity to agent
#[tokio::test]
#[ignore] // Requires mock backend running
async fn test_agent_connectivity() {
    let agent_addr: SocketAddr = "127.0.0.1:5555".parse().unwrap();

    // Wait for agent to start
    sleep(Duration::from_secs(2)).await;

    let result = TcpStream::connect(agent_addr).await;
    assert!(result.is_ok(), "Should connect to agent TCP server");
}

// T053: Test health check with healthy backend
#[tokio::test]
#[ignore] // Requires mock backend running
async fn test_health_check_healthy_backend() {
    let agent_addr: SocketAddr = "127.0.0.1:5555".parse().unwrap();

    // Wait for services to start
    sleep(Duration::from_secs(2)).await;

    let request = "localhost 50051 no-ssl localhost\n";
    let response = send_agent_request(agent_addr, request)
        .await
        .expect("Should receive response");

    assert_eq!(response.trim(), "up", "Healthy backend should return 'up'");
}

// T054: Test health check with SSL flag
#[tokio::test]
#[ignore] // Requires mock backend with TLS running
async fn test_health_check_with_ssl() {
    let agent_addr: SocketAddr = "127.0.0.1:5555".parse().unwrap();

    sleep(Duration::from_secs(2)).await;

    let request = "localhost 50052 ssl localhost\n";
    let response = send_agent_request(agent_addr, request)
        .await
        .expect("Should receive response");

    // Should fail to connect to non-existent SSL backend
    assert_eq!(
        response.trim(),
        "down",
        "Non-existent backend should return 'down'"
    );
}

// T055: Test protocol violation handling
#[tokio::test]
#[ignore] // Requires agent running
async fn test_protocol_violation() {
    let agent_addr: SocketAddr = "127.0.0.1:5555".parse().unwrap();

    sleep(Duration::from_secs(2)).await;

    // Send invalid request (wrong number of fields)
    let request = "invalid request\n";
    let response = send_agent_request(agent_addr, request)
        .await
        .expect("Should receive response");

    assert_eq!(
        response.trim(),
        "down",
        "Protocol violation should return 'down'"
    );
}

// T056: Test persistent connection (multiple requests)
#[tokio::test]
#[ignore] // Requires mock backend running
async fn test_persistent_connection() {
    let agent_addr: SocketAddr = "127.0.0.1:5555".parse().unwrap();

    sleep(Duration::from_secs(2)).await;

    let stream = TcpStream::connect(agent_addr)
        .await
        .expect("Should connect to agent");

    let mut reader = BufReader::new(stream);

    // Send multiple requests on same connection
    for _ in 0..3 {
        let request = "localhost 50051 no-ssl localhost\n";
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

        assert_eq!(response.trim(), "up", "Should receive 'up' response");
        response.clear();
    }
}

// T056: Test unreachable backend
#[tokio::test]
#[ignore] // Requires agent running
async fn test_unreachable_backend() {
    let agent_addr: SocketAddr = "127.0.0.1:5555".parse().unwrap();

    sleep(Duration::from_secs(2)).await;

    // Try to check a backend that doesn't exist
    let request = "nonexistent.example.com 9999 no-ssl nonexistent.example.com\n";
    let response = send_agent_request(agent_addr, request)
        .await
        .expect("Should receive response");

    assert_eq!(
        response.trim(),
        "down",
        "Unreachable backend should return 'down'"
    );
}
