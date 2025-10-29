# Data Model: Core HAProxy gRPC Agent

**Feature**: 001-core-agent
**Date**: 2025-10-28
**Status**: Complete

## Overview

This document defines the core entities, their fields, relationships, and validation rules for the HAProxy gRPC health check agent.

---

## 1. HealthCheckRequest

**Description**: Parsed representation of an Agent Text Protocol request from HAProxy

**Fields**:
- `backend_server`: String - Backend server name or IP address to check
- `backend_port`: u16 - Backend server port to check
- `ssl_flag`: SslFlag - Whether to use TLS for gRPC connection
- `proxy_host_name`: String - Value to use in Host/authority header for gRPC request

**Validation Rules**:
- `backend_server`: MUST NOT be empty
- `backend_port`: MUST be in range 1-65535
- `ssl_flag`: MUST be either `SslFlag::Ssl` or `SslFlag::NoSsl`
- `proxy_host_name`: MUST NOT be empty

**State Transitions**: None (immutable value object)

**Rust Implementation**:
```rust
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRequest {
    pub backend_server: String,
    pub backend_port: u16,
    pub ssl_flag: SslFlag,
    pub proxy_host_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SslFlag {
    Ssl,
    NoSsl,
}
```

**Relationships**:
- Produced by: `protocol::parse_request()`
- Consumed by: `checker::check_backend()`

---

## 2. HealthCheckResponse

**Description**: Response sent back to HAProxy via Agent Text Protocol

**Fields**:
- `status`: HealthStatus - The health check result

**Values**:
- `HealthStatus::Up` → serializes to `"up\n"`
- `HealthStatus::Down` → serializes to `"down\n"`
- `HealthStatus::Maint` → serializes to `"maint\n"` (future enhancement, not in MVP)

**Validation Rules**:
- `status`: MUST be one of the defined enum variants

**State Transitions**: None (immutable value object)

**Rust Implementation**:
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HealthStatus {
    Up,
    Down,
    Maint,  // Reserved for future use
}

impl HealthStatus {
    pub fn to_protocol_string(&self) -> String {
        match self {
            HealthStatus::Up => "up\n".to_string(),
            HealthStatus::Down => "down\n".to_string(),
            HealthStatus::Maint => "maint\n".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthCheckResponse {
    pub status: HealthStatus,
}
```

**Relationships**:
- Produced by: `checker::check_backend()`
- Consumed by: `server::handle_connection()` → written to TCP stream

---

## 3. AgentConfig

**Description**: Configuration for the HAProxy gRPC agent, loaded from env vars, CLI flags, or config file

**Fields**:

**Server Config**:
- `server_port`: u16 - TCP port for Agent Text Protocol server (default: 5555)
- `server_bind_address`: String - Bind address for agent server (default: "0.0.0.0")

**gRPC Config**:
- `grpc_connect_timeout_ms`: u64 - Timeout for establishing gRPC connection to backend (default: 1000)
- `grpc_rpc_timeout_ms`: u64 - Timeout for gRPC health check RPC (default: 1500)

**Metrics Config**:
- `metrics_port`: u16 - HTTP port for Prometheus metrics endpoint (default: 9090)
- `metrics_bind_address`: String - Bind address for metrics server (default: "0.0.0.0")

**Logging Config**:
- `log_level`: LogLevel - Logging verbosity (default: Info)
- `log_format`: LogFormat - Log output format (default: Json)

**Validation Rules**:
- `server_port`: MUST be 1-65535
- `metrics_port`: MUST be 1-65535, MUST NOT equal `server_port`
- `grpc_connect_timeout_ms`: MUST be > 0, SHOULD be < 2000 (leave room for RPC)
- `grpc_rpc_timeout_ms`: MUST be > 0
- `grpc_connect_timeout_ms + grpc_rpc_timeout_ms`: SHOULD be < 2000 (HAProxy default timeout)

**State Transitions**: None (loaded once at startup, immutable)

**Rust Implementation**:
```rust
use serde::{Deserialize, Serialize};

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

    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    #[serde(default = "default_bind_address")]
    pub metrics_bind_address: String,

    #[serde(default)]
    pub log_level: LogLevel,

    #[serde(default)]
    pub log_format: LogFormat,
}

fn default_server_port() -> u16 { 5555 }
fn default_metrics_port() -> u16 { 9090 }
fn default_bind_address() -> String { "0.0.0.0".to_string() }
fn default_grpc_connect_timeout() -> u64 { 1000 }
fn default_grpc_rpc_timeout() -> u64 { 1500 }

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl Default for LogLevel {
    fn default() -> Self { LogLevel::Info }
}

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}

impl Default for LogFormat {
    fn default() -> Self { LogFormat::Json }
}
```

**Relationships**:
- Loaded by: `config::load_config()`
- Used by: All modules (server, checker, metrics, logger)

---

## 4. BackendChannelKey

**Description**: Key for caching gRPC channels to backend servers

**Fields**:
- `server`: String - Backend server name/IP
- `port`: u16 - Backend server port
- `ssl_flag`: SslFlag - TLS enabled or not

**Rationale**: Channels can be reused for the same backend endpoint and SSL configuration

**Validation Rules**:
- Same as `HealthCheckRequest` fields

**State Transitions**: None (immutable hash key)

**Rust Implementation**:
```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BackendChannelKey {
    pub server: String,
    pub port: u16,
    pub ssl_flag: SslFlag,
}

impl From<&HealthCheckRequest> for BackendChannelKey {
    fn from(req: &HealthCheckRequest) -> Self {
        BackendChannelKey {
            server: req.backend_server.clone(),
            port: req.backend_port,
            ssl_flag: req.ssl_flag,
        }
    }
}
```

**Relationships**:
- Used by: `checker::ChannelCache` (DashMap<BackendChannelKey, Channel>)
- Derived from: `HealthCheckRequest`

---

## 5. Metrics

**Description**: Prometheus metrics tracked by the agent

**Counters**:
```rust
pub static CHECK_REQUESTS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    CounterVec::new(
        Opts::new("check_requests_total", "Total health check requests"),
        &["result"]  // Labels: up, down, error
    ).unwrap()
});

pub static CHECK_ERRORS_TOTAL: Lazy<CounterVec> = Lazy::new(|| {
    CounterVec::new(
        Opts::new("check_errors_total", "Total health check errors"),
        &["error_type"]  // Labels: timeout, parse_error, backend_unreachable, internal
    ).unwrap()
});
```

**Histograms**:
```rust
pub static CHECK_DURATION_SECONDS: Lazy<Histogram> = Lazy::new(|| {
    Histogram::with_opts(
        HistogramOpts::new("check_duration_seconds", "Health check duration")
            .buckets(vec![0.01, 0.05, 0.1, 0.5, 1.0, 2.0])
    ).unwrap()
});
```

**Gauges**:
```rust
pub static HAPROXY_CONNECTIONS_ACTIVE: Lazy<Gauge> = Lazy::new(|| {
    Gauge::new("haproxy_connections_active", "Active HAProxy connections").unwrap()
});

pub static GRPC_CHANNELS_ACTIVE: Lazy<Gauge> = Lazy::new(|| {
    Gauge::new("grpc_channels_active", "Active gRPC channels to backends").unwrap()
});
```

**Validation Rules**: N/A (metrics library handles validation)

**State Transitions**: Metrics are incremented/observed during operations

**Relationships**:
- Updated by: `server`, `checker` modules
- Exposed by: `metrics::serve_metrics()` HTTP endpoint

---

## 6. ConnectionState

**Description**: State machine for handling a single HAProxy connection

**States**:
- `Listening` - Server accepting connections
- `Connected` - HAProxy connection established
- `ReadingRequest` - Reading Agent Text Protocol request from TCP stream
- `ProcessingCheck` - Performing gRPC health check to backend
- `WritingResponse` - Writing response back to HAProxy
- `Closed` - Connection closed

**State Transitions**:
```
Listening → Connected (on accept)
Connected → ReadingRequest (start reading)
ReadingRequest → ProcessingCheck (request parsed)
ProcessingCheck → WritingResponse (check complete)
WritingResponse → ReadingRequest (response sent, keep-alive)
WritingResponse → Closed (connection closed by HAProxy)
ReadingRequest → Closed (parse error or EOF)
```

**Validation Rules**: Transitions must follow defined flow

**Implementation Note**: Not explicitly modeled as enum; implicit in async handler flow

**Relationships**:
- Managed by: `server::handle_connection()`
- Logs state transitions for observability

---

## Entity Relationship Diagram

```
┌─────────────────────────────┐
│      AgentConfig            │
│  (loaded at startup)        │
└──────────┬──────────────────┘
           │ used by
           ↓
┌─────────────────────────────┐
│      TCP Server             │
│  (Agent Text Protocol)      │
└──────────┬──────────────────┘
           │ receives
           ↓
┌─────────────────────────────┐
│   HealthCheckRequest        │
│  (parsed from text)         │
└──────────┬──────────────────┘
           │ processed by
           ↓
┌─────────────────────────────┐      ┌─────────────────────────┐
│   gRPC Health Checker       │─────→│  BackendChannelKey      │
│  (connects to backend)      │      │  (channel cache key)    │
└──────────┬──────────────────┘      └─────────────────────────┘
           │ produces
           ↓
┌─────────────────────────────┐
│   HealthCheckResponse       │
│  (up/down status)           │
└──────────┬──────────────────┘
           │ sent to
           ↓
      HAProxy (TCP stream)

     Metrics (updated throughout)
```

---

## Validation Summary

All entities follow validation rules:
- **Input validation**: Agent Text Protocol parser validates all fields
- **Configuration validation**: Config loader validates on startup, fails fast
- **Type safety**: Rust type system enforces invariants (u16 for ports, enums for states)
- **Error handling**: Invalid states map to `HealthStatus::Down` (fail-safe)

---

## Implementation Notes

1. **Immutability**: Request/Response objects are immutable value objects
2. **Thread Safety**: Channel cache uses `DashMap` for concurrent access
3. **Resource Management**: Channels are Arc-based, cheaply cloneable
4. **Memory Bounds**: No explicit eviction in MVP (bounded by unique backend count)
5. **Error Propagation**: Use `Result<T, E>` throughout, map errors to `Down` at protocol boundary

Phase 1 data model complete. Proceed to contracts generation.
