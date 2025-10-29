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
- ✅ Structured JSON logging
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
  -e AGENT_LOG_LEVEL=info \
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
| `AGENT_SERVER_BIND_ADDRESS` | `0.0.0.0` | Server bind address |
| `AGENT_SERVER_PORT` | `5555` | Agent TCP server port |
| `AGENT_METRICS_PORT` | `9090` | Prometheus metrics port |
| `AGENT_LOG_LEVEL` | `info` | Log level (trace, debug, info, warn, error) |
| `AGENT_LOG_FORMAT` | `json` | Log format (json, pretty) |
| `AGENT_GRPC_CONNECT_TIMEOUT_MS` | `1000` | gRPC connection timeout (ms) |
| `AGENT_GRPC_RPC_TIMEOUT_MS` | `1500` | gRPC RPC timeout (ms) |

### CLI Flags

```bash
haproxy-grpc-agent --help

Options:
  --server-port <PORT>          Agent server port [default: 5555]
  --metrics-port <PORT>         Metrics server port [default: 9090]
  --log-level <LEVEL>           Log level [default: info]
  --log-format <FORMAT>         Log format [default: json]
  --grpc-connect-timeout <MS>   gRPC connect timeout [default: 1000]
  --grpc-rpc-timeout <MS>       gRPC RPC timeout [default: 1500]
  --config <FILE>               Path to config file
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
```

Run with config file:

```bash
haproxy-grpc-agent --config config.toml
```

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
      - AGENT_LOG_LEVEL=info
      - AGENT_LOG_FORMAT=json
      - AGENT_GRPC_CONNECT_TIMEOUT_MS=1000
      - AGENT_GRPC_RPC_TIMEOUT_MS=1500
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
        - name: AGENT_LOG_LEVEL
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

# Run logging tests
cargo test --test logging_test -- --ignored
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
