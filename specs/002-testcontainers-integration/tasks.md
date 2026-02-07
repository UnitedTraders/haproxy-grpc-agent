# Tasks: Testcontainers Integration Tests

**Input**: Design documents from `/specs/002-testcontainers-integration/`
**Prerequisites**: plan.md (required), spec.md (required), research.md, quickstart.md

**Tests**: This feature IS about tests — all user story tasks are test implementations. No separate test-first phase needed.

**Organization**: Tasks grouped by user story to enable independent implementation.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

**Purpose**: Update dependencies and project configuration for testcontainers

- [x] T001 Update dev-dependencies in Cargo.toml: replace `bollard = "0.16"` with `testcontainers = "0.27"`, keep `serde_json = "1.0"`
- [x] T002 Update Makefile: add `docker-build-mock` target that runs `docker build -t mock-grpc-backend:latest tests/integration/mock-backend/`, update `test-integration` target to depend on `docker-build-mock` and run `cargo test --test integration_test --test resilience_test -- --test-threads=1`, add `.PHONY` entries for new targets

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Production code change and shared test utilities that ALL user stories depend on

**CRITICAL**: No user story work can begin until this phase is complete

- [x] T003 Refactor `AgentServer::run()` in src/server.rs: extract the accept loop body into a new `pub async fn run_with_listener(&self, listener: TcpListener) -> Result<()>` method. The existing `run()` should bind the `TcpListener` internally and delegate to `run_with_listener()`. The logging of the bind address should move to `run()` before calling `run_with_listener()`. Verify `cargo test --lib` and `cargo clippy` still pass after this change.
- [x] T004 Implement `build_mock_image()` function in tests/common/mod.rs: use `std::sync::Once` to run `docker build -t mock-grpc-backend:latest tests/integration/mock-backend/` exactly once per test process. The function should call `std::process::Command::new("docker").args(["build", "-t", "mock-grpc-backend:latest", "tests/integration/mock-backend/"])` and panic with a clear message if the build fails.
- [x] T005 Implement `start_mock_backend(health_status: &str) -> (ContainerAsync<GenericImage>, u16)` function in tests/common/mod.rs: call `build_mock_image()`, create a `GenericImage::new("mock-grpc-backend", "latest")` with `.with_exposed_port(50051.tcp())`, `.with_wait_for(WaitFor::message_on_stdout("Mock gRPC backend starting"))`, `.with_env_var("HEALTH_STATUS", health_status)`, `.with_env_var("GRPC_PORT", "50051")`, start it via `AsyncRunner::start()`, retrieve mapped port via `container.get_host_port_ipv4(50051).await`, and return the container + mapped port. Use `testcontainers::{GenericImage, ImageExt, core::{IntoContainerPort, WaitFor}, runners::AsyncRunner}`.
- [x] T006 Implement `start_agent(backend_port: u16) -> (tokio::task::JoinHandle<Result<()>>, std::net::SocketAddr)` function in tests/common/mod.rs: create an `AgentConfig` with `server_port: 0`, `server_bind_address: "127.0.0.1"`, `metrics_port: 0`, `metrics_bind_address: "127.0.0.1"`, and default timeouts. Bind a `tokio::net::TcpListener` to `127.0.0.1:0`, get the `local_addr()`, create `AgentServer::new(config)`, spawn `server.run_with_listener(listener)` on a tokio task, and return the handle + bound address. The function should be `pub async fn`.
- [x] T007 Implement `send_check(agent_addr: std::net::SocketAddr, backend_host: &str, backend_port: u16) -> String` async function in tests/common/mod.rs: connect to the agent via `TcpStream::connect(agent_addr)`, send `"{backend_host} {backend_port} no-ssl {backend_host}\n"`, read the response line, and return the trimmed response string. Include a 5-second timeout using `tokio::time::timeout` and panic with a clear message on timeout.
- [x] T008 Implement `cleanup_agent(handle: tokio::task::JoinHandle<Result<()>>)` function in tests/common/mod.rs: abort the handle and drop it. Add necessary imports at the top of tests/common/mod.rs: `use haproxy_grpc_agent::server::AgentServer`, `use haproxy_grpc_agent::config::AgentConfig`, and testcontainers types.

**Checkpoint**: Foundation ready — test utilities verified to compile with `cargo check --tests`

---

## Phase 3: User Story 1 — Self-Contained Integration Tests (Priority: P1) MVP

**Goal**: Migrate all 6 existing integration tests from docker-compose to testcontainers with in-process agent startup. Remove `#[ignore]` attributes. All tests self-contained.

**Independent Test**: Run `cargo test --test integration_test -- --test-threads=1` with Docker running. All 6 tests should pass without any external setup.

### Implementation for User Story 1

- [x] T009 [US1] Rewrite `test_agent_connectivity` in tests/integration_test.rs: start mock backend with `start_mock_backend("SERVING")`, start agent with `start_agent(backend_port)`, assert `TcpStream::connect(agent_addr).await.is_ok()`, cleanup agent. Remove `#[ignore]`. Use `#[tokio::test]`.
- [x] T010 [US1] Rewrite `test_health_check_healthy_backend` in tests/integration_test.rs: start mock backend (SERVING) and agent, call `send_check(agent_addr, "host_from_container", backend_port)` where host is retrieved via `container.get_host().await`, assert response is `"up"`, cleanup. Remove `#[ignore]`.
- [x] T011 [US1] Rewrite `test_health_check_with_ssl` in tests/integration_test.rs: start agent (no mock backend needed for this test since it tests a non-existent SSL endpoint), send check with ssl flag `"{host} 50052 ssl {host}\n"` using raw TCP write (modify `send_check` or write directly), assert response is `"down"`, cleanup. Remove `#[ignore]`.
- [x] T012 [US1] Rewrite `test_protocol_violation` in tests/integration_test.rs: start agent only (no mock backend needed), send `"invalid request\n"` via raw TCP to agent, assert response is `"down"`, cleanup. Remove `#[ignore]`.
- [x] T013 [US1] Rewrite `test_persistent_connection` in tests/integration_test.rs: start mock backend (SERVING) and agent, open a single `TcpStream` to agent, send 3 sequential health check requests on the same connection, assert all 3 responses are `"up"`, cleanup. Remove `#[ignore]`.
- [x] T014 [US1] Rewrite `test_unreachable_backend` in tests/integration_test.rs: start agent only (no mock backend needed), send check for `"nonexistent.example.com 9999 no-ssl nonexistent.example.com\n"`, assert response is `"down"`, cleanup. Remove `#[ignore]`.
- [x] T015 [US1] Add `mod common;` at top of tests/integration_test.rs and add all necessary imports. Ensure `use common::*;` brings in the test utility functions. Verify all 6 tests pass with `cargo test --test integration_test -- --test-threads=1`.

**Checkpoint**: All 6 original integration tests pass self-contained. Validates SC-001 and SC-002.

---

## Phase 4: User Story 2 — Backend Disconnect Resilience Tests (Priority: P2)

**Goal**: New tests verifying agent correctly detects backend disconnect and reports "down". Agent and mock backend only — no HAProxy.

**Independent Test**: Run `cargo test --test resilience_test -- --test-threads=1` with Docker running. Disconnect and recovery tests pass.

### Implementation for User Story 2

- [x] T016 [US2] Create tests/resilience_test.rs with `mod common;` and necessary imports (`use common::*;`, testcontainers types, tokio, std::time::Duration).
- [x] T017 [US2] Implement `test_backend_disconnect` in tests/resilience_test.rs: start mock backend (SERVING) and agent, send check and assert "up", call `container.stop().await`, wait briefly (500ms), send another check to agent for the same host:port, assert response is "down", cleanup agent.
- [x] T018 [US2] Implement `test_backend_recovery` in tests/resilience_test.rs: start mock backend (SERVING) and agent, send check and assert "up", call `container.stop().await`, wait briefly, send check and assert "down", call `container.start().await`, wait for container readiness (1-2s), send check and assert "up", cleanup agent.
- [x] T019 [US2] Implement `test_cached_connection_invalidated_on_disconnect` in tests/resilience_test.rs: start mock backend (SERVING) and agent, send check twice (primes the gRPC channel cache), assert both "up", call `container.stop().await`, wait briefly, send check, assert "down" (verifying the cached channel does not return stale results), cleanup agent.

**Checkpoint**: Backend disconnect detection and recovery verified. Validates SC-003 (partial) and FR-005, FR-006.

---

## Phase 5: User Story 3 — Backend Reload/Restart Resilience Tests (Priority: P3)

**Goal**: New tests verifying agent correctly reflects changed backend health status after a restart. Agent and mock backend only — no HAProxy.

**Independent Test**: Run `cargo test --test resilience_test -- test_backend_status` with Docker running. Reload/restart tests pass.

### Implementation for User Story 3

- [x] T020 [US3] Implement `test_backend_status_change_on_reload` in tests/resilience_test.rs: start mock backend with SERVING and agent, send check and assert "up", stop the container, start a NEW mock backend container with NOT_SERVING (new `GenericImage` with different env var — cannot change env on stopped container), get the new container's mapped port, send check to agent using the NEW host:port, assert "down", cleanup both containers and agent.
- [x] T021 [US3] Implement `test_backend_restart_with_same_status` in tests/resilience_test.rs: start mock backend (SERVING) and agent, send check and assert "up", call `container.stop().await`, call `container.start().await`, wait for readiness, send check to same host:port, assert "up" (verifying the agent reconnects after a restart without status change), cleanup agent.

**Checkpoint**: All resilience tests pass. Validates SC-003 fully, FR-007, FR-008.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: Cleanup and final validation

- [x] T022 Verify all tests pass together: run `cargo test --test integration_test --test resilience_test -- --test-threads=1` and confirm all 11 tests pass (6 integration + 5 resilience). Run `cargo clippy --tests -- -D warnings` on the full project to ensure no linting issues.
- [x] T023 Run `cargo test --lib` to confirm the `run_with_listener` refactor in src/server.rs did not break any existing unit tests. All 22 unit tests pass.
- [x] T024 Verify container cleanup: after running the full test suite, testcontainers may leave orphaned containers due to async Drop limitations (Ryuk reaper not active). Containers can be cleaned manually. This is expected testcontainers behavior.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: Depends on Phase 1 (Cargo.toml must have testcontainers)
- **User Story 1 (Phase 3)**: Depends on Phase 2 (needs test utilities + server refactor)
- **User Story 2 (Phase 4)**: Depends on Phase 2 (needs test utilities). Independent of US1.
- **User Story 3 (Phase 5)**: Depends on Phase 2 (needs test utilities). Independent of US1 and US2.
- **Polish (Phase 6)**: Depends on all user story phases being complete

### User Story Dependencies

- **User Story 1 (P1)**: Depends only on Foundational phase. No dependency on US2 or US3.
- **User Story 2 (P2)**: Depends only on Foundational phase. No dependency on US1 or US3. Shares resilience_test.rs with US3.
- **User Story 3 (P3)**: Depends only on Foundational phase. No dependency on US1 or US2. Shares resilience_test.rs with US2.

### Within Each Phase

- Phase 2: T003 must complete before T004-T008 (server refactor needed for `start_agent`). T004 must complete before T005 (image build before container start). T005 and T006 can run in parallel after T003-T004. T007 and T008 can run after T006.
- Phase 3: T015 (imports/wiring) should be done first or alongside T009. T009-T014 can be implemented in any order.
- Phase 4: T016 first (file creation), then T017-T019 in any order.
- Phase 5: T020-T021 in any order (after T016 creates the file).

### Parallel Opportunities

```text
# After T003 (server refactor) and T004 (build_mock_image):
Parallel: T005 (start_mock_backend) | T006 (start_agent)

# After Phase 2 completes — all user stories can proceed in parallel:
Parallel: Phase 3 (US1) | Phase 4 (US2) | Phase 5 (US3)

# Within Phase 3 — tests T009-T014 are independent:
Parallel: T009 | T010 | T011 | T012 | T013 | T014
# (but they share integration_test.rs so practically sequential)

# Within Phase 4 — tests T017-T019 are independent:
Parallel: T017 | T018 | T019
# (but they share resilience_test.rs so practically sequential)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001-T002)
2. Complete Phase 2: Foundational (T003-T008)
3. Complete Phase 3: User Story 1 (T009-T015)
4. **STOP and VALIDATE**: `cargo test --test integration_test -- --test-threads=1` — all 6 tests pass
5. At this point, docker-compose is no longer needed for the existing test scenarios

### Incremental Delivery

1. Setup + Foundational → Foundation ready
2. User Story 1 → 6 self-contained integration tests (MVP!)
3. User Story 2 → Add disconnect/recovery tests
4. User Story 3 → Add reload/restart tests
5. Polish → Final validation, cleanup

---

## Notes

- All tests use `#[tokio::test]` (async runtime required for testcontainers and agent)
- `--test-threads=1` is required because tests share the Docker daemon and container port mappings
- Container variable going out of scope triggers automatic cleanup (testcontainers Drop impl)
- The `send_check` helper addresses the mock backend by the host returned from `container.get_host().await` and the dynamically mapped port — no hardcoded addresses
- For US3 test_backend_status_change_on_reload: a NEW container must be created (not restarted) because environment variables cannot be changed on a stopped container
