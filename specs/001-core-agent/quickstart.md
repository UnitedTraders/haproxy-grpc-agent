# Quickstart: HAProxy gRPC Agent

**Feature**: 001-core-agent
**Date**: 2025-10-28

## Overview

This guide walks through building, deploying, and testing the HAProxy gRPC health check agent from scratch.

**Time Estimate**: 15 minutes

---

## Prerequisites

- Rust 1.75+ installed (`rustc --version`)
- Docker and Docker Compose installed
- HAProxy 3.1+ (for integration testing)
- Basic familiarity with gRPC and HAProxy

---

## Quick Start (5 minutes)

### 1. Build the Agent

```bash
# Clone repository (if applicable) or navigate to project root
cd haproxy-grpc-agent

# Build release binary
cargo build --release

# Binary location
ls -lh target/release/haproxy-grpc-agent
```

**Expected Output**: Statically-linked binary ~10-15MB

### 2. Run the Agent with Defaults

```bash
# Run with default configuration
./target/release/haproxy-grpc-agent

# Expected log output (JSON):
# {"timestamp":"2025-10-28T...","level":"info","message":"Starting HAProxy gRPC Agent"}
# {"timestamp":"2025-10-28T...","level":"info","message":"Server listening","port":5555}
# {"timestamp":"2025-10-28T...","level":"info","message":"Metrics server listening","port":9090}
```

### 3. Test Health Check (Manual)

```bash
# In another terminal, send a test request via netcat
echo "localhost 50051 no-ssl localhost" | nc localhost 5555

# Expected response:
# down
# (because no gRPC server is running on localhost:50051)
```

---

## Configuration (5 minutes)

### Configuration Methods (in order of precedence)

1. **CLI Flags** (highest priority)
2. **Config File**
3. **Environment Variables**
4. **Defaults** (lowest priority)

### Example: CLI Flags

```bash
./haproxy-grpc-agent \
  --server-port 6000 \
  --metrics-port 9100 \
  --log-level debug
```

### Example: Environment Variables

```bash
export HAPROXY_AGENT_SERVER_PORT=6000
export HAPROXY_AGENT_METRICS_PORT=9100
export HAPROXY_AGENT_LOG_LEVEL=debug

./haproxy-grpc-agent
```

### Example: Config File (TOML)

Create `config.toml`:
```toml
[server]
port = 6000
bind_address = "0.0.0.0"

[grpc]
connect_timeout_ms = 1000
rpc_timeout_ms = 1500

[metrics]
port = 9100
bind_address = "0.0.0.0"

[logging]
level = "debug"
format = "json"
```

Run with config:
```bash
./haproxy-grpc-agent --config config.toml
```

### Configuration Reference

| Setting | CLI Flag | Env Var | Default | Description |
|---------|----------|---------|---------|-------------|
| Server Port | `--server-port` | `HAPROXY_AGENT_SERVER_PORT` | 5555 | Agent Text Protocol TCP port |
| Server Bind | `--server-bind` | `HAPROXY_AGENT_SERVER_BIND` | 0.0.0.0 | Listen address |
| Metrics Port | `--metrics-port` | `HAPROXY_AGENT_METRICS_PORT` | 9090 | Prometheus metrics HTTP port |
| Metrics Bind | `--metrics-bind` | `HAPROXY_AGENT_METRICS_BIND` | 0.0.0.0 | Metrics listen address |
| gRPC Connect Timeout | `--grpc-connect-timeout` | `HAPROXY_AGENT_GRPC_CONNECT_TIMEOUT` | 1000ms | Backend connection timeout |
| gRPC RPC Timeout | `--grpc-rpc-timeout` | `HAPROXY_AGENT_GRPC_RPC_TIMEOUT` | 1500ms | Health check RPC timeout |
| Log Level | `--log-level` | `HAPROXY_AGENT_LOG_LEVEL` | info | trace/debug/info/warn/error |
| Log Format | `--log-format` | `HAPROXY_AGENT_LOG_FORMAT` | json | json or pretty |

---

## Integration Testing (5 minutes)

### Run Integration Tests

```bash
# Start test environment (HAProxy + mock gRPC backend + agent)
make test-integration

# Or manually:
cd tests/integration
docker-compose up --build
```

**What This Does**:
1. Builds agent binary in Docker
2. Starts mock gRPC backend (healthy state)
3. Starts HAProxy configured to use agent-check
4. Runs integration test suite

**Expected Output**:
```
✓ Test: HAProxy sends check, backend healthy → agent responds "up"
✓ Test: HAProxy sends check, backend down → agent responds "down"
✓ Test: HAProxy sends check, TLS backend → correct SSL handling
✓ Test: Multiple rapid checks → all responses correct
✓ Test: HAProxy disconnects abruptly → agent continues serving
```

---

## HAProxy Configuration (5 minutes)

### Configure HAProxy to Use the Agent

Add to `haproxy.cfg`:

```haproxy
backend grpc_backends
    mode tcp
    balance roundrobin

    # Configure agent check
    server backend1 192.168.1.10:50051 check agent-check agent-inter 2s agent-port 5555 agent-send "192.168.1.10 50051 no-ssl api.internal"
    server backend2 192.168.1.11:50051 check agent-check agent-inter 2s agent-port 5555 agent-send "192.168.1.11 50051 ssl api.internal"
```

**Parameter Breakdown**:
- `agent-check`: Enable agent-based health checking
- `agent-inter 2s`: Check every 2 seconds
- `agent-port 5555`: Agent TCP port
- `agent-send "..."`: Command sent to agent (Agent Text Protocol format)
  - Format: `<backend_server> <backend_port> <ssl_flag> <proxy_host>`

### Reload HAProxy

```bash
# Validate configuration
haproxy -c -f haproxy.cfg

# Reload (graceful)
systemctl reload haproxy
# or
haproxy -sf $(cat /var/run/haproxy.pid)
```

---

## Monitoring (Prometheus Metrics)

### View Metrics

```bash
# Query metrics endpoint
curl http://localhost:9090/metrics

# Example output:
# check_requests_total{result="up"} 142
# check_requests_total{result="down"} 8
# check_duration_seconds_bucket{le="0.05"} 130
# check_duration_seconds_bucket{le="0.1"} 145
# haproxy_connections_active 3
# grpc_channels_active 5
```

### Key Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `check_requests_total{result}` | Counter | Total checks (labels: up, down, error) |
| `check_duration_seconds` | Histogram | Check latency distribution |
| `check_errors_total{error_type}` | Counter | Errors (timeout, parse_error, backend_unreachable, internal) |
| `haproxy_connections_active` | Gauge | Current HAProxy connections |
| `grpc_channels_active` | Gauge | Cached gRPC channels to backends |

### Prometheus Scrape Config

Add to `prometheus.yml`:
```yaml
scrape_configs:
  - job_name: 'haproxy-grpc-agent'
    static_configs:
      - targets: ['localhost:9090']
    scrape_interval: 15s
```

---

## Logs

### View JSON Logs

```bash
# Pretty-print JSON logs with jq
./haproxy-grpc-agent | jq .

# Filter by level
./haproxy-grpc-agent | jq 'select(.level=="error")'

# Filter by component
./haproxy-grpc-agent | jq 'select(.component=="checker")'

# Follow trace_id
./haproxy-grpc-agent | jq 'select(.trace_id=="abc-123")'
```

### Log Fields

| Field | Type | Description |
|-------|------|-------------|
| `timestamp` | ISO 8601 | Log timestamp |
| `level` | String | trace/debug/info/warn/error |
| `component` | String | Module (server, checker, metrics, config) |
| `message` | String | Human-readable log message |
| `trace_id` | UUID | Request correlation ID |
| `backend` | String | Backend server:port (if applicable) |
| `ssl_flag` | String | ssl or no-ssl (if applicable) |
| `status` | String | up or down (if applicable) |
| `duration_ms` | Number | Operation duration (if applicable) |

---

## Deployment

### Docker Deployment

```bash
# Build Docker image
docker build -t haproxy-grpc-agent:latest -f deployments/docker/Dockerfile .

# Run container
docker run -d \
  --name haproxy-agent \
  -p 5555:5555 \
  -p 9090:9090 \
  -e HAPROXY_AGENT_LOG_LEVEL=info \
  haproxy-grpc-agent:latest

# Check logs
docker logs -f haproxy-agent
```

### systemd Deployment

```bash
# Copy binary
sudo cp target/release/haproxy-grpc-agent /usr/local/bin/

# Copy systemd unit file
sudo cp deployments/systemd/haproxy-grpc-agent.service /etc/systemd/system/

# Enable and start
sudo systemctl daemon-reload
sudo systemctl enable haproxy-grpc-agent
sudo systemctl start haproxy-grpc-agent

# Check status
sudo systemctl status haproxy-grpc-agent

# View logs
sudo journalctl -u haproxy-grpc-agent -f
```

---

## Troubleshooting

### Agent Not Responding

**Symptom**: HAProxy marks backends as down despite agent running

**Debug Steps**:
1. Check agent is listening: `netstat -tlnp | grep 5555`
2. Test manually: `echo "test.example.com 50051 no-ssl test.example.com" | nc localhost 5555`
3. Check logs for errors: `journalctl -u haproxy-grpc-agent | grep ERROR`

### All Checks Returning "down"

**Symptom**: Agent always responds `down\n` even for healthy backends

**Possible Causes**:
1. **Backend not running gRPC Health Check service**
   - Solution: Ensure backend implements `grpc.health.v1.Health/Check`
2. **Network connectivity issue**
   - Test: `nc -zv <backend_server> <backend_port>`
3. **Timeout too short**
   - Increase: `--grpc-rpc-timeout 3000` (3 seconds)
4. **TLS misconfiguration**
   - Verify `ssl_flag` matches backend TLS configuration

### High Latency

**Symptom**: Health checks taking >1 second

**Debug Steps**:
1. Check metrics: `curl localhost:9090/metrics | grep check_duration_seconds`
2. Enable debug logging: `--log-level debug`
3. Look for channel creation: `grep "Creating gRPC channel" logs`
   - High channel creation = connection pooling not working
4. Check backend latency: Test backend directly with `grpcurl`

### Memory Growth

**Symptom**: Agent memory usage growing over time

**Possible Causes**:
1. **Channel cache unbounded**
   - Check: `curl localhost:9090/metrics | grep grpc_channels_active`
   - If growing unbounded: implement LRU eviction (future enhancement)
2. **Connection leak**
   - Check: `curl localhost:9090/metrics | grep haproxy_connections_active`
   - Should match `netstat -an | grep :5555 | grep ESTABLISHED | wc -l`

---

## Next Steps

- **Add Observability**: Configure Prometheus + Grafana dashboards
- **Production Hardening**: Set resource limits, configure log rotation
- **Advanced Config**: Tune timeouts based on your backend characteristics
- **Scaling**: Run multiple agent instances behind load balancer if needed

---

## Reference

- **Agent Text Protocol**: See `specs/001-core-agent/contracts/agent-text-protocol.md`
- **Configuration Options**: See `docs/config-reference.md`
- **Deployment Guide**: See `docs/deployment-guide.md`
- **HAProxy Documentation**: https://www.haproxy.org/download/3.1/doc/configuration.txt
