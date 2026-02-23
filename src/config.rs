// Configuration module for HAProxy gRPC Agent
// T026-T034: Complete configuration implementation

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// T028: LogLevel enum with serde derives
#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LogLevel {
    Trace,
    Debug,
    #[default]
    Info,
    Warn,
    Error,
}

// T028: LogFormat enum with serde derives
#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LogFormat {
    #[default]
    Json,
    Pretty,
}

// T026: AgentConfig struct with all fields from data-model.md
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AgentConfig {
    #[serde(default = "default_server_port")]
    pub server_port: u16,

    #[serde(default = "default_bind_address")]
    pub server_bind_address: String,

    #[serde(default = "default_grpc_connect_timeout")]
    pub grpc_connect_timeout_ms: u64,

    #[serde(default = "default_grpc_rpc_timeout")]
    pub grpc_rpc_timeout_ms: u64,

    #[serde(default = "default_grpc_channel_cache_enabled")]
    pub grpc_channel_cache_enabled: bool,

    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    #[serde(default = "default_bind_address")]
    pub metrics_bind_address: String,

    #[serde(default)]
    pub log_level: LogLevel,

    #[serde(default)]
    pub log_format: LogFormat,
}

// T027: Default functions for AgentConfig
fn default_server_port() -> u16 {
    5555
}

fn default_metrics_port() -> u16 {
    9090
}

fn default_bind_address() -> String {
    "0.0.0.0".to_string()
}

fn default_grpc_connect_timeout() -> u64 {
    1000
}

fn default_grpc_rpc_timeout() -> u64 {
    1500
}

fn default_grpc_channel_cache_enabled() -> bool {
    true
}

impl Default for AgentConfig {
    fn default() -> Self {
        AgentConfig {
            server_port: default_server_port(),
            server_bind_address: default_bind_address(),
            grpc_connect_timeout_ms: default_grpc_connect_timeout(),
            grpc_rpc_timeout_ms: default_grpc_rpc_timeout(),
            grpc_channel_cache_enabled: default_grpc_channel_cache_enabled(),
            metrics_port: default_metrics_port(),
            metrics_bind_address: default_bind_address(),
            log_level: LogLevel::default(),
            log_format: LogFormat::default(),
        }
    }
}

// T031: CLI arguments structure
#[derive(Debug, Parser)]
#[command(name = "haproxy-grpc-agent")]
#[command(about = "HAProxy gRPC Health Check Agent", long_about = None)]
pub struct CliArgs {
    /// Path to configuration file (TOML format)
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// TCP port for Agent Text Protocol server
    #[arg(long)]
    pub server_port: Option<u16>,

    /// Bind address for agent server
    #[arg(long)]
    pub server_bind: Option<String>,

    /// HTTP port for Prometheus metrics
    #[arg(long)]
    pub metrics_port: Option<u16>,

    /// Bind address for metrics server
    #[arg(long)]
    pub metrics_bind: Option<String>,

    /// gRPC connection timeout in milliseconds
    #[arg(long)]
    pub grpc_connect_timeout: Option<u64>,

    /// gRPC RPC timeout in milliseconds
    #[arg(long)]
    pub grpc_rpc_timeout: Option<u64>,

    /// Enable or disable gRPC channel caching (default: true)
    #[arg(long, num_args = 0..=1, default_missing_value = "true", action = clap::ArgAction::Set)]
    pub grpc_channel_cache: Option<bool>,

    /// Log level
    #[arg(long, value_enum)]
    pub log_level: Option<LogLevel>,

    /// Log format
    #[arg(long, value_enum)]
    pub log_format: Option<LogFormat>,
}

impl AgentConfig {
    // T029: Config validation function
    pub fn validate(&self) -> Result<()> {
        // Validate server port
        if self.server_port == 0 {
            anyhow::bail!("server_port must be between 1 and 65535");
        }

        // Validate metrics port
        if self.metrics_port == 0 {
            anyhow::bail!("metrics_port must be between 1 and 65535");
        }

        // Validate ports don't conflict
        if self.server_port == self.metrics_port {
            anyhow::bail!(
                "server_port ({}) and metrics_port ({}) cannot be the same",
                self.server_port,
                self.metrics_port
            );
        }

        // Validate timeouts
        if self.grpc_connect_timeout_ms == 0 {
            anyhow::bail!("grpc_connect_timeout_ms must be greater than 0");
        }

        if self.grpc_rpc_timeout_ms == 0 {
            anyhow::bail!("grpc_rpc_timeout_ms must be greater than 0");
        }

        // Validate total timeout is reasonable (should be < 2000ms for HAProxy)
        let total_timeout = self.grpc_connect_timeout_ms + self.grpc_rpc_timeout_ms;
        if total_timeout >= 2000 {
            eprintln!(
                "WARNING: Total gRPC timeout ({}ms) is >= 2000ms (HAProxy default timeout). \
                Consider reducing timeouts to avoid agent-check timeouts.",
                total_timeout
            );
        }

        Ok(())
    }

    // T030-T033: Load configuration with precedence: CLI > file > env > defaults
    pub fn load() -> Result<Self> {
        // Parse CLI arguments
        let cli_args = CliArgs::parse();

        // Start with defaults
        let mut config = AgentConfig::default();

        // T030: Load from environment variables (if not using config file)
        if cli_args.config.is_none() {
            config = Self::load_from_env(config)?;
        }

        // T032: Load from config file if specified
        if let Some(config_path) = &cli_args.config {
            config = Self::load_from_file(config_path)?;
        }

        // T031: Apply CLI overrides (highest precedence)
        config = Self::apply_cli_overrides(config, cli_args);

        // T034: Fail-fast validation
        config
            .validate()
            .context("Configuration validation failed")?;

        Ok(config)
    }

    // T030: Load configuration from environment variables
    fn load_from_env(mut config: AgentConfig) -> Result<Self> {
        if let Ok(port) = std::env::var("HAPROXY_AGENT_SERVER_PORT") {
            config.server_port = port.parse().context("Invalid HAPROXY_AGENT_SERVER_PORT")?;
        }

        if let Ok(bind) = std::env::var("HAPROXY_AGENT_SERVER_BIND") {
            config.server_bind_address = bind;
        }

        if let Ok(port) = std::env::var("HAPROXY_AGENT_METRICS_PORT") {
            config.metrics_port = port.parse().context("Invalid HAPROXY_AGENT_METRICS_PORT")?;
        }

        if let Ok(bind) = std::env::var("HAPROXY_AGENT_METRICS_BIND") {
            config.metrics_bind_address = bind;
        }

        if let Ok(timeout) = std::env::var("HAPROXY_AGENT_GRPC_CONNECT_TIMEOUT") {
            config.grpc_connect_timeout_ms = timeout
                .parse()
                .context("Invalid HAPROXY_AGENT_GRPC_CONNECT_TIMEOUT")?;
        }

        if let Ok(timeout) = std::env::var("HAPROXY_AGENT_GRPC_RPC_TIMEOUT") {
            config.grpc_rpc_timeout_ms = timeout
                .parse()
                .context("Invalid HAPROXY_AGENT_GRPC_RPC_TIMEOUT")?;
        }

        if let Ok(level) = std::env::var("HAPROXY_AGENT_LOG_LEVEL") {
            config.log_level = match level.to_lowercase().as_str() {
                "trace" => LogLevel::Trace,
                "debug" => LogLevel::Debug,
                "info" => LogLevel::Info,
                "warn" => LogLevel::Warn,
                "error" => LogLevel::Error,
                _ => anyhow::bail!("Invalid log level: {}", level),
            };
        }

        if let Ok(format) = std::env::var("HAPROXY_AGENT_LOG_FORMAT") {
            config.log_format = match format.to_lowercase().as_str() {
                "json" => LogFormat::Json,
                "pretty" => LogFormat::Pretty,
                _ => anyhow::bail!("Invalid log format: {}", format),
            };
        }

        if let Ok(cache) = std::env::var("HAPROXY_AGENT_GRPC_CHANNEL_CACHE") {
            config.grpc_channel_cache_enabled = match cache.to_lowercase().as_str() {
                "true" => true,
                "false" => false,
                _ => anyhow::bail!("Invalid HAPROXY_AGENT_GRPC_CHANNEL_CACHE value: {} (expected 'true' or 'false')", cache),
            };
        }

        Ok(config)
    }

    // T032: Load configuration from TOML file
    fn load_from_file(path: &PathBuf) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let config: AgentConfig = toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        Ok(config)
    }

    // T031: Apply CLI argument overrides
    fn apply_cli_overrides(mut config: AgentConfig, cli: CliArgs) -> Self {
        if let Some(port) = cli.server_port {
            config.server_port = port;
        }

        if let Some(bind) = cli.server_bind {
            config.server_bind_address = bind;
        }

        if let Some(port) = cli.metrics_port {
            config.metrics_port = port;
        }

        if let Some(bind) = cli.metrics_bind {
            config.metrics_bind_address = bind;
        }

        if let Some(timeout) = cli.grpc_connect_timeout {
            config.grpc_connect_timeout_ms = timeout;
        }

        if let Some(timeout) = cli.grpc_rpc_timeout {
            config.grpc_rpc_timeout_ms = timeout;
        }

        if let Some(level) = cli.log_level {
            config.log_level = level;
        }

        if let Some(format) = cli.log_format {
            config.log_format = format;
        }

        if let Some(cache) = cli.grpc_channel_cache {
            config.grpc_channel_cache_enabled = cache;
        }

        config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T046: Unit test for config validation with valid config
    #[test]
    fn test_config_validation_valid() {
        let config = AgentConfig {
            server_port: 5555,
            server_bind_address: "0.0.0.0".to_string(),
            grpc_connect_timeout_ms: 1000,
            grpc_rpc_timeout_ms: 1500,
            metrics_port: 9090,
            metrics_bind_address: "0.0.0.0".to_string(),
            grpc_channel_cache_enabled: true,
            log_level: LogLevel::Info,
            log_format: LogFormat::Json,
        };

        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_validation_valid_custom_ports() {
        let config = AgentConfig {
            server_port: 8080,
            server_bind_address: "127.0.0.1".to_string(),
            grpc_connect_timeout_ms: 500,
            grpc_rpc_timeout_ms: 1000,
            grpc_channel_cache_enabled: true,
            metrics_port: 9091,
            metrics_bind_address: "127.0.0.1".to_string(),
            log_level: LogLevel::Debug,
            log_format: LogFormat::Pretty,
        };

        let result = config.validate();
        assert!(result.is_ok());
    }

    // T047: Unit test for config validation with port conflict
    #[test]
    fn test_config_validation_port_conflict() {
        let config = AgentConfig {
            server_port: 9090,
            server_bind_address: "0.0.0.0".to_string(),
            grpc_connect_timeout_ms: 1000,
            grpc_rpc_timeout_ms: 1500,
            grpc_channel_cache_enabled: true,
            metrics_port: 9090, // Same as server_port!
            metrics_bind_address: "0.0.0.0".to_string(),
            log_level: LogLevel::Info,
            log_format: LogFormat::Json,
        };

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("cannot be the same"));
    }

    #[test]
    fn test_config_validation_invalid_server_port_zero() {
        let config = AgentConfig {
            server_port: 0,
            server_bind_address: "0.0.0.0".to_string(),
            grpc_connect_timeout_ms: 1000,
            grpc_rpc_timeout_ms: 1500,
            grpc_channel_cache_enabled: true,
            metrics_port: 9090,
            metrics_bind_address: "0.0.0.0".to_string(),
            log_level: LogLevel::Info,
            log_format: LogFormat::Json,
        };

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("server_port"));
    }

    #[test]
    fn test_config_validation_invalid_metrics_port_zero() {
        let config = AgentConfig {
            server_port: 5555,
            server_bind_address: "0.0.0.0".to_string(),
            grpc_connect_timeout_ms: 1000,
            grpc_rpc_timeout_ms: 1500,
            grpc_channel_cache_enabled: true,
            metrics_port: 0,
            metrics_bind_address: "0.0.0.0".to_string(),
            log_level: LogLevel::Info,
            log_format: LogFormat::Json,
        };

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("metrics_port"));
    }

    // T048: Unit test for config validation with invalid timeout
    #[test]
    fn test_config_validation_invalid_connect_timeout_zero() {
        let config = AgentConfig {
            server_port: 5555,
            server_bind_address: "0.0.0.0".to_string(),
            grpc_connect_timeout_ms: 0, // Invalid!
            grpc_rpc_timeout_ms: 1500,
            grpc_channel_cache_enabled: true,
            metrics_port: 9090,
            metrics_bind_address: "0.0.0.0".to_string(),
            log_level: LogLevel::Info,
            log_format: LogFormat::Json,
        };

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("grpc_connect_timeout_ms"));
    }

    #[test]
    fn test_config_validation_invalid_rpc_timeout_zero() {
        let config = AgentConfig {
            server_port: 5555,
            server_bind_address: "0.0.0.0".to_string(),
            grpc_connect_timeout_ms: 1000,
            grpc_rpc_timeout_ms: 0, // Invalid!
            grpc_channel_cache_enabled: true,
            metrics_port: 9090,
            metrics_bind_address: "0.0.0.0".to_string(),
            log_level: LogLevel::Info,
            log_format: LogFormat::Json,
        };

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("grpc_rpc_timeout_ms"));
    }

    #[test]
    fn test_config_default_values() {
        let config = AgentConfig::default();

        assert_eq!(config.server_port, 5555);
        assert_eq!(config.server_bind_address, "0.0.0.0");
        assert_eq!(config.grpc_connect_timeout_ms, 1000);
        assert_eq!(config.grpc_rpc_timeout_ms, 1500);
        assert_eq!(config.metrics_port, 9090);
        assert_eq!(config.metrics_bind_address, "0.0.0.0");
    }

    #[test]
    fn test_config_default_grpc_channel_cache_enabled() {
        let config = AgentConfig::default();
        assert!(
            config.grpc_channel_cache_enabled,
            "grpc_channel_cache_enabled should default to true"
        );
    }
}
