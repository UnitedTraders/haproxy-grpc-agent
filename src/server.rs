// TCP server module for Agent Text Protocol
// T067-T076: Complete TCP server implementation

use crate::checker::GrpcHealthChecker;
use crate::config::AgentConfig;
use crate::protocol;
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use uuid::Uuid;

pub struct AgentServer {
    config: AgentConfig,
    health_checker: Arc<GrpcHealthChecker>,
}

impl AgentServer {
    pub fn new(config: AgentConfig) -> Self {
        let health_checker = Arc::new(GrpcHealthChecker::new(config.clone()));

        AgentServer {
            config,
            health_checker,
        }
    }

    // T067-T069: TCP server with connection accept loop
    pub async fn run(&self) -> Result<()> {
        // T068: Bind to configured address and port
        let bind_addr = format!("{}:{}", self.config.server_bind_address, self.config.server_port);

        let listener = TcpListener::bind(&bind_addr)
            .await
            .with_context(|| format!("Failed to bind to {}", bind_addr))?;

        tracing::info!(
            address = %bind_addr,
            "Agent Text Protocol server listening"
        );

        // T069: Connection accept loop spawning tasks per connection
        loop {
            match listener.accept().await {
                Ok((stream, peer_addr)) => {
                    tracing::info!(
                        peer = %peer_addr,
                        "HAProxy connection established"
                    );

                    let health_checker = Arc::clone(&self.health_checker);

                    // Spawn a task to handle this connection
                    tokio::spawn(async move {
                        if let Err(e) = handle_connection(stream, health_checker).await {
                            tracing::warn!(
                                peer = %peer_addr,
                                error = %e,
                                "Connection handling error"
                            );
                        }

                        tracing::info!(
                            peer = %peer_addr,
                            "HAProxy connection closed"
                        );
                    });
                }
                Err(e) => {
                    tracing::error!(error = %e, "Failed to accept connection");
                    // Continue accepting other connections
                }
            }
        }
    }
}

// T070-T076: handle_connection function
#[tracing::instrument(skip(stream, health_checker), fields(trace_id = %Uuid::new_v4()))]
async fn handle_connection(
    stream: TcpStream,
    health_checker: Arc<GrpcHealthChecker>,
) -> Result<()> {
    let peer_addr = stream.peer_addr().ok();
    let mut reader = BufReader::new(stream);

    // T071: Persistent connection handling (loop over requests)
    loop {
        let mut line = String::new();

        // Read line from TCP stream
        let bytes_read = reader
            .read_line(&mut line)
            .await
            .context("Failed to read from TCP stream")?;

        // T072: Graceful connection close detection (EOF)
        if bytes_read == 0 {
            tracing::debug!("Connection closed by peer (EOF)");
            break;
        }

        // T074: Integrate protocol::parse_request
        match protocol::parse_request(&line) {
            Ok(request) => {
                tracing::info!(
                    backend = %format!("{}:{}", request.backend_server, request.backend_port),
                    ssl = ?request.ssl_flag,
                    "Processing health check request"
                );

                // T075: Integrate checker::check_backend
                let response = health_checker.check_backend(&request).await;

                tracing::info!(
                    backend = %format!("{}:{}", request.backend_server, request.backend_port),
                    status = ?response.status,
                    "Health check completed"
                );

                // T076: Write response to TCP stream
                let response_str = response.to_string();
                reader
                    .get_mut()
                    .write_all(response_str.as_bytes())
                    .await
                    .context("Failed to write response")?;

                reader
                    .get_mut()
                    .flush()
                    .await
                    .context("Failed to flush response")?;
            }
            Err(e) => {
                // Protocol violation - log warning and return down
                tracing::warn!(
                    peer = ?peer_addr,
                    error = %e,
                    input = %line.trim(),
                    "Protocol violation"
                );

                // Return down for protocol violations
                let response = "down\n";
                if let Err(write_err) = reader.get_mut().write_all(response.as_bytes()).await {
                    tracing::error!(error = %write_err, "Failed to write error response");
                    // T073: Abrupt disconnect - break on write failure
                    break;
                }

                if let Err(flush_err) = reader.get_mut().flush().await {
                    tracing::error!(error = %flush_err, "Failed to flush error response");
                    break;
                }
            }
        }
    }

    Ok(())
}
