# Agent Text Protocol Specification

**Version**: 1.0
**Feature**: 001-core-agent
**Date**: 2025-10-28

## Overview

The Agent Text Protocol is a simple line-based TCP protocol used by HAProxy to query backend server health status. This agent implements the server side of this protocol.

---

## Connection Model

- **Transport**: TCP
- **Port**: Configurable (default 5555)
- **Encoding**: UTF-8 text
- **Line Terminator**: `\n` (newline, ASCII 0x0A)
- **Connection Type**: Persistent (HAProxy may send multiple requests per connection)

---

## Request Format

Each request is a single line with 4 whitespace-separated fields:

```
<backend_server_name> <backend_server_port> <ssl_flag> <proxy_host_name>\n
```

### Field Definitions

| Field | Type | Description | Validation |
|-------|------|-------------|------------|
| `backend_server_name` | String | Backend server name or IP to check | Non-empty, valid hostname or IP |
| `backend_server_port` | Integer | Backend server port to check | 1-65535 |
| `ssl_flag` | Enum | TLS mode for gRPC connection | Must be `ssl` or `no-ssl` |
| `proxy_host_name` | String | Value for Host/authority header in gRPC request | Non-empty string |

### Example Requests

**Plain gRPC backend check**:
```
backend-api.example.com 50051 no-ssl backend-api.example.com
```

**TLS gRPC backend check**:
```
secure-backend.example.com 443 ssl secure-backend.example.com
```

**IP address backend**:
```
192.168.1.100 9090 no-ssl api.internal
```

---

## Response Format

Each response is a single line with the health status:

```
<status>\n
```

### Status Values

| Status | Meaning | When to Return |
|--------|---------|----------------|
| `up` | Backend is healthy | gRPC health check succeeded |
| `down` | Backend is unhealthy | gRPC health check failed, backend unreachable, timeout, or internal error |
| `maint` | Backend is in maintenance | Reserved for future use (not in MVP) |

### Example Responses

**Successful health check**:
```
up
```

**Failed health check**:
```
down
```

---

## Error Handling

### Invalid Request Format

**Scenario**: Request does not have exactly 4 fields

**Agent Behavior**:
1. Log warning with trace_id: `WARN: Protocol violation: expected 4 fields, got N`
2. Respond with `down\n`
3. Continue accepting requests (do not close connection)

**Example**:
```
Input:  backend.example.com 50051\n
Output: down\n
Log:    {"level":"warn","message":"Protocol violation: expected 4 fields, got 2","trace_id":"..."}
```

### Invalid ssl_flag

**Scenario**: `ssl_flag` is not `ssl` or `no-ssl`

**Agent Behavior**:
1. Log warning: `WARN: Invalid ssl_flag: {value}`
2. Respond with `down\n`
3. Continue accepting requests

### Invalid Port

**Scenario**: `backend_server_port` is not a valid u16 (1-65535)

**Agent Behavior**:
1. Log warning: `WARN: Invalid port: {value}`
2. Respond with `down\n`
3. Continue accepting requests

### Backend Unreachable

**Scenario**: gRPC connection to backend fails

**Agent Behavior**:
1. Log error: `ERROR: Backend unreachable: {backend}:{port}`
2. Respond with `down\n`
3. Increment metric: `check_errors_total{error_type="backend_unreachable"}`

### Timeout

**Scenario**: gRPC health check exceeds timeout (default 2s total)

**Agent Behavior**:
1. Log error: `ERROR: Health check timeout: {backend}:{port}`
2. Respond with `down\n`
3. Increment metric: `check_errors_total{error_type="timeout"}`

### Internal Agent Error

**Scenario**: Unexpected error within agent (panic recovery, resource exhaustion, etc.)

**Agent Behavior**:
1. Log error: `ERROR: Internal error: {details}`
2. Respond with `down\n` (fail-safe default)
3. Increment metric: `check_errors_total{error_type="internal"}`
4. Do NOT crash agent (graceful degradation)

---

## Connection Lifecycle

### Establish Connection

1. HAProxy connects to agent TCP port
2. Agent accepts connection
3. Agent increments metric: `haproxy_connections_active`
4. Agent logs: `INFO: HAProxy connection established`

### Handle Requests

1. Agent reads line from TCP stream (blocks until `\n`)
2. Agent parses request into fields
3. Agent performs gRPC health check to backend
4. Agent writes response (`up\n` or `down\n`)
5. Repeat from step 1 (persistent connection)

### Close Connection

**Normal Close** (HAProxy initiates):
1. HAProxy sends FIN or closes socket
2. Agent receives EOF on TCP stream
3. Agent decrements metric: `haproxy_connections_active`
4. Agent logs: `INFO: HAProxy connection closed`

**Abrupt Close** (network failure):
1. Agent detects connection reset
2. Agent decrements metric: `haproxy_connections_active`
3. Agent logs: `WARN: HAProxy connection reset`

**Agent-Initiated Close** (future enhancement):
- Not supported in MVP
- Agent never closes connection unilaterally

---

## Performance Characteristics

### Latency

- **Target**: <2s total (HAProxy default timeout)
- **Breakdown**:
  - Parse request: <1ms
  - gRPC connection setup: <1s (cached channels: <10ms)
  - gRPC health check RPC: <1.5s
  - Write response: <1ms

### Throughput

- **Concurrent Connections**: 1000+ HAProxy connections
- **Requests per Second**: 1000+ health checks/sec
- **Memory**: <50MB under normal load

### Timeouts

- **gRPC Connect Timeout**: 1000ms (configurable)
- **gRPC RPC Timeout**: 1500ms (configurable)
- **Total Timeout**: <2000ms (HAProxy agent-check timeout)

---

## Security Considerations

### Input Validation

- All fields validated before use
- Malformed input returns `down` (fail-safe)
- No code injection possible (no eval/shell execution)

### Resource Limits

- Connection limit: OS-level (ulimit)
- Memory limit: Bounded by channel cache size
- No request size limit (single line, typically <256 bytes)

### TLS

- Agent does NOT support TLS for HAProxy connection (plain TCP)
- `ssl_flag` controls TLS for agent→backend gRPC connection

---

## Observability

### Logs

Every request logs:
```json
{
  "timestamp": "2025-10-28T12:34:56.789Z",
  "level": "info",
  "component": "server",
  "message": "Health check completed",
  "trace_id": "uuid-v4",
  "backend": "backend.example.com:50051",
  "ssl_flag": "no-ssl",
  "status": "up",
  "duration_ms": 123
}
```

### Metrics

- `check_requests_total{result="up|down|error"}`: Counter
- `check_duration_seconds`: Histogram (buckets: 0.01, 0.05, 0.1, 0.5, 1.0, 2.0)
- `check_errors_total{error_type="..."}`: Counter
- `haproxy_connections_active`: Gauge

---

## Compliance

This specification implements the HAProxy Agent Check Protocol as documented in HAProxy 3.1+ documentation with the following constraints:

- **Supported Responses**: `up`, `down` only (MVP)
- **Future**: `maint`, optional weight/reason fields
- **Extension**: gRPC backend health checking (not in standard HAProxy agent-check protocol)

---

## Example Session

```
# HAProxy connects
→ (connection established)
← INFO: HAProxy connection established

# Request 1: Healthy backend
→ api.example.com 50051 no-ssl api.example.com\n
← up\n
← INFO: Health check completed (status=up, backend=api.example.com:50051, duration=85ms)

# Request 2: Unhealthy backend
→ down-service.example.com 50051 no-ssl down-service.example.com\n
← down\n
← ERROR: Backend unreachable (backend=down-service.example.com:50051)

# Request 3: Invalid format
→ invalid request\n
← down\n
← WARN: Protocol violation: expected 4 fields, got 2

# HAProxy disconnects
→ (connection closed)
← INFO: HAProxy connection closed
```

---

## References

- HAProxy Agent Check Documentation: https://www.haproxy.org/download/3.1/doc/configuration.txt (search "agent-check")
- gRPC Health Checking Protocol: https://github.com/grpc/grpc/blob/master/doc/health-checking.md
