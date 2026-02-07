# Feature Specification: Testcontainers Integration Tests

**Feature Branch**: `002-testcontainers-integration`
**Created**: 2026-02-07
**Status**: Draft
**Input**: User description: "Current tests use docker-compose ran by make externally. Refactor integration tests so they use 'testcontainer' (https://crates.io/crates/testcontainers). And add tests for disconnect or reload of mock backend that do not use haproxy, only agent and mock backend."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Self-Contained Integration Tests (Priority: P1)

As a developer, I want integration tests to manage their own container lifecycle using the testcontainers library so that I can run tests with a single `cargo test` command without needing to manually start docker-compose or any external infrastructure beforehand.

**Why this priority**: This is the foundation of the entire feature. Without self-contained test infrastructure, no other test scenarios can be automated. Eliminating the docker-compose dependency removes the primary friction point in the current developer workflow.

**Independent Test**: Can be fully tested by running `cargo test` and verifying that containers start automatically, tests execute, and containers are cleaned up — all without any manual setup steps.

**Acceptance Scenarios**:

1. **Given** the developer has Docker running locally, **When** they execute `cargo test --test integration_test`, **Then** the mock gRPC backend container starts automatically, the agent process starts, tests execute, and all resources are cleaned up after completion.
2. **Given** the developer runs integration tests, **When** a test completes (pass or fail), **Then** all spawned containers and processes are stopped and removed without manual intervention.
3. **Given** the existing integration test scenarios (connectivity, healthy backend, SSL flag handling, protocol violation, persistent connection, unreachable backend), **When** migrated to use testcontainers, **Then** all existing test scenarios continue to pass with equivalent assertions.

---

### User Story 2 - Backend Disconnect Resilience Tests (Priority: P2)

As a developer, I want tests that verify the agent correctly handles a backend that suddenly disconnects (TCP connection drop) so that I can be confident the agent reports the correct health status to HAProxy when backends go away unexpectedly.

**Why this priority**: Backend disconnects are a common production failure mode. Validating agent behavior during disconnect is critical for reliability. These tests operate only against the agent and mock backend (no HAProxy required), making them simpler to implement and faster to run.

**Independent Test**: Can be tested by starting the agent and mock backend, verifying a healthy check succeeds, then stopping the mock backend container, and verifying the agent reports the backend as down.

**Acceptance Scenarios**:

1. **Given** the agent is running and the mock backend is healthy, **When** the mock backend container is stopped (simulating a crash/disconnect), **Then** the agent reports the backend as down on the next health check request.
2. **Given** the mock backend was previously stopped, **When** the mock backend container is restarted, **Then** the agent reports the backend as up on a subsequent health check request.
3. **Given** the agent has a cached connection to the mock backend, **When** the backend disappears, **Then** the agent does not continue reporting the backend as up due to stale cached state.

---

### User Story 3 - Backend Reload/Restart Resilience Tests (Priority: P3)

As a developer, I want tests that verify the agent correctly handles a backend that restarts or reloads (e.g., during a rolling deployment) so that I can be confident the agent recovers gracefully and resumes reporting healthy status after the backend comes back.

**Why this priority**: Deployments and service restarts are routine operations. Verifying that the agent handles the full stop-start cycle correctly ensures smooth operations during maintenance windows. These tests also operate without HAProxy.

**Independent Test**: Can be tested by starting the agent and mock backend, performing a health check, stopping and restarting the mock backend with a different health status, and verifying the agent reflects the new status.

**Acceptance Scenarios**:

1. **Given** the agent is connected to a healthy mock backend, **When** the mock backend is stopped and restarted with a NOT_SERVING health status, **Then** the agent reports the backend as down after the restart.
2. **Given** the agent is connected to a NOT_SERVING mock backend, **When** the mock backend is stopped and restarted with a SERVING health status, **Then** the agent reports the backend as up after the restart.
3. **Given** the mock backend restarts within the agent's gRPC connection timeout window, **When** the agent performs a health check, **Then** the check completes without the agent itself crashing or hanging.

---

### Edge Cases

- What happens when the mock backend container fails to start (e.g., port conflict)? Tests should produce a clear error rather than hang.
- What happens when the agent performs a health check at the exact moment the backend is shutting down? The agent should report down, not panic or hang.
- What happens when multiple sequential stop/start cycles are performed? The agent should consistently track the backend's current state.
- What happens when the backend restarts on a different port? The agent should report down for the original address.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: Integration tests MUST use the testcontainers library to manage the mock gRPC backend container lifecycle programmatically from within Rust test code.
- **FR-002**: Integration tests MUST NOT require docker-compose or any external orchestration to run. Running `cargo test` with Docker available MUST be sufficient.
- **FR-003**: All existing integration test scenarios (agent connectivity, healthy backend check, SSL flag handling, protocol violation, persistent connection, unreachable backend) MUST be preserved with equivalent coverage after migration.
- **FR-004**: The mock backend container image MUST be built or available such that testcontainers can use it (either pre-built image or built as part of test setup).
- **FR-005**: A backend disconnect test MUST verify that the agent reports "down" when a previously healthy backend container is stopped.
- **FR-006**: A backend recovery test MUST verify that the agent reports "up" when a previously stopped backend container is restarted in a healthy state.
- **FR-007**: A backend reload test MUST verify that the agent correctly reflects a changed health status (e.g., SERVING to NOT_SERVING) after a backend restart.
- **FR-008**: All new resilience tests (disconnect, recovery, reload) MUST operate only with the agent and mock backend — no HAProxy container required.
- **FR-009**: Container cleanup MUST occur automatically after each test, whether the test passes or fails, to prevent resource leaks.
- **FR-010**: The agent process MUST be started programmatically within each test (or test fixture) and stopped during cleanup, not rely on an externally running agent.

## Assumptions

- Docker is available on the developer's machine (testcontainers requires a running Docker daemon).
- The mock backend Docker image is either pre-built before tests or the test setup includes a build step. Given testcontainers typically works with pre-built images, the mock backend image will be built as a prerequisite (e.g., via a build script or cargo build step) rather than building during each test run.
- The agent binary is built before tests run (standard Rust test workflow builds the project first).
- Tests will use randomized or dynamically assigned ports to avoid conflicts when running in parallel or alongside other services.
- The existing mock backend implementation (gRPC health service with configurable status) is sufficient for all new test scenarios.
- The `#[ignore]` attribute will be removed from migrated tests since they will be self-contained and no longer require external infrastructure.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: All integration tests pass when running `cargo test` with only Docker available — no manual docker-compose or infrastructure setup required.
- **SC-002**: All 6 existing integration test scenarios continue to pass after migration with equivalent assertions.
- **SC-003**: At least 3 new resilience test scenarios (disconnect detection, recovery after restart, status change after reload) pass consistently.
- **SC-004**: Container cleanup completes within 10 seconds of test completion, leaving no orphaned containers.
- **SC-005**: The full integration test suite completes within 2 minutes on a standard development machine.
- **SC-006**: Tests can run in CI environments without additional setup beyond having Docker available.
