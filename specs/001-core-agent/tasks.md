# Tasks: Core HAProxy gRPC Agent

**Input**: Design documents from `/specs/001-core-agent/`
**Prerequisites**: plan.md (required), spec.md (required), data-model.md, contracts/, research.md

**Tests**: Integration tests are REQUIRED per constitution (Integration-Heavy Testing principle). Unit tests are optional for complex logic only.

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

- **Single project**: `src/`, `tests/` at repository root
- All paths assume Rust single project structure

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: Project initialization and basic structure

- [ ] T001 Create Rust project with `cargo init --name haproxy-grpc-agent`
- [ ] T002 [P] Add tokio dependency to Cargo.toml with features full, macros
- [ ] T003 [P] Add tonic dependency to Cargo.toml for gRPC client
- [ ] T004 [P] Add prometheus dependency to Cargo.toml for metrics
- [ ] T005 [P] Add serde dependency to Cargo.toml with derive feature
- [ ] T006 [P] Add clap dependency to Cargo.toml with derive feature for CLI parsing
- [ ] T007 [P] Add config dependency to Cargo.toml for configuration management
- [ ] T008 [P] Add tracing and tracing-subscriber dependencies to Cargo.toml for structured logging
- [ ] T009 [P] Add dashmap dependency to Cargo.toml for concurrent channel cache
- [ ] T010 [P] Add once_cell dependency to Cargo.toml for lazy static metrics
- [ ] T011 Configure Cargo.toml release profile with lto=true, codegen-units=1, strip=true, opt-level="z"
- [ ] T012 Create src/main.rs with basic tokio main function skeleton
- [ ] T013 [P] Create src/config.rs module file with module declaration
- [ ] T014 [P] Create src/server.rs module file with module declaration
- [ ] T015 [P] Create src/protocol.rs module file with module declaration
- [ ] T016 [P] Create src/checker.rs module file with module declaration
- [ ] T017 [P] Create src/metrics.rs module file with module declaration
- [ ] T018 [P] Create src/logger.rs module file with module declaration
- [ ] T019 Create tests/integration/ directory structure
- [ ] T020 Create tests/unit/ directory structure
- [ ] T021 Create tests/common/mod.rs for shared test utilities
- [ ] T022 [P] Create deployments/docker/Dockerfile multi-stage build skeleton
- [ ] T023 [P] Create deployments/systemd/haproxy-grpc-agent.service file
- [ ] T024 [P] Create Makefile with build, test, test-integration targets
- [ ] T025 Create .gitignore for Rust project (target/, Cargo.lock for libraries)

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Core infrastructure that MUST be complete before ANY user story can be implemented

**⚠️ CRITICAL**: No user story work can begin until this phase is complete

- [ ] T026 Implement AgentConfig struct in src/config.rs with all fields from data-model.md
- [ ] T027 Implement default functions for AgentConfig in src/config.rs
- [ ] T028 Implement LogLevel and LogFormat enums in src/config.rs with serde derives
- [ ] T029 Implement config validation function in src/config.rs checking port ranges and timeout constraints
- [ ] T030 Implement config loading from environment variables in src/config.rs
- [ ] T031 Implement config loading from CLI flags using clap in src/config.rs
- [ ] T032 Implement config loading from TOML file using config crate in src/config.rs
- [ ] T033 Implement configuration precedence logic (CLI > file > env > defaults) in src/config.rs
- [ ] T034 Implement fail-fast validation on startup with clear error messages in src/config.rs
- [ ] T035 Implement HealthCheckRequest struct in src/protocol.rs with all fields from data-model.md
- [ ] T036 Implement SslFlag enum in src/protocol.rs
- [ ] T037 Implement HealthCheckResponse struct in src/protocol.rs
- [ ] T038 Implement HealthStatus enum in src/protocol.rs with to_protocol_string method
- [ ] T039 Implement parse_request function in src/protocol.rs parsing Agent Text Protocol format
- [ ] T040 Implement protocol validation in src/protocol.rs for field count, port range, ssl_flag values
- [ ] T041 Implement error types for protocol parsing in src/protocol.rs (ParseError enum)
- [ ] T042 [P] Unit test for parse_request with valid input in tests/unit/protocol_test.rs
- [ ] T043 [P] Unit test for parse_request with invalid field count in tests/unit/protocol_test.rs
- [ ] T044 [P] Unit test for parse_request with invalid port in tests/unit/protocol_test.rs
- [ ] T045 [P] Unit test for parse_request with invalid ssl_flag in tests/unit/protocol_test.rs
- [ ] T046 [P] Unit test for config validation with valid config in tests/unit/config_test.rs
- [ ] T047 [P] Unit test for config validation with port conflict in tests/unit/config_test.rs
- [ ] T048 [P] Unit test for config validation with invalid timeout in tests/unit/config_test.rs

**Checkpoint**: Foundation ready - user story implementation can now begin in parallel

---

## Phase 3: User Story 1 - Basic Health Check Response (Priority: P1) 🎯 MVP

**Goal**: TCP server accepts HAProxy connections, parses Agent Text Protocol requests, performs gRPC health checks to backends, responds with up/down status

**Independent Test**: Deploy HAProxy + agent + mock gRPC backend via docker-compose, send health check request, verify up/down response within 2s

### Integration Tests for User Story 1 (REQUIRED per constitution)

> **NOTE: Write these tests FIRST, ensure they FAIL before implementation**

- [ ] T049 [P] [US1] Create docker-compose.yml in tests/integration/ with services: agent, haproxy, mock-grpc-backend
- [ ] T050 [P] [US1] Create haproxy.cfg in tests/integration/ with agent-check configuration
- [ ] T051 [P] [US1] Create mock gRPC backend implementation in tests/integration/ using tonic server with Health service
- [ ] T052 [P] [US1] Integration test: HAProxy sends check, backend healthy (SERVING) → agent responds "up\n" in tests/integration/tests.rs
- [ ] T053 [P] [US1] Integration test: HAProxy sends check, backend unhealthy (NOT_SERVING) → agent responds "down\n" in tests/integration/tests.rs
- [ ] T054 [P] [US1] Integration test: HAProxy sends check, backend unreachable → agent responds "down\n" in tests/integration/tests.rs
- [ ] T055 [P] [US1] Integration test: Multiple rapid checks → all correct responses in tests/integration/tests.rs
- [ ] T056 [P] [US1] Integration test: Response time <2s for health checks in tests/integration/tests.rs

### Implementation for User Story 1

- [ ] T057 [P] [US1] Implement BackendChannelKey struct in src/checker.rs with Hash trait
- [ ] T058 [P] [US1] Implement From<&HealthCheckRequest> for BackendChannelKey in src/checker.rs
- [ ] T059 [US1] Implement ChannelCache using DashMap<BackendChannelKey, Channel> in src/checker.rs
- [ ] T060 [US1] Implement get_or_create_channel function in src/checker.rs with TLS configuration based on ssl_flag
- [ ] T061 [US1] Implement gRPC channel creation with connect timeout (1000ms) in src/checker.rs
- [ ] T062 [US1] Implement gRPC Health Check client using tonic in src/checker.rs
- [ ] T063 [US1] Implement check_backend function in src/checker.rs calling grpc.health.v1.Health/Check
- [ ] T064 [US1] Implement RPC timeout (1500ms) for health check call in src/checker.rs
- [ ] T065 [US1] Implement ServingStatus mapping (SERVING→Up, others→Down) in src/checker.rs
- [ ] T066 [US1] Implement error handling (unreachable, timeout → Down) in src/checker.rs
- [ ] T067 [US1] Implement TCP server using tokio::net::TcpListener in src/server.rs
- [ ] T068 [US1] Implement server bind to configured address and port in src/server.rs
- [ ] T069 [US1] Implement connection accept loop in src/server.rs spawning tasks per connection
- [ ] T070 [US1] Implement handle_connection function in src/server.rs reading line from stream
- [ ] T071 [US1] Implement persistent connection handling (loop over requests) in src/server.rs
- [ ] T072 [US1] Implement graceful connection close detection (EOF) in src/server.rs
- [ ] T073 [US1] Implement abrupt disconnect handling in src/server.rs
- [ ] T074 [US1] Integrate protocol::parse_request in handle_connection in src/server.rs
- [ ] T075 [US1] Integrate checker::check_backend in handle_connection in src/server.rs
- [ ] T076 [US1] Write response (up\n or down\n) to TCP stream in src/server.rs
- [ ] T077 [US1] Update main.rs to load config and start TCP server

**Checkpoint**: At this point, User Story 1 should be fully functional and testable independently via `make test-integration`

---

## Phase 4: User Story 2 - Agent Configuration (Priority: P2)

**Goal**: Agent accepts configuration from env vars, CLI flags, TOML file with proper precedence and validation

**Independent Test**: Start agent with different config methods, verify settings applied and logged at startup

### Integration Tests for User Story 2 (REQUIRED per constitution)

- [ ] T078 [P] [US2] Integration test: Start agent with env vars → config applied in tests/integration/tests.rs
- [ ] T079 [P] [US2] Integration test: Start agent with CLI flags → overrides env vars in tests/integration/tests.rs
- [ ] T080 [P] [US2] Integration test: Start agent with config file → loads from file in tests/integration/tests.rs
- [ ] T081 [P] [US2] Integration test: Invalid config → agent exits with error in tests/integration/tests.rs
- [ ] T082 [P] [US2] Integration test: No config → uses defaults in tests/integration/tests.rs

### Implementation for User Story 2

- [ ] T083 [US2] Implement CLI argument parsing using clap derive in src/main.rs
- [ ] T084 [US2] Add --config flag for TOML file path in src/main.rs
- [ ] T085 [US2] Add --server-port, --metrics-port CLI flags in src/main.rs
- [ ] T086 [US2] Add --log-level, --log-format CLI flags in src/main.rs
- [ ] T087 [US2] Add --grpc-connect-timeout, --grpc-rpc-timeout CLI flags in src/main.rs
- [ ] T088 [US2] Implement config merge logic in src/config.rs combining all sources
- [ ] T089 [US2] Log active configuration at INFO level on startup in src/main.rs
- [ ] T090 [US2] Create example config.toml in repository root with all options documented

**Checkpoint**: At this point, User Stories 1 AND 2 should both work independently

---

## Phase 5: User Story 3 - Structured Logging (Priority: P3)

**Goal**: Agent emits structured JSON logs with trace IDs, timestamps, levels, component names for all operations

**Independent Test**: Run agent, trigger operations (startup, health checks, errors), verify JSON logs with correct fields

### Integration Tests for User Story 3 (REQUIRED per constitution)

- [ ] T091 [P] [US3] Integration test: Startup logs configuration at INFO level in JSON format in tests/integration/tests.rs
- [ ] T092 [P] [US3] Integration test: Health check request logged with trace_id in tests/integration/tests.rs
- [ ] T093 [P] [US3] Integration test: Error logged with actionable context in tests/integration/tests.rs
- [ ] T094 [P] [US3] Integration test: DEBUG log level → detailed traces in tests/integration/tests.rs
- [ ] T095 [P] [US3] Integration test: ERROR log level → only errors logged in tests/integration/tests.rs

### Implementation for User Story 3

- [ ] T096 [US3] Implement logger initialization using tracing-subscriber in src/logger.rs
- [ ] T097 [US3] Configure JSON formatter for structured logging in src/logger.rs
- [ ] T098 [US3] Configure log level from AgentConfig in src/logger.rs
- [ ] T099 [US3] Support both JSON and pretty log formats in src/logger.rs
- [ ] T100 [US3] Add tracing spans with trace_id generation in src/server.rs handle_connection
- [ ] T101 [US3] Log startup with configuration details at INFO level in src/main.rs
- [ ] T102 [US3] Log HAProxy connection established at INFO level in src/server.rs
- [ ] T103 [US3] Log health check request/response with trace_id at INFO level in src/server.rs
- [ ] T104 [US3] Log protocol violations at WARN level in src/protocol.rs
- [ ] T105 [US3] Log backend unreachable at ERROR level in src/checker.rs
- [ ] T106 [US3] Log timeout errors at ERROR level in src/checker.rs
- [ ] T107 [US3] Log connection closed/reset at INFO/WARN level in src/server.rs
- [ ] T108 [US3] Initialize logger in main.rs before starting server

**Checkpoint**: All user stories 1, 2, and 3 should now be independently functional

---

## Phase 6: User Story 4 - Prometheus Metrics (Priority: P4)

**Goal**: Agent exposes /metrics HTTP endpoint with Prometheus-formatted counters, histograms, gauges

**Independent Test**: Query /metrics endpoint, verify Prometheus format and all required metrics present

### Integration Tests for User Story 4 (REQUIRED per constitution)

- [ ] T109 [P] [US4] Integration test: /metrics endpoint returns Prometheus format in tests/integration/tests.rs
- [ ] T110 [P] [US4] Integration test: check_requests_total counter increments in tests/integration/tests.rs
- [ ] T111 [P] [US4] Integration test: check_duration_seconds histogram populated in tests/integration/tests.rs
- [ ] T112 [P] [US4] Integration test: check_errors_total counter increments on errors in tests/integration/tests.rs
- [ ] T113 [P] [US4] Integration test: haproxy_connections_active gauge reflects connections in tests/integration/tests.rs

### Implementation for User Story 4

- [ ] T114 [P] [US4] Define CHECK_REQUESTS_TOTAL counter with result label in src/metrics.rs
- [ ] T115 [P] [US4] Define CHECK_ERRORS_TOTAL counter with error_type label in src/metrics.rs
- [ ] T116 [P] [US4] Define CHECK_DURATION_SECONDS histogram with buckets 0.01, 0.05, 0.1, 0.5, 1.0, 2.0 in src/metrics.rs
- [ ] T117 [P] [US4] Define HAPROXY_CONNECTIONS_ACTIVE gauge in src/metrics.rs
- [ ] T118 [P] [US4] Define GRPC_CHANNELS_ACTIVE gauge in src/metrics.rs
- [ ] T119 [US4] Register all metrics using lazy_static or once_cell in src/metrics.rs
- [ ] T120 [US4] Implement HTTP server for /metrics endpoint using hyper in src/metrics.rs
- [ ] T121 [US4] Implement metrics handler returning Prometheus text format in src/metrics.rs
- [ ] T122 [US4] Bind metrics server to configured metrics_port in src/metrics.rs
- [ ] T123 [US4] Increment CHECK_REQUESTS_TOTAL in src/server.rs after each check
- [ ] T124 [US4] Observe CHECK_DURATION_SECONDS in src/server.rs timing health checks
- [ ] T125 [US4] Increment CHECK_ERRORS_TOTAL with error_type labels in src/checker.rs
- [ ] T126 [US4] Increment/decrement HAPROXY_CONNECTIONS_ACTIVE in src/server.rs on connect/disconnect
- [ ] T127 [US4] Update GRPC_CHANNELS_ACTIVE when channels added to cache in src/checker.rs
- [ ] T128 [US4] Start metrics server in main.rs concurrently with TCP server
- [ ] T129 [US4] Implement graceful degradation (metrics failure doesn't stop health checks) in src/main.rs

**Checkpoint**: All user stories should now be independently functional

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: Improvements that affect multiple user stories

- [ ] T130 [P] Create docs/protocol-spec.md documenting Agent Text Protocol
- [ ] T131 [P] Create docs/config-reference.md with all configuration options
- [ ] T132 [P] Create docs/deployment-guide.md with Docker and systemd instructions
- [ ] T133 Complete Dockerfile multi-stage build for musl static linking in deployments/docker/Dockerfile
- [ ] T134 Add Docker build and run targets to Makefile
- [ ] T135 Complete systemd service file with restart policies in deployments/systemd/haproxy-grpc-agent.service
- [ ] T136 Create README.md in repository root with quickstart, features, architecture
- [ ] T137 [P] Add error handling for port-already-in-use in src/server.rs and src/metrics.rs
- [ ] T138 Implement graceful shutdown handling (SIGTERM) in src/main.rs
- [ ] T139 Implement graceful shutdown for in-flight requests in src/server.rs
- [ ] T140 Add cargo clippy fixes for all warnings
- [ ] T141 Add cargo fmt check to Makefile
- [ ] T142 Run integration test suite against HAProxy 3.1 in CI configuration
- [ ] T143 Run integration test suite against HAProxy 3.2 in CI configuration
- [ ] T144 Run integration test suite against HAProxy latest in CI configuration
- [ ] T145 Verify binary size <20MB after release build
- [ ] T146 Verify memory usage <50MB under 100 req/s load test
- [ ] T147 Verify startup time <5s measurement

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies - can start immediately
- **Foundational (Phase 2)**: Depends on Setup completion - BLOCKS all user stories
- **User Stories (Phase 3-6)**: All depend on Foundational phase completion
  - User stories can then proceed in parallel (if staffed)
  - Or sequentially in priority order (US1 → US2 → US3 → US4)
- **Polish (Phase 7)**: Depends on all user stories being complete

### User Story Dependencies

- **User Story 1 (P1)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 2 (P2)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 3 (P3)**: Can start after Foundational (Phase 2) - No dependencies on other stories
- **User Story 4 (P4)**: Can start after Foundational (Phase 2) - No dependencies on other stories

### Within Each User Story

- Integration tests (if included) MUST be written and FAIL before implementation
- Protocol/config foundation before server implementation
- gRPC checker before server integration
- Core implementation before error handling refinements
- Story complete before moving to next priority

### Parallel Opportunities

- All Setup tasks marked [P] can run in parallel
- All Foundational tasks marked [P] (unit tests) can run in parallel after foundation code complete
- Once Foundational phase completes, all user stories can start in parallel (if team capacity allows)
- All integration tests for a user story marked [P] can run in parallel
- Implementation tasks within a story marked [P] can run in parallel
- Different user stories can be worked on in parallel by different team members

---

## Parallel Example: User Story 1

```bash
# Launch all integration test setup together:
Task: "Create docker-compose.yml in tests/integration/"
Task: "Create haproxy.cfg in tests/integration/"
Task: "Create mock gRPC backend in tests/integration/"

# Then launch all integration tests together (after setup complete):
Task: "Integration test: healthy backend → up"
Task: "Integration test: unhealthy backend → down"
Task: "Integration test: backend unreachable → down"
Task: "Integration test: multiple rapid checks"
Task: "Integration test: response time <2s"

# Launch parallel implementation tasks:
Task: "Implement BackendChannelKey struct in src/checker.rs"
Task: "Implement From<&HealthCheckRequest> for BackendChannelKey in src/checker.rs"
# (Then non-parallel tasks that depend on these)
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL - blocks all stories)
3. Complete Phase 3: User Story 1
4. **STOP and VALIDATE**: Test User Story 1 independently via `make test-integration`
5. Deploy/demo if ready

**MVP Scope**: 77 tasks (T001-T077)

### Incremental Delivery

1. Complete Setup + Foundational → Foundation ready (T001-T048)
2. Add User Story 1 → Test independently → Deploy/Demo (MVP!) (T049-T077)
3. Add User Story 2 → Test independently → Deploy/Demo (T078-T090)
4. Add User Story 3 → Test independently → Deploy/Demo (T091-T108)
5. Add User Story 4 → Test independently → Deploy/Demo (T109-T129)
6. Polish → Final release (T130-T147)
7. Each story adds value without breaking previous stories

### Parallel Team Strategy

With multiple developers:

1. Team completes Setup + Foundational together (T001-T048)
2. Once Foundational is done:
   - Developer A: User Story 1 (T049-T077)
   - Developer B: User Story 2 (T078-T090)
   - Developer C: User Story 3 (T091-T108)
   - Developer D: User Story 4 (T109-T129)
3. Stories complete and integrate independently
4. Team collaborates on Polish phase (T130-T147)

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label (US1, US2, US3, US4) maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Integration tests are REQUIRED per constitution (Integration-Heavy Testing principle)
- Unit tests are optional, only for complex logic (config parsing, protocol parsing)
- Verify tests fail before implementing
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- Avoid: vague tasks, same file conflicts, cross-story dependencies that break independence

---

## Constitution Compliance

This task breakdown ensures compliance with all constitution principles:

- **Agent Pattern**: Tasks build single binary (T133 Docker), minimal deps (T002-T010), simple config (T026-T034, T083-T090)
- **Integration-Heavy Testing**: Integration tests for every user story (T049-T056, T078-T082, T091-T095, T109-T113), docker-compose setup
- **Observability**: Structured logging tasks (T096-T108), Prometheus metrics tasks (T114-T129)
- **HAProxy Protocol Compliance**: Protocol parsing (T035-T041), contract tests, multi-version testing (T142-T144)
- **Simplicity & Reliability**: YAGNI (only 4 stories), fail-safe defaults (T066), graceful degradation (T129, T139)

---

## Task Count Summary

- **Total Tasks**: 147
- **Setup (Phase 1)**: 25 tasks
- **Foundational (Phase 2)**: 23 tasks
- **User Story 1 (Phase 3)**: 29 tasks (8 integration tests + 21 implementation)
- **User Story 2 (Phase 4)**: 13 tasks (5 integration tests + 8 implementation)
- **User Story 3 (Phase 5)**: 18 tasks (5 integration tests + 13 implementation)
- **User Story 4 (Phase 6)**: 21 tasks (5 integration tests + 16 implementation)
- **Polish (Phase 7)**: 18 tasks

**MVP Scope** (just US1): 77 tasks
**Parallel Tasks**: 58 tasks marked [P] can run in parallel within their phase
**Independent Stories**: All 4 user stories are independently testable after Foundational phase
