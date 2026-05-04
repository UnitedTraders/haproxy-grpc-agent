// T018: logger.rs module file
// Structured logging module using tracing
// Configures JSON logging with trace IDs and configurable output destination

use crate::config::{AgentConfig, LogDestination, LogFormat, LogRotation};
use anyhow::Result;
use tracing_appender::rolling;
use tracing_subscriber::EnvFilter;

/// Initialize the logging system based on configuration.
/// Supports console (stderr) and file output destinations with optional rotation.
pub fn init(config: &AgentConfig) -> Result<()> {
    let directive = config.logging.build_env_filter_directive(&config.log_level);
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&directive));

    let format = config.logging.resolved_format(&config.log_format);

    match config.logging.destination {
        LogDestination::Console => init_console(env_filter, format),
        LogDestination::File => {
            let file_path = config
                .logging
                .file_path
                .as_deref()
                .expect("file_path validated in config");
            init_file(
                env_filter,
                format,
                file_path,
                config.logging.file_rotation,
                config.logging.file_max_files,
            )?;
        }
    }

    Ok(())
}

fn init_console(env_filter: EnvFilter, format: LogFormat) {
    match format {
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .json()
                .with_current_span(false)
                .with_span_list(true)
                .with_writer(std::io::stderr)
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_writer(std::io::stderr)
                .init();
        }
    }
}

fn init_file(
    env_filter: EnvFilter,
    format: LogFormat,
    file_path: &str,
    rotation: Option<LogRotation>,
    max_files: Option<usize>,
) -> Result<()> {
    let path = std::path::Path::new(file_path);

    // Ensure parent directory exists
    if let Some(parent) = path.parent()
        && !parent.exists()
    {
        std::fs::create_dir_all(parent)
            .map_err(|e| anyhow::anyhow!("Failed to create log directory {:?}: {}", parent, e))?;
    }

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid log file path: {}", file_path))?;

    let dir = path.parent().ok_or_else(|| {
        anyhow::anyhow!("Invalid log file path (no parent directory): {}", file_path)
    })?;

    // Determine rotation strategy
    let tracing_rotation = match rotation {
        Some(LogRotation::Daily) => rolling::Rotation::DAILY,
        Some(LogRotation::Hourly) => rolling::Rotation::HOURLY,
        Some(LogRotation::Never) | None => rolling::Rotation::NEVER,
    };

    // Build the file appender with optional rotation and retention
    let mut builder = rolling::Builder::new()
        .rotation(tracing_rotation)
        .filename_prefix(file_name);

    if let Some(max) = max_files {
        builder = builder.max_log_files(max);
    }

    let file_appender = builder
        .build(dir)
        .map_err(|e| anyhow::anyhow!("Failed to create log file appender: {}", e))?;

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // Leak the guard so it lives for the lifetime of the program.
    // This is intentional — the guard must not be dropped or logs will stop being written.
    std::mem::forget(_guard);

    match format {
        LogFormat::Json => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .json()
                .with_current_span(false)
                .with_span_list(true)
                .with_writer(non_blocking)
                .init();
        }
        LogFormat::Pretty => {
            tracing_subscriber::fmt()
                .with_env_filter(env_filter)
                .with_writer(non_blocking)
                .init();
        }
    }

    Ok(())
}
