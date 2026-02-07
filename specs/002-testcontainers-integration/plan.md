# Implementation Plan: Testcontainers Integration Tests

**Branch**: `002-testcontainers-integration` | **Date**: 2026-02-07 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/002-testcontainers-integration/spec.md`

## Summary

Refactor integration tests from docker-compose-based external orchestration to self-contained testcontainers-driven tests runnable via `cargo test`. Add new resilience tests verifying agent behavior during backend disconnect and reload scenarios, operating without HAProxy — only the agent and mock backend.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2024)
**Primary Dependencies**: testcontainers 0.27 (new), tokio 1.x, tonic 0.14, existing mock-grpc-backend Docker image
**Storage**: N/A
**Testing**: cargo test with testcontainers (async runner), tokio::test
**Target Platform**: Linux/macOS (Docker required)
**Project Type**: Single Rust project
**Performance Goals**: Full integration suite completes within 2 minutes
**Constraints**: Docker daemon must be running; mock backend image must be pre-built
**Scale/Scope**: ~10 integration test functions across 2 test files

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Agent Pattern | PASS | No changes to agent binary or its dependencies. testcontainers is dev-dependency only. |
| II. Integration-Heavy Testing | PASS | This feature directly enhances integration testing. Resilience tests exercise real container stop/start flows. |
| III. Observability | PASS | No changes to logging or metrics. Existing structured logging preserved. |
| IV. HAProxy Protocol Compliance | PASS | Existing protocol tests preserved with equivalent assertions. New resilience tests verify agent protocol responses during backend failures. |
| V. Simplicity & Reliability | PASS | testcontainers replaces more complex docker-compose orchestration. In-process agent startup eliminates subprocess management. No new abstractions beyond necessary test utilities. |

**Post-Phase 1 re-check**: All gates remain PASS. The design adds `testcontainers` as a single new dev-dependency (justified: replaces docker-compose + bollard for cleaner test lifecycle). No production code complexity increases.

## Project Structure

### Documentation (this feature)

```text
specs/002-testcontainers-integration/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── spec.md              # Feature specification
└── checklists/
    └── requirements.md  # Spec quality checklist
```

### Source Code (repository root)

```text
src/
├── server.rs            # MODIFY: Expose bound address for dynamic port allocation
├── lib.rs               # No changes (already exports server module)
├── checker.rs           # No changes
├── config.rs            # No changes
├── main.rs              # No changes
├── metrics.rs           # No changes
├── protocol.rs          # No changes
└── logger.rs            # No changes

tests/
├── integration_test.rs  # REWRITE: Migrate to testcontainers
├── resilience_test.rs   # NEW: Backend disconnect/reload tests
├── logging_test.rs      # No changes
└── common/
    └── mod.rs           # REWRITE: Shared test utilities (agent startup, container helpers)

tests/integration/
├── mock-backend/        # No changes (Dockerfile + source preserved for image building)
├── docker-compose.yml   # REMOVE (or keep for reference)
├── haproxy.cfg          # No changes (not used by new tests but retained)
└── README.md            # UPDATE: Document new test approach

Makefile                 # MODIFY: Update test-integration target
Cargo.toml               # MODIFY: Replace bollard with testcontainers in dev-dependencies
```

**Structure Decision**: Single project layout. Tests live in `tests/` directory as integration test files. The existing `src/` structure is unchanged except for a minor `server.rs` modification to support dynamic port binding.

## Complexity Tracking

No constitution violations to justify.

## Design Decisions

### D1: Agent Server Modification for Dynamic Port Binding

The current `AgentServer::run()` binds to `config.server_port` internally and runs the accept loop. Tests need to know the actual bound port when using port 0.

**Approach**: Add a `run_with_listener` method that accepts an already-bound `TcpListener`, and modify `run()` to call it. This way:
- `main.rs` continues calling `run()` as before (no behavior change)
- Tests bind to port 0, get the `SocketAddr`, and pass the listener to `run_with_listener()`

```rust
// In server.rs
pub async fn run_with_listener(&self, listener: TcpListener) -> Result<()> {
    // existing accept loop, using provided listener
}

pub async fn run(&self) -> Result<()> {
    let bind_addr = format!("{}:{}", self.config.server_bind_address, self.config.server_port);
    let listener = TcpListener::bind(&bind_addr).await?;
    self.run_with_listener(listener).await
}
```

### D2: Test Utility Module (`tests/common/mod.rs`)

Shared helpers for both test files:

- `build_mock_image()` — Ensures the mock-grpc-backend Docker image is built (runs `docker build` once per test session using `std::sync::Once`)
- `start_mock_backend(health_status: &str)` — Starts mock backend container via testcontainers with given `HEALTH_STATUS` env var, returns `ContainerAsync` + mapped port
- `start_agent(backend_port: u16)` — Creates `AgentConfig` with port 0, binds `TcpListener`, spawns agent on tokio task, returns `(JoinHandle, SocketAddr)`
- `send_check(agent_addr: SocketAddr, backend: &str, port: u16)` — Sends a health check request to agent and returns response string
- `cleanup_agent(handle: JoinHandle)` — Aborts the agent task

### D3: Mock Backend Image Build Strategy

The mock backend image needs to exist before testcontainers can use it. Strategy:

1. Add `docker-build-mock` Makefile target: `docker build -t mock-grpc-backend:latest tests/integration/mock-backend/`
2. In test utilities, use `std::sync::Once` to run `docker build` on first test invocation
3. Image tag: `mock-grpc-backend:latest`

The `std::sync::Once` approach ensures the image is built exactly once per test run, even when multiple test files or tests execute.

### D4: Test Organization

**`tests/integration_test.rs`** — Migrated existing tests:
- `test_agent_connectivity` — Start mock backend + agent via testcontainers/in-process, connect to agent
- `test_health_check_healthy_backend` — Send check request, expect "up"
- `test_health_check_with_ssl` — Send SSL request to non-existent SSL backend, expect "down"
- `test_protocol_violation` — Send malformed request, expect "down"
- `test_persistent_connection` — Multiple requests on same TCP connection
- `test_unreachable_backend` — Check non-existent backend, expect "down"

All tests remove `#[ignore]` and become self-contained.

**`tests/resilience_test.rs`** — New tests (no HAProxy):
- `test_backend_disconnect` — Start agent + backend, verify "up", stop backend container, verify "down"
- `test_backend_recovery` — Start agent + backend, stop backend, restart backend, verify "up" again
- `test_backend_status_change_on_reload` — Start with SERVING, stop, start new container with NOT_SERVING, verify "down"
- `test_cached_connection_invalidated_on_disconnect` — Verify agent doesn't serve stale "up" from cached gRPC channel after backend stops

### D5: Cargo.toml Changes

```toml
[dev-dependencies]
testcontainers = "0.27"
serde_json = "1.0"
# Remove bollard (replaced by testcontainers which uses bollard internally)
```

### D6: Makefile Changes

```makefile
# Build mock backend Docker image
docker-build-mock:
	docker build -t mock-grpc-backend:latest tests/integration/mock-backend/

# Run integration tests (self-contained via testcontainers)
test-integration: docker-build-mock
	cargo test --test integration_test --test resilience_test -- --test-threads=1

# Full test suite
test: test-unit test-integration
```

Note: `--test-threads=1` because tests manipulate container state and share Docker daemon resources. Parallel test execution could be enabled later with per-test isolated containers but adds complexity.

## File Change Summary

| File | Action | Reason |
|------|--------|--------|
| `Cargo.toml` | MODIFY | Add testcontainers, remove bollard |
| `src/server.rs` | MODIFY | Add `run_with_listener()` for dynamic port binding |
| `tests/common/mod.rs` | REWRITE | Test utilities: image build, container start, agent start |
| `tests/integration_test.rs` | REWRITE | Migrate 6 tests to testcontainers, remove `#[ignore]` |
| `tests/resilience_test.rs` | NEW | 4 new disconnect/reload tests |
| `Makefile` | MODIFY | Update test-integration target, add docker-build-mock |
