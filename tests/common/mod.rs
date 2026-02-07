// Shared test utilities for integration and resilience tests

use haproxy_grpc_agent::config::AgentConfig;
use haproxy_grpc_agent::server::AgentServer;
use std::net::SocketAddr;
use std::sync::Once;
use std::time::Duration;
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

static BUILD_MOCK_IMAGE: Once = Once::new();

/// Ensures the mock-grpc-backend Docker image is built exactly once per test process.
pub fn build_mock_image() {
    BUILD_MOCK_IMAGE.call_once(|| {
        let output = std::process::Command::new("docker")
            .args([
                "build",
                "-t",
                "mock-grpc-backend:latest",
                "tests/integration/mock-backend/",
            ])
            .output()
            .expect("Failed to execute docker build command");

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!("Failed to build mock-grpc-backend Docker image: {}", stderr);
        }
    });
}

/// Starts a mock gRPC backend container with the given health status.
/// Returns the container handle and the dynamically mapped host port.
pub async fn start_mock_backend(health_status: &str) -> (ContainerAsync<GenericImage>, u16) {
    build_mock_image();

    let container = GenericImage::new("mock-grpc-backend", "latest")
        .with_exposed_port(50051.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Mock gRPC backend starting"))
        .with_env_var("HEALTH_STATUS", health_status)
        .with_env_var("GRPC_PORT", "50051")
        .start()
        .await
        .expect("Failed to start mock-grpc-backend container");

    let port = container
        .get_host_port_ipv4(50051)
        .await
        .expect("Failed to get mapped port for mock backend");

    (container, port)
}

/// Starts the agent server in-process on a dynamic port.
/// Returns the tokio JoinHandle and the bound SocketAddr.
pub async fn start_agent() -> (tokio::task::JoinHandle<anyhow::Result<()>>, SocketAddr) {
    let config = AgentConfig {
        server_port: 0,
        server_bind_address: "127.0.0.1".to_string(),
        metrics_port: 0,
        metrics_bind_address: "127.0.0.1".to_string(),
        ..AgentConfig::default()
    };

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind agent listener");
    let addr = listener
        .local_addr()
        .expect("Failed to get agent bound address");

    let server = AgentServer::new(config);
    let handle = tokio::spawn(async move { server.run_with_listener(listener).await });

    // Brief pause to let the accept loop start
    tokio::time::sleep(Duration::from_millis(50)).await;

    (handle, addr)
}

/// Sends a health check request to the agent and returns the trimmed response.
pub async fn send_check(agent_addr: SocketAddr, backend_host: &str, backend_port: u16) -> String {
    let request = format!(
        "{} {} no-ssl {}\n",
        backend_host, backend_port, backend_host
    );
    send_raw_request(agent_addr, &request).await
}

/// Sends a raw request string to the agent and returns the trimmed response.
pub async fn send_raw_request(agent_addr: SocketAddr, request: &str) -> String {
    let result = tokio::time::timeout(Duration::from_secs(5), async {
        let mut stream = TcpStream::connect(agent_addr)
            .await
            .expect("Failed to connect to agent");

        stream
            .write_all(request.as_bytes())
            .await
            .expect("Failed to write request to agent");
        stream.flush().await.expect("Failed to flush agent stream");

        let mut reader = BufReader::new(stream);
        let mut response = String::new();
        reader
            .read_line(&mut response)
            .await
            .expect("Failed to read response from agent");

        response.trim().to_string()
    })
    .await;

    result.expect("Agent request timed out after 5 seconds")
}

/// Aborts the agent server task.
pub fn cleanup_agent(handle: tokio::task::JoinHandle<anyhow::Result<()>>) {
    handle.abort();
}
