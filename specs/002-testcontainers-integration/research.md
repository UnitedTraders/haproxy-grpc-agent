# Research: Testcontainers Integration Tests

**Date**: 2026-02-07
**Feature**: 002-testcontainers-integration

## R1: Testcontainers Crate Selection & API

**Decision**: Use `testcontainers` v0.27 with `GenericImage` for the mock backend container.

**Rationale**: The `testcontainers` crate (v0.27) provides:
- Async API via `runners::AsyncRunner` compatible with tokio `#[tokio::test]`
- `GenericImage` for referencing pre-built Docker images without custom trait implementations
- `container.stop()` / `container.start()` on `&self` for stop/restart without recreating
- `container.get_host_port_ipv4(port)` for dynamic port mapping
- Automatic cleanup on `Drop` (container removed when variable goes out of scope)
- `WaitFor::message_on_stdout()` for deterministic readiness detection
- `ImageExt::with_env_var()` for configuring the mock backend's `HEALTH_STATUS`

**Alternatives considered**:
- `bollard` (already in dev-dependencies): Too low-level for test orchestration; requires manual lifecycle management. Good for Docker API access but not for test fixtures.
- `testcontainers-modules`: Only provides pre-built modules for common services (Redis, Postgres). No gRPC health backend module. Not needed since we have our own image.
- `docker-compose` (current approach): Requires external orchestration, hardcoded ports, manual cleanup. Cannot be self-contained in `cargo test`.

## R2: Mock Backend Image Strategy

**Decision**: Pre-build the mock backend Docker image before tests run, referenced by `GenericImage::new("mock-grpc-backend", "latest")`.

**Rationale**: Testcontainers works with pre-built Docker images. The mock backend at `tests/integration/mock-backend/` has a Dockerfile that produces a small image. A build step (either `make docker-build-mock` or a `build.rs` / shell script) will ensure the image exists before tests execute. This avoids building the image inside each test run.

**Alternatives considered**:
- Build image within each test: Slow (Rust compilation takes minutes), not practical for CI.
- Embed mock backend as in-process code: Would require making the mock backend a library crate importable by tests. More invasive refactor, but eliminates Docker dependency. However, this loses the ability to test real container stop/start/network behavior.
- Use `GenericImage::new()` with an assumption image exists: Simplest approach, requires a prerequisite build step. Best balance of simplicity and correctness.

## R3: Agent Process Lifecycle in Tests

**Decision**: Start the agent as a library-level server within the test process using `AgentServer::new(config).run()` spawned on a tokio task, rather than as a subprocess.

**Rationale**: The agent already exposes its server through `lib.rs` → `server::AgentServer`. Starting it in-process:
- Avoids building and spawning a separate binary
- Allows direct configuration via `AgentConfig` struct with dynamic ports
- Cleanup is trivial (abort the tokio task)
- No port conflicts with hardcoded values — use port 0 and bind dynamically
- Faster test startup (no process spawn overhead)

The `AgentServer::run()` method takes `&self` and runs a TCP accept loop. It can be spawned as a tokio task and aborted on cleanup.

**Note**: The current `AgentServer::run()` does not return the bound address (it binds internally). A small refactor to expose the actual bound port (e.g., binding to port 0 and returning the `SocketAddr`) will be needed for dynamic port allocation.

**Alternatives considered**:
- Spawn agent as subprocess (`Command::new`): Similar to current logging tests approach. Works but slower, harder to configure dynamically, requires binary to be built first.
- Containerize agent too: Overkill — the agent is the system under test, not a dependency. In-process testing gives faster feedback and easier debugging.

## R4: Container Stop/Restart for Disconnect Testing

**Decision**: Use `container.stop().await` / `container.start().await` on the same `ContainerAsync` instance for disconnect/reconnect tests.

**Rationale**: Testcontainers v0.27 supports stop/start on `&self`, preserving the container and its port mappings. This directly models:
- Backend crash: `container.stop()` — kills the process, TCP connections break
- Backend recovery: `container.start()` — restarts on same port mapping

For the "reload with changed status" scenario, we'll need to stop the current container and start a new one with different `HEALTH_STATUS` env var, since environment variables cannot be changed on a stopped container.

**Alternatives considered**:
- `container.pause()` / `container.unpause()`: Freezes processes (SIGSTOP) but TCP connections remain open. Useful for simulating network partition/timeout, but doesn't model a real process crash.
- Kill and recreate container: Works but loses the port mapping, requiring re-querying the new port. Acceptable for the "reload with different status" scenario.

## R5: Test Structure & Organization

**Decision**: Create a new test file `tests/integration_test.rs` (replacing the existing one) with testcontainers-based tests, and add `tests/resilience_test.rs` for the new disconnect/reload tests.

**Rationale**: Separating resilience tests (which manipulate container lifecycle) from basic integration tests (which use a stable backend) keeps test intent clear and allows independent execution. Both files share common test utilities from `tests/common/mod.rs`.

**Alternatives considered**:
- Single test file: All tests in one file. Becomes unwieldy with 10+ test functions and shared setup.
- Test per scenario: Too many files for this scope. Two files (integration + resilience) is the right granularity.

## R6: Dynamic Port Allocation

**Decision**: Use port 0 for the agent server bind and retrieve the actual bound port from the `TcpListener`. Use testcontainers' dynamic port mapping for the mock backend container.

**Rationale**: Port 0 tells the OS to assign an available ephemeral port. This:
- Eliminates port conflicts between parallel test runs
- Removes hardcoded port assumptions (5555, 50051)
- Is standard practice in integration testing

**Implementation**: `AgentServer::run()` currently binds to `config.server_port`. We'll add a method that returns the bound address, or modify `run()` to accept a `tokio::sync::oneshot::Sender` to communicate the bound address. Alternatively, bind externally and pass the `TcpListener` to `run()`.

## R7: Makefile Updates

**Decision**: Update `test-integration` target to run `cargo test --test integration_test --test resilience_test` instead of docker-compose. Add a `docker-build-mock` target for building the mock backend image.

**Rationale**: The Makefile should reflect the new workflow. The old docker-compose-based target becomes obsolete. Keeping `make test-integration` as the entry point preserves developer muscle memory.

**Alternatives considered**:
- Remove Makefile targets entirely: Too disruptive; developers expect `make test-integration`.
- Keep docker-compose as fallback: Unnecessary maintenance burden if testcontainers works.
