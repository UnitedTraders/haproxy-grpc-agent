// T012: Basic tokio main function skeleton
// T077: Wire together config, logger, and server

mod checker;
mod config;
mod logger;
mod metrics;
mod protocol;
mod server;

use anyhow::{Context, Result};

#[tokio::main]
async fn main() -> Result<()> {
    // T077: Load configuration
    let config = config::AgentConfig::load().context("Failed to load configuration")?;

    // T077: Initialize logger based on config
    logger::init(&config).context("Failed to initialize logger")?;

    // T077: Log startup configuration
    tracing::info!(
        server_bind = %format!("{}:{}", config.server_bind_address, config.server_port),
        grpc_connect_timeout_ms = config.grpc_connect_timeout_ms,
        grpc_rpc_timeout_ms = config.grpc_rpc_timeout_ms,
        log_level = ?config.log_level,
        log_format = ?config.log_format,
        "HAProxy gRPC Agent starting"
    );

    // T077: Create and run TCP server
    let server = server::AgentServer::new(config);

    tracing::info!("Initialization complete, starting server");

    server.run().await.context("Server error")?;

    Ok(())
}
