# Integration Test Setup

This directory contains the infrastructure for integration testing the HAProxy gRPC Agent.

## Components

- **mock-backend/**: A mock gRPC server implementing `grpc.health.v1.Health` protocol
- **haproxy.cfg**: HAProxy configuration for testing
- **docker-compose.yml**: Orchestrates test environment

## Running Integration Tests

### Option 1: Docker Compose (Full Integration)

```bash
# Start all services
cd tests/integration
docker-compose up -d

# Run integration tests
cargo test --test integration_test -- --ignored --test-threads=1

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

### Option 2: Local Testing

```bash
# Terminal 1: Start mock backend
cd tests/integration/mock-backend
cargo run

# Terminal 2: Start agent
cargo run --release

# Terminal 3: Run integration tests
cargo test --test integration_test -- --ignored --test-threads=1
```

## Test Scenarios

The integration tests cover:

1. **Basic connectivity**: Verifies TCP connection to agent
2. **Healthy backend**: Tests successful health check returning "up"
3. **SSL support**: Tests TLS-enabled backends
4. **Protocol violations**: Verifies error handling returns "down"
5. **Persistent connections**: Tests multiple requests on same TCP connection
6. **Unreachable backends**: Verifies timeout handling returns "down"

## Environment Variables

Mock backend supports:
- `GRPC_PORT`: Port to listen on (default: 50051)
- `HEALTH_STATUS`: Health status to return (SERVING, NOT_SERVING, UNKNOWN, SERVICE_UNKNOWN)

Agent supports (from config.rs):
- `AGENT_SERVER_BIND_ADDRESS`: Bind address (default: 0.0.0.0)
- `AGENT_SERVER_PORT`: TCP port (default: 5555)
- `AGENT_GRPC_CONNECT_TIMEOUT_MS`: gRPC connect timeout (default: 1000)
- `AGENT_GRPC_RPC_TIMEOUT_MS`: gRPC RPC timeout (default: 1500)
- `AGENT_LOG_LEVEL`: Log level (trace, debug, info, warn, error)
- `AGENT_LOG_FORMAT`: Log format (json, pretty)
