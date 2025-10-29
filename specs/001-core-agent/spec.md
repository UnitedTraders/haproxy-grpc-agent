# Feature Specification: Core HAProxy gRPC Agent

**Feature Branch**: `001-core-agent`
**Created**: 2025-10-28
**Status**: Draft
**Input**: Implement a lightweight gRPC-checking agent that responds to HAProxy health check requests according to the Agent Text Protocol.

Request format:
```
<backend_server_name> <backend_server_port> <ssl_flag> <proxy_host_name>\n
```

Where:
- `ssl_flag` - is one of `ssl` or `no-ssl`. Value `ssl` means that gRPC channel should be over TLS, `no-ssl` for plain gRPC
- `backend_server_name` - server name or IP to check
- `backend_server_port` - server port to check
- `proxy_host_name` - value to put in Host header of gRPC request

Successful response:
```
up\n
```

Failed response
```
down\n
```

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Basic Health Check Response (Priority: P1)

As a HAProxy administrator, I want the gRPC agent to respond to health check requests with correct status information so that HAProxy can make accurate routing decisions for backend servers.

Upon receiving command with Agent Text Protocol gRPC agent should
1. Create/reuse gRPC channel to starts channel to <backend_server_name>:<backend_server_port>
2. Call service `grpc.health.v1.Health` method `Check` with empty `service` in request
3. Convert resulting `HealthCheckResponse` to `up` for `SERVING` or `down` for any other returned value

Specification for `grpc.health.v1.Health` can be found in [Proto file](https://github.com/grpc/grpc-proto/blob/master/grpc/health/v1/health.proto).

**Why this priority**: This is the core functionality - without reliable health check responses, the agent has no value. This is the MVP that enables all other features.

**Independent Test**: Can be fully tested by deploying HAProxy with agent-check configuration pointing to the gRPC agent, sending health check requests, and verifying correct `up|down` responses are returned within the timeout window.

**Acceptance Scenarios**:

1. **Given** a running gRPC agent, **When** HAProxy sends a health check request, **Then** the agent responds with a valid status (`up`, `down`, or `maint`) within 2 seconds
2. **Given** a healthy backend service, **When** the agent is queried, **Then** the agent returns `up` status
3. **Given** an unhealthy backend service, **When** the agent is queried, **Then** the agent returns `down` status
4. **Given** a backend in maintenance mode, **When** the agent is queried, **Then** the agent returns `maint` status
5. **Given** HAProxy sends multiple rapid health checks, **When** the agent processes them, **Then** all responses are delivered correctly without connection issues

---

### User Story 2 - Agent Configuration (Priority: P2)

As a deployment engineer, I want to configure the gRPC agent via environment variables, CLI flags, or a config file so that I can adapt it to different environments without code changes.

**Why this priority**: After basic functionality works, configurability is essential for production deployment across different environments (dev, staging, prod) and infrastructure patterns.

**Independent Test**: Can be tested by starting the agent with different configuration methods (env vars, CLI flags, config file) and verifying the agent adopts the specified settings (port, log level, backend check endpoint, etc.).

**Acceptance Scenarios**:

1. **Given** environment variables are set, **When** the agent starts, **Then** it uses those configuration values
2. **Given** CLI flags are provided, **When** the agent starts, **Then** CLI flags override environment variables
3. **Given** a config file exists, **When** the agent starts with `--config` flag, **Then** it loads settings from the file
4. **Given** invalid configuration is provided, **When** the agent starts, **Then** it fails fast with a clear error message explaining what's wrong
5. **Given** no configuration is provided, **When** the agent starts, **Then** it uses sensible defaults and logs the active configuration

---

### User Story 3 - Structured Logging (Priority: P3)

As an operations engineer, I want the agent to emit structured JSON logs with trace IDs and clear error messages so that I can debug issues and analyze agent behavior in production.

**Why this priority**: Once the agent is functioning and configurable, observability becomes critical for production operations and troubleshooting.

**Independent Test**: Can be tested by running the agent with different log levels, triggering various operations (startup, health checks, errors), and verifying that structured JSON logs are emitted with correct fields (timestamp, level, component, message, trace_id).

**Acceptance Scenarios**:

1. **Given** the agent starts up, **When** initialization completes, **Then** configuration details are logged at INFO level in JSON format
2. **Given** a health check request arrives, **When** the agent processes it, **Then** the request and response are logged with a trace_id
3. **Given** an error occurs during health check, **When** the agent handles it, **Then** an ERROR log is emitted with actionable context (what failed, why, what to check)
4. **Given** log level is set to DEBUG, **When** operations occur, **Then** detailed execution traces are logged
5. **Given** log level is set to ERROR, **When** normal operations occur, **Then** only errors are logged (not INFO/DEBUG)

---

### User Story 4 - Prometheus Metrics (Priority: P4)

As a monitoring engineer, I want the agent to expose Prometheus-compatible metrics on a `/metrics` endpoint so that I can monitor agent health, performance, and error rates.

**Why this priority**: Metrics enable proactive monitoring and alerting. This is important but comes after basic functionality, configuration, and logging.

**Independent Test**: Can be tested by querying the `/metrics` endpoint and verifying that Prometheus-formatted metrics are returned, including counters for check requests, histograms for latency, and gauges for active connections.

**Acceptance Scenarios**:

1. **Given** the agent is running, **When** `/metrics` endpoint is queried, **Then** Prometheus-formatted metrics are returned
2. **Given** health check requests are processed, **When** metrics are scraped, **Then** `check_requests_total` counter reflects the number of requests
3. **Given** health checks have varying latencies, **When** metrics are scraped, **Then** `check_duration_seconds` histogram shows latency distribution
4. **Given** errors occur during health checks, **When** metrics are scraped, **Then** `check_errors_total` counter reflects error count
5. **Given** HAProxy connections are active, **When** metrics are scraped, **Then** `grpc_connections_active` gauge shows current connection count

---

### Edge Cases

- What happens when HAProxy disconnects abruptly during a health check?
- How does the agent handle requests when the backend service is unreachable? - returns `down`
- What if the gRPC service fails to start (port already in use)? - agent should exit
- How does the agent behave under high load (1000+ concurrent health checks)? - undefined
- What if configuration changes while the agent is running? - nothing, restart is required to re-read configuration
- How does the agent handle protocol violations from HAProxy? - should report to logger with WARN status

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST expose a plain text socket that confirms Agent Text Protocol
- **FR-002**: System MUST respond to health check requests with valid status: `up`, `down`, or `maint`
- **FR-003**: System MUST complete health check responses within 2 seconds (default HAProxy timeout)
- **FR-004**: System MUST handle persistent connections from HAProxy
- **FR-005**: System MUST log all HAProxy communication (requests, responses, connection state)
- **FR-006**: System MUST support configuration via environment variables, CLI flags, and config file
- **FR-007**: System MUST emit structured JSON logs with fields: timestamp, level, component, message, trace_id
- **FR-008**: System MUST expose Prometheus metrics endpoint on `/metrics`
- **FR-009**: System MUST report `down` status on internal errors (fail-safe default)
- **FR-010**: System MUST continue serving health checks even if metrics/logging subsystems fail
- **FR-011**: System MUST be deployable as a single statically-linked binary
- **FR-012**: System MUST validate configuration on startup and fail fast with clear errors if invalid
- **FR-013**: System MUST be testable against HAProxy versions 3.1, 3.2, and latest stable

### Key Entities *(include if feature involves data)*

- **Health Check Request**: text request from HAProxy asking for backend status
  - Attributes: backend_server_name, backend_server_port, proxy_host_name
  - Source: HAProxy instance

- **Health Check Response**: text response sent back to HAProxy
  - Value: status (up|down|maint)
  - Format: Must conform to Agent Text Protocol specification

- **Agent Configuration**: Settings that control agent behavior
  - Attributes: grpc_port, metrics_port, log_level, backend_check_endpoint, check_timeout
  - Sources: env vars, CLI flags, config file (precedence: CLI > file > env > defaults)

- **Metrics Data**: Prometheus-compatible metrics about agent operations
  - Counters: check_requests_total, check_errors_total (by error type)
  - Histograms: check_duration_seconds
  - Gauges: grpc_connections_active

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Agent responds to 100% of valid health check requests within 2 seconds
- **SC-002**: Agent handles at least 1000 concurrent health check requests without degradation
- **SC-003**: Integration tests pass against all supported HAProxy versions (3.1, 3.2, latest stable)
- **SC-004**: All HAProxy communication is logged with structured JSON and trace IDs for debugging
- **SC-005**: Metrics endpoint responds within 100ms and includes all required metrics
- **SC-006**: Agent can be deployed and configured in under 5 minutes using provided documentation
- **SC-007**: Agent binary size is under 20MB (statically linked)
- **SC-008**: Agent memory footprint is under 50MB under normal load (100 req/s)
- **SC-009**: Zero data loss on graceful shutdown (in-flight requests complete)
- **SC-010**: Startup time is under 5 seconds
