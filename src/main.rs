// T012: Basic tokio main function skeleton
// T077: Wire together config, logger, and server

mod checker;
mod config;
mod logger;
mod metrics;
mod protocol;
mod server;

use anyhow::{Context, Result};
use tokio::signal;

// T138: Graceful shutdown signal handler
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install SIGTERM handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            tracing::info!("Received Ctrl+C signal");
        }
        _ = terminate => {
            tracing::info!("Received SIGTERM signal");
        }
    }

    tracing::info!("Starting graceful shutdown...");
}

#[tokio::main]
async fn main() -> Result<()> {
    // T077: Load configuration
    let config = config::AgentConfig::load().context("Failed to load configuration")?;

    // T077: Initialize logger based on config
    logger::init(&config).context("Failed to initialize logger")?;

    // T077: Log startup configuration
    tracing::info!(
        server_bind = %format!("{}:{}", config.server_bind_address, config.server_port),
        metrics_port = config.metrics_port,
        grpc_connect_timeout_ms = config.grpc_connect_timeout_ms,
        grpc_rpc_timeout_ms = config.grpc_rpc_timeout_ms,
        grpc_channel_cache_enabled = config.grpc_channel_cache_enabled,
        log_level = ?config.log_level,
        log_format = ?config.log_format,
        "HAProxy gRPC Agent starting"
    );

    // T128: Create metrics server
    // T129: Graceful degradation - metrics failure doesn't stop health checks
    let metrics_server_result = metrics::MetricsServer::new(&config);
    let metrics_handle = match metrics_server_result {
        Ok(metrics_server) => {
            tracing::info!("Metrics server configured successfully");
            Some(tokio::spawn(async move {
                if let Err(e) = metrics_server.run().await {
                    tracing::error!(error = %e, "Metrics server error - continuing without metrics");
                }
            }))
        }
        Err(e) => {
            // T129: Graceful degradation - log error but continue
            tracing::warn!(
                error = %e,
                "Failed to initialize metrics server - continuing without metrics"
            );
            None
        }
    };

    // T077: Create and run TCP server
    let server = server::AgentServer::new(config);

    tracing::info!("Initialization complete, starting server");

    // T138: Run server with graceful shutdown
    let server_handle = tokio::spawn(async move { server.run().await });

    // T138: Wait for either server to exit or shutdown signal
    tokio::select! {
        result = server_handle => {
            if let Ok(Err(e)) = result {
                tracing::error!(error = %e, "Server error");
            }
        }
        _ = shutdown_signal() => {
            tracing::info!("Shutdown signal received, stopping server...");
        }
    }

    // Clean up metrics server
    if let Some(handle) = metrics_handle {
        handle.abort();
    }

    tracing::info!("Shutdown complete");
    Ok(())
}
