// Integration tests for channel cache configuration
// Tests cache-enabled and cache-disabled behavior

mod common;

use common::{cleanup_agent, send_check, start_agent_with_config, start_mock_backend};
use haproxy_grpc_agent::config::AgentConfig;

// T009: Test that health checks work with caching disabled
#[tokio::test]
async fn test_health_check_cache_disabled() {
    let (_container, backend_port) = start_mock_backend("SERVING").await;

    let config = AgentConfig {
        server_port: 0,
        server_bind_address: "127.0.0.1".to_string(),
        metrics_port: 0,
        metrics_bind_address: "127.0.0.1".to_string(),
        grpc_channel_cache_enabled: false,
        ..AgentConfig::default()
    };

    let (handle, agent_addr) = start_agent_with_config(config).await;

    // Send two consecutive checks — both should succeed with fresh channels
    let response1 = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(
        response1, "up",
        "First check with cache disabled should return 'up'"
    );

    let response2 = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(
        response2, "up",
        "Second check with cache disabled should return 'up'"
    );

    cleanup_agent(handle);
}

// T010: Test that default config (cache enabled) preserves existing behavior
#[tokio::test]
async fn test_cache_enabled_default_behavior() {
    let (_container, backend_port) = start_mock_backend("SERVING").await;

    let config = AgentConfig {
        server_port: 0,
        server_bind_address: "127.0.0.1".to_string(),
        metrics_port: 0,
        metrics_bind_address: "127.0.0.1".to_string(),
        grpc_channel_cache_enabled: true,
        ..AgentConfig::default()
    };

    let (handle, agent_addr) = start_agent_with_config(config).await;

    // Send two consecutive checks — both should succeed (second uses cached channel)
    let response1 = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(
        response1, "up",
        "First check with cache enabled should return 'up'"
    );

    let response2 = send_check(agent_addr, "127.0.0.1", backend_port).await;
    assert_eq!(
        response2, "up",
        "Second check with cache enabled should return 'up' (cached channel)"
    );

    cleanup_agent(handle);
}

// T011: Test that unreachable backend returns 'down' with caching disabled
#[tokio::test]
async fn test_cache_disabled_unreachable_backend() {
    let config = AgentConfig {
        server_port: 0,
        server_bind_address: "127.0.0.1".to_string(),
        metrics_port: 0,
        metrics_bind_address: "127.0.0.1".to_string(),
        grpc_channel_cache_enabled: false,
        ..AgentConfig::default()
    };

    let (handle, agent_addr) = start_agent_with_config(config).await;

    let response = send_check(agent_addr, "nonexistent.example.com", 9999).await;
    assert_eq!(
        response, "down",
        "Unreachable backend with cache disabled should return 'down'"
    );

    cleanup_agent(handle);
}
