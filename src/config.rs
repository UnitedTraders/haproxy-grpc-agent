// Configuration module for HAProxy gRPC Agent
// T026-T034: Complete configuration implementation

use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
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

// T002: LogDestination enum for configurable log output target
#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum LogDestination {
    #[default]
    Console,
    File,
}

// T003: LoggingConfig struct for the [logging] TOML section
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct LoggingConfig {
    #[serde(default)]
    pub destination: LogDestination,

    #[serde(default)]
    pub level: Option<LogLevel>,

    #[serde(default)]
    pub format: Option<LogFormat>,

    #[serde(default)]
    pub file_path: Option<String>,

    /// Log file rotation strategy: "never", "daily", "hourly"
    #[serde(default)]
    pub file_rotation: Option<LogRotation>,

    /// Maximum number of rotated log files to keep
    #[serde(default)]
    pub file_max_files: Option<usize>,

    #[serde(default)]
    pub packages: HashMap<String, LogLevel>,
}

// Log file rotation strategy
#[derive(Debug, Clone, Copy, Deserialize, Serialize, clap::ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum LogRotation {
    Never,
    Daily,
    Hourly,
}

impl LogLevel {
    /// Returns the string representation of the log level for tracing directives
    pub fn as_str(&self) -> &'static str {
        match self {
            LogLevel::Trace => "trace",
            LogLevel::Debug => "debug",
            LogLevel::Info => "info",
            LogLevel::Warn => "warn",
            LogLevel::Error => "error",
        }
    }
}

impl LoggingConfig {
    /// Returns the effective log level, preferring [logging].level over top-level log_level
    pub fn resolved_level(&self, top_level: &LogLevel) -> LogLevel {
        self.level.unwrap_or(*top_level)
    }

    /// Returns the effective log format, preferring [logging].format over top-level log_format
    pub fn resolved_format(&self, top_level: &LogFormat) -> LogFormat {
        self.format.unwrap_or(*top_level)
    }

    /// Builds an EnvFilter directive string from the resolved level and per-package overrides.
    /// Example output: "info,haproxy_grpc_agent::checker=debug,tonic=warn"
    pub fn build_env_filter_directive(&self, top_level: &LogLevel) -> String {
        let base_level = self.resolved_level(top_level);
        let mut directive = base_level.as_str().to_string();

        for (package, level) in &self.packages {
            directive.push(',');
            directive.push_str(package);
            directive.push('=');
            directive.push_str(level.as_str());
        }

        directive
    }
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

    #[serde(default)]
    pub logging: LoggingConfig,
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
            logging: LoggingConfig::default(),
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

    /// Log destination (console or file)
    #[arg(long, value_enum)]
    pub log_destination: Option<LogDestination>,

    /// Log file path (required when --log-destination=file)
    #[arg(long)]
    pub log_file_path: Option<String>,

    /// Log file rotation strategy (never, daily, hourly)
    #[arg(long, value_enum)]
    pub log_file_rotation: Option<LogRotation>,

    /// Maximum number of rotated log files to keep
    #[arg(long)]
    pub log_file_max_files: Option<usize>,
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

        // Validate logging config
        if matches!(self.logging.destination, LogDestination::File) {
            match &self.logging.file_path {
                None => anyhow::bail!(
                    "logging.file_path is required when logging.destination is \"file\""
                ),
                Some(path) if path.is_empty() => anyhow::bail!(
                    "logging.file_path must not be empty when logging.destination is \"file\""
                ),
                _ => {}
            }
        }

        if let Some(max_files) = self.logging.file_max_files
            && max_files == 0
        {
            anyhow::bail!("logging.file_max_files must be greater than 0");
        }

        // Warn if file_max_files is set without rotation
        if self.logging.file_max_files.is_some() && self.logging.file_rotation.is_none() {
            eprintln!(
                "WARNING: logging.file_max_files is set but logging.file_rotation is not. \
                 file_max_files has no effect without rotation enabled."
            );
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
                _ => anyhow::bail!(
                    "Invalid HAPROXY_AGENT_GRPC_CHANNEL_CACHE value: {} (expected 'true' or 'false')",
                    cache
                ),
            };
        }

        if let Ok(dest) = std::env::var("HAPROXY_AGENT_LOG_DESTINATION") {
            config.logging.destination = match dest.to_lowercase().as_str() {
                "console" => LogDestination::Console,
                "file" => LogDestination::File,
                _ => anyhow::bail!("Invalid HAPROXY_AGENT_LOG_DESTINATION: {} (expected 'console' or 'file')", dest),
            };
        }

        if let Ok(path) = std::env::var("HAPROXY_AGENT_LOG_FILE_PATH") {
            config.logging.file_path = Some(path);
        }

        if let Ok(rotation) = std::env::var("HAPROXY_AGENT_LOG_FILE_ROTATION") {
            config.logging.file_rotation = Some(match rotation.to_lowercase().as_str() {
                "never" => LogRotation::Never,
                "daily" => LogRotation::Daily,
                "hourly" => LogRotation::Hourly,
                _ => anyhow::bail!(
                    "Invalid HAPROXY_AGENT_LOG_FILE_ROTATION: {} (expected 'never', 'daily', or 'hourly')",
                    rotation
                ),
            });
        }

        if let Ok(max_files) = std::env::var("HAPROXY_AGENT_LOG_FILE_MAX_FILES") {
            config.logging.file_max_files = Some(
                max_files.parse().context("Invalid HAPROXY_AGENT_LOG_FILE_MAX_FILES")?,
            );
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

        if let Some(dest) = cli.log_destination {
            config.logging.destination = dest;
        }

        if let Some(path) = cli.log_file_path {
            config.logging.file_path = Some(path);
        }

        if let Some(rotation) = cli.log_file_rotation {
            config.logging.file_rotation = Some(rotation);
        }

        if let Some(max_files) = cli.log_file_max_files {
            config.logging.file_max_files = Some(max_files);
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
            logging: LoggingConfig::default(),
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
            logging: LoggingConfig::default(),
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
            logging: LoggingConfig::default(),
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
            logging: LoggingConfig::default(),
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
            logging: LoggingConfig::default(),
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
            logging: LoggingConfig::default(),
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
            logging: LoggingConfig::default(),
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

    // T008: LoggingConfig default and validation tests
    #[test]
    fn test_logging_config_defaults() {
        let logging = LoggingConfig::default();
        assert!(matches!(logging.destination, LogDestination::Console));
        assert!(logging.level.is_none());
        assert!(logging.format.is_none());
        assert!(logging.file_path.is_none());
        assert!(logging.file_rotation.is_none());
        assert!(logging.file_max_files.is_none());
        assert!(logging.packages.is_empty());
    }

    #[test]
    fn test_logging_config_in_agent_config_default() {
        let config = AgentConfig::default();
        assert!(matches!(config.logging.destination, LogDestination::Console));
        assert!(config.logging.packages.is_empty());
    }

    #[test]
    fn test_logging_validation_file_without_path() {
        let mut config = AgentConfig::default();
        config.logging.destination = LogDestination::File;
        config.logging.file_path = None;

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("file_path"));
    }

    #[test]
    fn test_logging_validation_file_with_empty_path() {
        let mut config = AgentConfig::default();
        config.logging.destination = LogDestination::File;
        config.logging.file_path = Some("".to_string());

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("file_path"));
    }

    #[test]
    fn test_logging_validation_file_with_valid_path() {
        let mut config = AgentConfig::default();
        config.logging.destination = LogDestination::File;
        config.logging.file_path = Some("/tmp/test.log".to_string());

        let result = config.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_logging_validation_max_size_zero() {
        let mut config = AgentConfig::default();
        config.logging.file_max_files = Some(0);

        let result = config.validate();
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("file_max_files"));
    }

    #[test]
    fn test_logging_validation_console_ignores_file_settings() {
        let mut config = AgentConfig::default();
        config.logging.destination = LogDestination::Console;
        // These should be ignored for console destination
        config.logging.file_rotation = Some(LogRotation::Daily);
        config.logging.file_max_files = Some(5);

        let result = config.validate();
        assert!(result.is_ok());
    }

    // T012: Tests for resolved_level
    #[test]
    fn test_resolved_level_logging_level_set() {
        let mut logging = LoggingConfig::default();
        logging.level = Some(LogLevel::Debug);
        let top_level = LogLevel::Info;

        assert!(matches!(logging.resolved_level(&top_level), LogLevel::Debug));
    }

    #[test]
    fn test_resolved_level_logging_level_unset_falls_back() {
        let logging = LoggingConfig::default();
        let top_level = LogLevel::Warn;

        assert!(matches!(logging.resolved_level(&top_level), LogLevel::Warn));
    }

    #[test]
    fn test_resolved_level_both_set_logging_wins() {
        let mut logging = LoggingConfig::default();
        logging.level = Some(LogLevel::Error);
        let top_level = LogLevel::Trace;

        assert!(matches!(
            logging.resolved_level(&top_level),
            LogLevel::Error
        ));
    }

    // T013: Tests for build_env_filter_directive
    #[test]
    fn test_env_filter_directive_no_overrides() {
        let logging = LoggingConfig::default();
        let directive = logging.build_env_filter_directive(&LogLevel::Info);
        assert_eq!(directive, "info");
    }

    #[test]
    fn test_env_filter_directive_single_override() {
        let mut logging = LoggingConfig::default();
        logging
            .packages
            .insert("haproxy_grpc_agent::checker".to_string(), LogLevel::Debug);
        let directive = logging.build_env_filter_directive(&LogLevel::Info);
        assert!(directive.starts_with("info,"));
        assert!(directive.contains("haproxy_grpc_agent::checker=debug"));
    }

    #[test]
    fn test_env_filter_directive_multiple_overrides() {
        let mut logging = LoggingConfig::default();
        logging
            .packages
            .insert("haproxy_grpc_agent::checker".to_string(), LogLevel::Debug);
        logging
            .packages
            .insert("tonic".to_string(), LogLevel::Warn);
        let directive = logging.build_env_filter_directive(&LogLevel::Info);
        assert!(directive.starts_with("info,"));
        assert!(directive.contains("haproxy_grpc_agent::checker=debug"));
        assert!(directive.contains("tonic=warn"));
    }

    #[test]
    fn test_env_filter_directive_with_logging_level() {
        let mut logging = LoggingConfig::default();
        logging.level = Some(LogLevel::Warn);
        let directive = logging.build_env_filter_directive(&LogLevel::Info);
        assert_eq!(directive, "warn");
    }
}
