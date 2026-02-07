// Resilience tests for HAProxy gRPC Agent
// Tests backend disconnect, recovery, and reload scenarios
// These tests operate without HAProxy — only agent and mock backend

mod common;

use common::{cleanup_agent, send_check, start_agent, start_mock_backend};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::sleep;

/// Helper: retry sending a check until expected response or timeout.
/// Backend restart needs time — gRPC channel reconnection is not instant.
async fn send_check_with_retry(
    agent_addr: std::net::SocketAddr,
    host: &str,
    port: u16,
    expected: &str,
    max_retries: usize,
) -> String {
    let mut last_response = String::new();
    for _ in 0..max_retries {
        last_response = send_check(agent_addr, host, port).await;
        if last_response == expected {
            return last_response;
        }
        sleep(Duration::from_millis(500)).await;
    }
    last_response
}

/// Wait until a TCP port is accepting connections (backend is ready).
async fn wait_for_port(host: &str, port: u16, max_retries: usize) {
    for _ in 0..max_retries {
        if TcpStream::connect(format!("{}:{}", host, port))
            .await
            .is_ok()
        {
            return;
        }
        sleep(Duration::from_millis(500)).await;
    }
    panic!(
        "Port {}:{} did not become available after retries",
        host, port
    );
}

// Test that agent reports "down" when a previously healthy backend is stopped
#[tokio::test]
async fn test_backend_disconnect() {
    let (container, backend_port) = start_mock_backend("SERVING").await;
    let (handle, agent_addr) = start_agent().await;

    // Verify backend is initially healthy
    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(response, "up", "Backend should initially be up");

    // Stop the backend container (simulates crash/disconnect)
    container
        .stop()
        .await
        .expect("Failed to stop mock backend");
    sleep(Duration::from_millis(500)).await;

    // Agent should now report the backend as down
    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(
        response, "down",
        "Agent should report 'down' after backend disconnect"
    );

    cleanup_agent(handle);
}

// Test that agent recovers after backend restart
#[tokio::test]
async fn test_backend_recovery() {
    let (container, backend_port) = start_mock_backend("SERVING").await;
    let (handle, agent_addr) = start_agent().await;

    // Verify backend is initially healthy
    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(response, "up", "Backend should initially be up");

    // Stop the backend
    container
        .stop()
        .await
        .expect("Failed to stop mock backend");
    sleep(Duration::from_millis(500)).await;

    // Verify backend is down
    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(response, "down", "Backend should be down after stop");

    // Restart the backend — Docker reassigns the host port mapping
    container
        .start()
        .await
        .expect("Failed to restart mock backend");

    // Docker assigns a new host port after restart — re-query it
    let new_port = container
        .get_host_port_ipv4(50051)
        .await
        .expect("Failed to get new port after restart");

    // Wait for the new port to accept TCP connections
    wait_for_port("127.0.0.1", new_port, 20).await;

    // Agent should connect to the new port and report "up"
    let response =
        send_check_with_retry(agent_addr, "127.0.0.1", new_port, "up", 20).await;
    assert_eq!(
        response, "up",
        "Agent should report 'up' after backend recovery"
    );

    cleanup_agent(handle);
}

// Test that cached gRPC channel is invalidated on disconnect
#[tokio::test]
async fn test_cached_connection_invalidated_on_disconnect() {
    let (container, backend_port) = start_mock_backend("SERVING").await;
    let (handle, agent_addr) = start_agent().await;

    // Send checks twice to prime the gRPC channel cache
    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(response, "up", "First check should be up");

    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(response, "up", "Second check (cached channel) should be up");

    // Stop backend — cached channel should become invalid
    container
        .stop()
        .await
        .expect("Failed to stop mock backend");
    sleep(Duration::from_millis(500)).await;

    // Agent must NOT return stale "up" from cached channel
    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(
        response, "down",
        "Agent should report 'down' (not stale 'up' from cache) after disconnect"
    );

    cleanup_agent(handle);
}

// Test that agent reflects changed health status after backend reload
#[tokio::test]
async fn test_backend_status_change_on_reload() {
    let (container, backend_port) = start_mock_backend("SERVING").await;
    let (handle, agent_addr) = start_agent().await;

    // Verify initially healthy
    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(response, "up", "Backend should initially be up (SERVING)");

    // Stop the original container
    container
        .stop()
        .await
        .expect("Failed to stop original backend");
    sleep(Duration::from_millis(500)).await;

    // Start a NEW container with NOT_SERVING status
    // (cannot change env vars on stopped container, so we create a new one)
    let (_container2, backend_port2) = start_mock_backend("NOT_SERVING").await;

    // Check the new backend — should report down (NOT_SERVING)
    let response = send_check(agent_addr, "127.0.0.1", backend_port2).await;
    assert_eq!(
        response, "down",
        "Agent should report 'down' for NOT_SERVING backend"
    );

    cleanup_agent(handle);
}

// Test that agent reconnects after backend restart with same status
#[tokio::test]
async fn test_backend_restart_with_same_status() {
    let (container, backend_port) = start_mock_backend("SERVING").await;
    let (handle, agent_addr) = start_agent().await;

    // Verify initially healthy
    let response = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(response, "up", "Backend should initially be up");

    // Stop and restart with same status
    container
        .stop()
        .await
        .expect("Failed to stop mock backend");
    container
        .start()
        .await
        .expect("Failed to restart mock backend");

    // Docker assigns a new host port after restart — re-query it
    let new_port = container
        .get_host_port_ipv4(50051)
        .await
        .expect("Failed to get new port after restart");

    // Wait for the new port to accept TCP connections
    wait_for_port("127.0.0.1", new_port, 20).await;

    // Agent should connect to the new port and report "up"
    let response =
        send_check_with_retry(agent_addr, "127.0.0.1", new_port, "up", 20).await;
    assert_eq!(
        response, "up",
        "Agent should report 'up' after restart with same status"
    );

    cleanup_agent(handle);
}
