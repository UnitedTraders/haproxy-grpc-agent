# Research: Core HAProxy gRPC Agent

**Feature**: 001-core-agent
**Date**: 2025-10-28
**Status**: Complete

## Overview

This document consolidates research findings for implementing the HAProxy gRPC health check agent using Rust, tokio async runtime, and tonic gRPC client library.

---

## 1. Async Runtime: Tokio

**Decision**: Use tokio 1.x as the async runtime

**Rationale**:
- Industry standard for async Rust applications
- Excellent performance for I/O-bound workloads (TCP server + gRPC client)
- Built-in support for TCP listeners (`tokio::net::TcpListener`)
- Mature ecosystem with comprehensive documentation
- Powers tonic gRPC library (dependency alignment)
- Multi-threaded work-stealing scheduler for handling concurrent connections

**Alternatives Considered**:
- **async-std**: Similar API to tokio but smaller ecosystem and less tonic integration
- **smol**: Lightweight but lacks ecosystem maturity for production use
- **Blocking I/O**: Would not meet 1000+ concurrent connection requirement

**Best Practices**:
- Use `#[tokio::main]` for main entry point
- Leverage `tokio::spawn` for concurrent task handling (one task per HAProxy connection)
- Use `tokio::time::timeout` to enforce 2s health check timeout
- Enable `tokio` features: `full` (includes net, time, sync, macros)

---

## 2. gRPC Client: Tonic

**Decision**: Use tonic 0.10+ for gRPC health checks to backend servers

**Rationale**:
- Native Rust gRPC implementation built on tokio
- Automatic HTTP/2 connection pooling and multiplexing
- Support for both TLS and plain TCP connections (required per Agent Text Protocol)
- Code generation from `.proto` files (standardized health check protocol)
- Efficient connection reuse for repeated health checks to same backend

**Alternatives Considered**:
- **grpcio** (C bindings): Better performance but adds C dependency (violates statically-linked binary requirement)
- **tarpc**: RPC framework but not gRPC-compatible

**Best Practices**:
- Use gRPC Health Checking Protocol (standard `grpc.health.v1.Health/Check`)
- Implement connection pooling per backend (reuse channels for same `<host>:<port>`)
- Set connect timeout (1s) and RPC timeout (1.5s) to stay within 2s total check limit
- Handle TLS configuration based on `ssl_flag` from Agent Text Protocol

**Implementation Notes**:
```rust
// Example: Creating gRPC channel with TLS support
let channel = if ssl_flag == "ssl" {
    let tls = ClientTlsConfig::new()
        .domain_name(proxy_host_name);
    Channel::from_shared(endpoint)?
        .tls_config(tls)?
        .connect_timeout(Duration::from_secs(1))
        .connect()
        .await?
} else {
    Channel::from_shared(endpoint)?
        .connect_timeout(Duration::from_secs(1))
        .connect()
        .await?
};
```

---

## 3. Configuration Management

**Decision**: Use `clap` for CLI parsing, `config` crate for file loading, `serde` for serialization

**Rationale**:
- **clap 4.x**: Derive macros for type-safe CLI argument parsing, auto-generated help
- **config crate**: Layered configuration (env vars, files, defaults) with precedence
- **serde**: Standard serialization framework for Rust, works with TOML/YAML/JSON

**Configuration Precedence** (highest to lowest):
1. CLI flags (`--port 8080`)
2. Config file (`--config config.toml`)
3. Environment variables (`HAPROXY_AGENT_PORT=8080`)
4. Defaults (port 5555, log level INFO)

**Best Practices**:
- Validate configuration on startup (fail fast with clear errors)
- Log active configuration at INFO level for audit trail
- Support common formats: TOML (preferred), YAML, JSON
- Use environment variable prefix `HAPROXY_AGENT_` to avoid conflicts

**Configuration Schema**:
```toml
[server]
port = 5555              # Agent Text Protocol TCP port
bind_address = "0.0.0.0" # Listen address

[grpc]
connect_timeout_ms = 1000  # Backend gRPC connection timeout
rpc_timeout_ms = 1500      # Health check RPC timeout

[metrics]
port = 9090               # Prometheus metrics HTTP port
bind_address = "0.0.0.0"

[logging]
level = "info"            # trace, debug, info, warn, error
format = "json"           # json or pretty
```

---

## 4. Structured Logging

**Decision**: Use `tracing` + `tracing-subscriber` with JSON formatter

**Rationale**:
- `tracing`: Modern structured logging framework, async-aware
- Span-based tracing with automatic trace IDs
- JSON formatting via `tracing-subscriber` with `JsonFormat`
- Integration with tokio tasks (preserves context across async boundaries)
- Field-based structured logging (not string concatenation)

**Best Practices**:
- Use spans for request lifecycle tracking: `#[instrument]` macro
- Include fields: timestamp, level, component (module), message, trace_id, span_id
- Log at appropriate levels:
  - ERROR: Agent internal failures, backend unreachable
  - WARN: Protocol violations from HAProxy, timeout warnings
  - INFO: Startup config, connection events, health check results
  - DEBUG: Detailed protocol parsing, connection state
  - TRACE: Full request/response payloads

**Implementation Example**:
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(stream), fields(trace_id = %uuid::Uuid::new_v4()))]
async fn handle_connection(mut stream: TcpStream) {
    info!("HAProxy connection established");
    // ... handle request
    info!(status = "up", backend = %backend_addr, "Health check completed");
}
```

---

## 5. Prometheus Metrics

**Decision**: Use `prometheus` crate with `lazy_static` for metric registration

**Rationale**:
- Official Prometheus client library for Rust
- Type-safe metric definitions (Counter, Histogram, Gauge)
- Efficient in-memory metric aggregation
- HTTP endpoint via `hyper` (lightweight HTTP server)

**Metrics to Expose**:
```rust
// Counters
check_requests_total (labels: result=[up|down|error])
check_errors_total (labels: error_type=[timeout|parse_error|backend_unreachable])

// Histograms
check_duration_seconds (buckets: 0.01, 0.05, 0.1, 0.5, 1.0, 2.0)

// Gauges
haproxy_connections_active
grpc_channels_active (per unique backend)
```

**Best Practices**:
- Use histogram for latency (not summary - easier to aggregate)
- Label cardinality control: avoid high-cardinality labels (no IP addresses)
- Serve metrics on separate port (9090) from agent port (5555)
- Use `lazy_static!` or `once_cell` for global metric registration

---

## 6. HAProxy Agent Text Protocol Parsing

**Decision**: Custom parser using `nom` or manual `str::split_whitespace`

**Rationale**:
- Protocol is simple: `<server> <port> <ssl_flag> <proxy_host>\n`
- `nom`: Parser combinator library, safe and composable
- Manual parsing: Zero dependencies, sufficient for simple format

**Recommendation**: Manual parsing for simplicity (constitution principle V)

**Parser Requirements**:
- Split on whitespace into 4 fields
- Validate `ssl_flag` is either `ssl` or `no-ssl`
- Validate port is valid u16 (1-65535)
- Handle malformed input gracefully (log WARN, return `down\n`)

**Implementation Example**:
```rust
fn parse_agent_request(line: &str) -> Result<HealthCheckRequest, ParseError> {
    let parts: Vec<&str> = line.trim().split_whitespace().collect();
    if parts.len() != 4 {
        return Err(ParseError::InvalidFieldCount);
    }
    let backend_server = parts[0].to_string();
    let backend_port: u16 = parts[1].parse()
        .map_err(|_| ParseError::InvalidPort)?;
    let ssl_flag = match parts[2] {
        "ssl" => SslFlag::Ssl,
        "no-ssl" => SslFlag::NoSsl,
        _ => return Err(ParseError::InvalidSslFlag),
    };
    let proxy_host = parts[3].to_string();
    Ok(HealthCheckRequest { backend_server, backend_port, ssl_flag, proxy_host })
}
```

---

## 7. Connection Pooling Strategy

**Decision**: Implement per-backend gRPC channel cache with LRU eviction

**Rationale**:
- gRPC channels are expensive to create (TLS handshake, HTTP/2 setup)
- HAProxy checks same backends repeatedly
- Reusing channels improves latency and reduces resource usage

**Implementation**:
- Use `dashmap` (concurrent HashMap) for thread-safe channel cache
- Key: `(backend_server, backend_port, ssl_flag)` tuple
- Value: `tonic::transport::Channel`
- No explicit eviction initially (bounded by unique backend count)
- Future enhancement: LRU eviction if memory grows (not in MVP)

**Best Practices**:
- Clone channels (cheap Arc-based clone in tonic)
- Handle channel disconnection gracefully (recreate on next check)
- Log channel creation/reuse at DEBUG level

---

## 8. Error Handling Strategy

**Decision**: Fail-safe defaults per constitution principle V

**Error Mapping**:
- Backend unreachable → `down\n`
- gRPC RPC timeout → `down\n`
- Protocol parse error → `down\n` (log WARN)
- Internal agent error → `down\n` (log ERROR)
- Metrics/logging failure → continue health checks (log ERROR)

**Rationale**:
- Health check failures should not crash agent
- Conservative approach: report `down` on ambiguity
- Preserves HAProxy routing safety

---

## 9. Testing Strategy

**Decision**: Integration-first testing per constitution principle II

**Integration Tests**:
- Use `docker-compose` to spin up:
  - HAProxy 3.1+ with agent-check configuration
  - Mock gRPC backend (healthy/unhealthy states)
  - Agent binary under test
- Test scenarios:
  - HAProxy sends check, backend healthy → `up` response
  - HAProxy sends check, backend down → `down` response
  - HAProxy sends check, TLS backend → correct SSL handling
  - HAProxy disconnects abruptly → agent continues serving
  - Multiple rapid checks → all responses correct

**Unit Tests** (minimal):
- `protocol.rs`: Parse valid/invalid Agent Text Protocol requests
- `config.rs`: Configuration precedence and validation

**Test Execution**:
- `make test-integration`: Runs docker-compose + cargo test
- CI: GitHub Actions with Docker-in-Docker

---

## 10. Deployment & Binary Size Optimization

**Decision**: Multi-stage Docker build with musl static linking

**Rationale**:
- musl libc produces fully static binaries (no glibc dependency)
- Binary can run in `scratch` or `distroless` container
- Meets <20MB binary size constraint

**Build Configuration**:
```dockerfile
FROM rust:1.75-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /build
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl

FROM scratch
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/haproxy-grpc-agent /
ENTRYPOINT ["/haproxy-grpc-agent"]
```

**Cargo.toml Optimizations**:
```toml
[profile.release]
lto = true           # Link-time optimization
codegen-units = 1    # Better optimization, slower build
strip = true         # Strip symbols
opt-level = "z"      # Optimize for size
```

**Expected Binary Size**: 10-15MB (well within 20MB constraint)

---

## Summary

All technical decisions support constitution principles:
- **Agent Pattern**: Single static binary, minimal dependencies, simple deployment
- **Integration-Heavy Testing**: Docker-compose integration tests, real HAProxy
- **Observability**: Structured JSON logs (tracing), Prometheus metrics
- **Protocol Compliance**: Strict Agent Text Protocol parsing, fail-safe defaults
- **Simplicity & Reliability**: Manual parsing over complex libs, connection pooling only where needed, graceful error handling

No NEEDS CLARIFICATION items remain. Proceed to Phase 1 design.
