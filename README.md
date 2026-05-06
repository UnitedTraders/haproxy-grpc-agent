# HAProxy gRPC Agent

[![CI/CD](https://github.com/unitedtraders/haproxy-grpc-agent/actions/workflows/ci.yml/badge.svg)](https://github.com/unitedtraders/haproxy-grpc-agent/actions/workflows/ci.yml)

A HAProxy agent-check backend that performs gRPC health checks using the standard `grpc.health.v1.Health` service.

## Overview

This project implements a [HAProxy agent-check](https://www.haproxy.com/documentation/haproxy-configuration-tutorials/reliability/health-checks/#agent-checks) server that checks gRPC backend health using the standard `grpc.health.v1.Health/Check` RPC method.

### Motivation

Standard HTTP checks (`option httpchk`) with Netty-based gRPC applications often lead to exceptions like:
```
INTERNAL: Encountered end-of-stream mid-frame
```

This agent solves this by using proper gRPC health check protocol instead of HTTP/1.1 checks.

## Features

- ✅ gRPC health checks via `grpc.health.v1.Health/Check`
- ✅ TLS/SSL support for secure backends
- ✅ Connection pooling and caching
- ✅ Structured JSON logging (console or file)
- ✅ Per-package log level overrides
- ✅ Log file rotation (daily, hourly)
- ✅ Prometheus metrics export
- ✅ Configurable timeouts
- ✅ Docker support
- ✅ Low resource footprint (<50MB memory, <20MB binary)

## Quick Start

### Using Docker (Recommended)

Pull from GitHub Container Registry:

```bash
docker pull ghcr.io/unitedtraders/haproxy-grpc-agent:latest
```

Run the agent:

```bash
docker run -d \
  --name haproxy-grpc-agent \
  -p 5555:5555 \
  -p 9090:9090 \
  -e HAPROXY_AGENT_LOG_LEVEL=info \
  ghcr.io/unitedtraders/haproxy-grpc-agent:latest
```

### Using Pre-built Binaries

Download from the [releases page](https://github.com/unitedtraders/haproxy-grpc-agent/releases):

```bash
# Linux AMD64
curl -L https://github.com/unitedtraders/haproxy-grpc-agent/releases/latest/download/haproxy-grpc-agent-linux-amd64.tar.gz | tar xz

# Linux ARM64
curl -L https://github.com/unitedtraders/haproxy-grpc-agent/releases/latest/download/haproxy-grpc-agent-linux-arm64.tar.gz | tar xz

# Run
./haproxy-grpc-agent
```

### Building from Source

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/unitedtraders/haproxy-grpc-agent.git
cd haproxy-grpc-agent
cargo build --release

# Run
./target/release/haproxy-grpc-agent
```

## Configuration

The agent can be configured via environment variables, CLI flags, or a TOML configuration file.

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `HAPROXY_AGENT_SERVER_PORT` | `5555` | Agent TCP server port |
| `HAPROXY_AGENT_SERVER_BIND` | `0.0.0.0` | Server bind address |
| `HAPROXY_AGENT_METRICS_PORT` | `9090` | Prometheus metrics port |
| `HAPROXY_AGENT_METRICS_BIND` | `0.0.0.0` | Metrics server bind address |
| `HAPROXY_AGENT_LOG_LEVEL` | `info` | Log level (trace, debug, info, warn, error) |
| `HAPROXY_AGENT_LOG_FORMAT` | `json` | Log format (json, pretty) |
| `HAPROXY_AGENT_LOG_DESTINATION` | `console` | Log destination (console, file) |
| `HAPROXY_AGENT_LOG_FILE_PATH` | — | Log file path (required when destination=file) |
| `HAPROXY_AGENT_LOG_FILE_ROTATION` | — | File rotation strategy (never, daily, hourly) |
| `HAPROXY_AGENT_LOG_FILE_MAX_FILES` | — | Max rotated log files to keep |
| `HAPROXY_AGENT_GRPC_CONNECT_TIMEOUT` | `1000` | gRPC connection timeout (ms) |
| `HAPROXY_AGENT_GRPC_RPC_TIMEOUT` | `1500` | gRPC RPC timeout (ms) |
| `HAPROXY_AGENT_GRPC_CHANNEL_CACHE` | `true` | Enable gRPC channel caching (true, false) |

### CLI Flags

```bash
haproxy-grpc-agent --help

Options:
  -c, --config <FILE>               Path to config file
  --server-port <PORT>              Agent server port
  --server-bind <ADDRESS>           Server bind address
  --metrics-port <PORT>             Metrics server port
  --metrics-bind <ADDRESS>          Metrics bind address
  --log-level <LEVEL>               Log level (trace, debug, info, warn, error)
  --log-format <FORMAT>             Log format (json, pretty)
  --log-destination <DEST>          Log destination (console, file)
  --log-file-path <PATH>            Log file path (required when --log-destination=file)
  --log-file-rotation <STRATEGY>    File rotation (never, daily, hourly)
  --log-file-max-files <N>          Max rotated log files to keep
  --grpc-connect-timeout <MS>       gRPC connect timeout
  --grpc-rpc-timeout <MS>           gRPC RPC timeout
  --grpc-channel-cache [true|false] Enable gRPC channel caching
```

### TOML Configuration File

Create a `config.toml`:

```toml
server_bind_address = "0.0.0.0"
server_port = 5555
metrics_port = 9090
log_level = "info"
log_format = "json"
grpc_connect_timeout_ms = 1000
grpc_rpc_timeout_ms = 1500

[logging]
destination = "console"       # "console" or "file"
# level = "debug"             # overrides top-level log_level
# format = "pretty"           # overrides top-level log_format

# File destination settings (used when destination = "file"):
# file_path = "/var/log/haproxy-agent/agent.log"
# file_rotation = "daily"     # "never", "daily", "hourly"
# file_max_files = 7          # max rotated files to keep

# Per-package log level overrides:
# [logging.packages]
# haproxy_grpc_agent = "debug"
# haproxy_grpc_agent::server = "trace"
# tonic = "warn"
```

Run with config file:

```bash
haproxy-grpc-agent --config config.toml
```

### Logging Configuration

The agent supports two log destinations: **console** (stderr, default) and **file**.

**Write logs to a file with daily rotation:**

```toml
[logging]
destination = "file"
file_path = "/var/log/haproxy-agent/agent.log"
file_rotation = "daily"
file_max_files = 7
```

**Per-package log level overrides** allow fine-grained control:

```toml
log_level = "warn"

[logging]
level = "info"

[logging.packages]
haproxy_grpc_agent = "debug"
tonic = "warn"
```

The `[logging].level` overrides the top-level `log_level`. Individual packages in `[logging.packages]` override both. Setting `RUST_LOG` environment variable overrides everything.

**Configuration precedence:** CLI flags > environment variables > config file `[logging]` section > config file top-level > defaults.

## Usage

### Protocol

The agent listens on a TCP port (default 5555) for health check requests in the format:

```
<backend_server> <backend_port> <ssl_flag> <proxy_host_name>\n
```

**Example:**
```
myservice.default.svc.cluster.local 50051 no-ssl myservice.default.svc.cluster.local\n
```

**Response:**
- `up\n` - Backend is healthy (gRPC status: SERVING)
- `down\n` - Backend is unhealthy or unreachable

### HAProxy Configuration

Configure HAProxy backend with agent-check:

```haproxy
backend grpc_backend
    mode tcp
    balance roundrobin

    # Agent check configuration
    server grpc1 myservice.example.com:50051 \
        check \
        agent-check \
        agent-inter 5s \
        agent-port 5555 \
        agent-addr haproxy-grpc-agent.default.svc.cluster.local \
        agent-send "myservice.example.com 50051 no-ssl myservice.example.com\n"

    server grpc2 myservice2.example.com:50051 \
        check \
        agent-check \
        agent-inter 5s \
        agent-port 5555 \
        agent-addr haproxy-grpc-agent.default.svc.cluster.local \
        agent-send "myservice2.example.com 50051 ssl myservice2.example.com\n"
```

### SSL/TLS Backends

For TLS-enabled backends, use `ssl` instead of `no-ssl`:

```
myservice.example.com 50051 ssl myservice.example.com\n
```

## Monitoring

### Prometheus Metrics

Metrics are exposed on `/metrics` endpoint (default port 9090):

```bash
curl http://localhost:9090/metrics
```

**Available Metrics:**

- `check_requests_total{result}` - Total health check requests
- `check_errors_total{error_type}` - Total health check errors
- `check_duration_seconds` - Health check duration histogram
- `haproxy_connections_active` - Active HAProxy connections
- `grpc_channels_active` - Active gRPC channels

### Structured Logging

JSON logs include:

- `timestamp` - ISO8601 timestamp
- `level` - Log level (INFO, WARN, ERROR)
- `target` - Log source module
- `fields` - Structured fields
  - `message` - Log message
  - `trace_id` - Request trace ID
  - `backend` - Backend address
  - `error` - Error details (if any)

**Example log:**
```json
{
  "timestamp": "2025-10-29T17:48:25.770595Z",
  "level": "INFO",
  "fields": {
    "message": "Health check completed",
    "backend": "myservice.example.com:50051",
    "status": "Up",
    "trace_id": "a1b2c3d4-e5f6-7890-abcd-ef1234567890"
  },
  "target": "haproxy_grpc_agent::server"
}
```

## Docker Deployment

### Docker Compose Example

```yaml
version: '3.8'

services:
  haproxy-grpc-agent:
    image: ghcr.io/unitedtraders/haproxy-grpc-agent:latest
    ports:
      - "5555:5555"
      - "9090:9090"
    environment:
      - HAPROXY_AGENT_LOG_LEVEL=info
      - HAPROXY_AGENT_LOG_FORMAT=json
      - HAPROXY_AGENT_GRPC_CONNECT_TIMEOUT=1000
      - HAPROXY_AGENT_GRPC_RPC_TIMEOUT=1500
    restart: unless-stopped
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: haproxy-grpc-agent
spec:
  replicas: 2
  selector:
    matchLabels:
      app: haproxy-grpc-agent
  template:
    metadata:
      labels:
        app: haproxy-grpc-agent
    spec:
      containers:
      - name: agent
        image: ghcr.io/unitedtraders/haproxy-grpc-agent:latest
        ports:
        - containerPort: 5555
          name: agent
        - containerPort: 9090
          name: metrics
        env:
        - name: HAPROXY_AGENT_LOG_LEVEL
          value: "info"
        resources:
          requests:
            memory: "32Mi"
            cpu: "50m"
          limits:
            memory: "128Mi"
            cpu: "200m"
---
apiVersion: v1
kind: Service
metadata:
  name: haproxy-grpc-agent
spec:
  selector:
    app: haproxy-grpc-agent
  ports:
  - port: 5555
    name: agent
  - port: 9090
    name: metrics
```

## Development

### Running Tests

```bash
# Run all tests
cargo test

# Run unit tests only
cargo test --lib --bins

# Run integration tests (requires Docker)
cargo test --test integration_test -- --ignored

# Run logging configuration tests
cargo test --test config_logging_test -- --include-ignored --test-threads=1
```

### Code Quality

```bash
# Format code
cargo fmt

# Run clippy
cargo clippy -- -D warnings

# Run all checks
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

### Building Docker Image Locally

```bash
docker build -t haproxy-grpc-agent:dev .
```

## CI/CD

The project uses GitHub Actions for continuous integration and deployment:

### Automated Workflows

1. **Test on Every Commit** - Runs tests, formatting, and clippy checks on all branches
2. **Docker Build** - Builds and publishes multi-arch Docker images to ghcr.io on `main` branch
3. **Release Binaries** - Creates release artifacts for Linux (amd64, arm64) and macOS when tags are pushed

### Triggering a Release

```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions will automatically:
# - Build binaries for all platforms
# - Create a GitHub release
# - Attach binary artifacts
# - Build and push Docker images
```

## Architecture

```
┌─────────────┐         ┌──────────────────┐         ┌─────────────┐
│   HAProxy   │ ──TCP──>│ haproxy-grpc-    │ ─gRPC──>│   Backend   │
│             │ <──────>│      agent       │ <──────>│   Service   │
└─────────────┘         └──────────────────┘         └─────────────┘
                               │
                               │ HTTP
                               ▼
                        ┌──────────────┐
                        │  Prometheus  │
                        │   /metrics   │
                        └──────────────┘
```

## Performance

- **Binary Size**: <20MB (release build)
- **Memory Usage**: <50MB under 100 req/s load
- **Startup Time**: <5 seconds
- **Response Time**: <2 seconds per health check

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch
3. Make your changes with tests
4. Run `cargo fmt && cargo clippy && cargo test`
5. Submit a pull request

## License

[Add your license here]

## Support

- **Issues**: https://github.com/unitedtraders/haproxy-grpc-agent/issues
- **Documentation**: See `docs/` directory
- **Discussions**: https://github.com/unitedtraders/haproxy-grpc-agent/discussions
