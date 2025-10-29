// T018: logger.rs module file
// Structured logging module using tracing
// Configures JSON logging with trace IDs

use crate::config::{AgentConfig, LogFormat, LogLevel};
use anyhow::Result;
use tracing_subscriber::EnvFilter;

/// Initialize the logging system based on configuration
pub fn init(config: &AgentConfig) -> Result<()> {
    let log_level = match config.log_level {
        LogLevel::Trace => "trace",
        LogLevel::Debug => "debug",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
    };

    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(log_level));

    match config.log_format {
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .json()
                .with_current_span(false)
                .with_span_list(true)
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::fmt().with_env_filter(env_filter).init();
        }
    }

    Ok(())
}
