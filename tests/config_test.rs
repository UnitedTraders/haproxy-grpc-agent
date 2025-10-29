// Unit tests for configuration
// T046-T048: Config validation tests

use haproxy_grpc_agent::config::{AgentConfig, LogFormat, LogLevel};

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
