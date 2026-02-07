# Quickstart: Testcontainers Integration Tests

## Prerequisites

- Docker daemon running
- Rust toolchain (1.75+)

## Build Mock Backend Image

```bash
make docker-build-mock
# or manually:
docker build -t mock-grpc-backend:latest tests/integration/mock-backend/
```

## Run Integration Tests

```bash
# All integration + resilience tests
make test-integration

# Or directly via cargo:
cargo test --test integration_test --test resilience_test -- --test-threads=1

# Run only resilience tests (disconnect/reload):
cargo test --test resilience_test -- --test-threads=1

# Run only migrated integration tests:
cargo test --test integration_test -- --test-threads=1
```

## Debug Failing Tests

Keep containers alive after test failure for inspection:

```bash
TESTCONTAINERS_COMMAND=keep cargo test --test resilience_test -- --test-threads=1 --nocapture
```

Then inspect running containers:

```bash
docker ps  # See containers still running
docker logs <container_id>  # Check mock backend logs
```

## Key Test Utilities

Tests share utilities from `tests/common/mod.rs`:

- `start_mock_backend("SERVING")` — Starts containerized mock gRPC backend
- `start_agent(backend_port)` — Starts agent in-process on dynamic port
- `send_check(agent_addr, "host", port)` — Sends health check and returns response
