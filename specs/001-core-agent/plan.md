# Implementation Plan: Core HAProxy gRPC Agent

**Branch**: `001-core-agent` | **Date**: 2025-10-28 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/001-core-agent/spec.md`

**Note**: This template is filled in by the `/speckit.plan` command. See `.specify/templates/commands/plan.md` for the execution workflow.

## Summary

Implement a lightweight TCP-based agent that responds to HAProxy health check requests according to the Agent Text Protocol. The agent receives plain text commands from HAProxy specifying backend servers to check, establishes gRPC connections to those backends to verify health, and responds with `up` or `down` status. The agent will support configuration via multiple methods (env vars, CLI flags, config file), emit structured JSON logs, and expose Prometheus metrics. The core technical approach emphasizes simplicity, reliability, and strict protocol compliance to ensure production-ready HAProxy integration.

## Technical Context

**Language/Version**: Rust 1.75+
**Primary Dependencies**: tokio (async runtime), tonic (gRPC client for backend checks), prometheus (metrics), serde (config/logging)
**Storage**: N/A (stateless agent, no persistence required)
**Testing**: cargo test (unit tests), docker-compose + integration tests (HAProxy 3.1+)
**Target Platform**: Linux server (x86_64, ARM64) - containerized deployment
**Project Type**: single (single binary agent)
**Performance Goals**: 1000+ concurrent health check requests without degradation, <2s response time (HAProxy default timeout), <100ms metrics endpoint response
**Constraints**: <50MB memory under normal load, <20MB binary size (statically linked), <5s startup time, must handle abrupt HAProxy disconnects gracefully
**Scale/Scope**: Agent Text Protocol server + gRPC health checker, ~2000-5000 LOC, 4 user stories (MVP + 3 enhancements)

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

### I. Agent Pattern ✅

- ✅ Single binary deployment: Agent will be compiled to single statically-linked executable
- ✅ Minimal dependencies: Using tokio (async), tonic (gRPC client), prometheus (metrics), serde (config)
- ✅ Simple configuration: Supporting env vars, CLI flags, and config file
- ✅ Single TCP endpoint: Agent exposes one TCP server for HAProxy Agent Text Protocol
- ✅ Clear HAProxy integration: All communication via Agent Text Protocol interface
- ✅ No orchestration requirement: Runs standalone, integrates cleanly with Docker/systemd

**Status**: PASS - Design fully aligns with Agent Pattern principle

### II. Integration-Heavy Testing ✅

- ✅ Integration tests required: Every feature includes HAProxy integration test
- ✅ Protocol flow verification: Tests will verify complete Agent Check Protocol (request → response → status codes)
- ✅ Contract tests: gRPC service definitions tested against HAProxy expectations
- ✅ Local runnability: `make test-integration` will spin up HAProxy + agent in containers
- ✅ Unit tests optional: Only using unit tests for complex logic (config parsing, health check logic)

**Status**: PASS - Testing strategy prioritizes real HAProxy integration

### III. Observability ✅

- ✅ Structured JSON logs: All operations emit JSON with timestamp, level, component, message, trace_id
- ✅ Prometheus metrics: `/metrics` endpoint exposes check_requests_total, check_duration_seconds, check_errors_total, grpc_connections_active
- ✅ Actionable errors: Error messages include context (what failed, why, what to check)
- ✅ HAProxy communication logging: All requests/responses/connection state logged with trace_id
- ✅ Configuration audit: Startup logs active configuration at INFO level

**Status**: PASS - Full observability built into design

### IV. HAProxy Protocol Compliance ✅

- ✅ Request format: Parse Agent Text Protocol format `<backend_server_name> <backend_server_port> <ssl_flag> <proxy_host_name>\n`
- ✅ Response format: Strict adherence to `up\n` or `down\n` format
- ✅ Timing compliance: All responses within 2s timeout (HAProxy default)
- ✅ Connection handling: Support for persistent connections, abrupt disconnects
- ✅ Protocol validation: Deviations logged with WARN, errors return `down`
- ✅ Multi-version testing: Tests against HAProxy 3.1, 3.2, latest stable

**Status**: PASS - Protocol compliance is core requirement

### V. Simplicity & Reliability ✅

- ✅ YAGNI: Only implementing documented use cases from spec (4 user stories)
- ✅ Fail-safe defaults: Agent reports `down` on internal errors (FR-009)
- ✅ Graceful degradation: Health checks continue even if metrics/logging fails (FR-010)
- ✅ Justified dependencies: Dependencies limited to gRPC runtime, Prometheus client, logging (all essential)
- ✅ Explicit error handling: No silent failures, all errors logged and surfaced (FR-012)

**Status**: PASS - Design prioritizes simplicity and reliability

### Gates Summary

| Principle | Status | Notes |
|-----------|--------|-------|
| Agent Pattern | ✅ PASS | Single binary, minimal deps, clear integration |
| Integration-Heavy Testing | ✅ PASS | Real HAProxy tests for every feature |
| Observability | ✅ PASS | JSON logs, Prometheus metrics, trace IDs |
| HAProxy Protocol Compliance | ✅ PASS | Strict protocol adherence, multi-version tests |
| Simplicity & Reliability | ✅ PASS | YAGNI, fail-safe, graceful degradation |

**Overall**: ✅ ALL GATES PASS - Proceed to Phase 0 research

## Project Structure

### Documentation (this feature)

```text
specs/[###-feature]/
├── plan.md              # This file (/speckit.plan command output)
├── research.md          # Phase 0 output (/speckit.plan command)
├── data-model.md        # Phase 1 output (/speckit.plan command)
├── quickstart.md        # Phase 1 output (/speckit.plan command)
├── contracts/           # Phase 1 output (/speckit.plan command)
└── tasks.md             # Phase 2 output (/speckit.tasks command - NOT created by /speckit.plan)
```

### Source Code (repository root)

```text
src/
├── main.rs               # Main entry point, CLI bootstrap
├── config.rs             # Configuration (env/CLI/file loading, validation)
├── server.rs             # TCP server for Agent Text Protocol
├── protocol.rs           # Agent Text Protocol parser/formatter
├── checker.rs            # gRPC health checker (connects to backends)
├── metrics.rs            # Prometheus metrics collection
└── logger.rs             # Structured JSON logging with trace IDs

tests/
├── integration/          # HAProxy integration tests (PRIMARY)
│   ├── docker-compose.yml  # HAProxy + agent + mock gRPC backend
│   ├── haproxy.cfg         # HAProxy test configuration
│   └── tests.rs            # Integration test suite
├── unit/                 # Unit tests (config, protocol parser)
│   ├── config_test.rs
│   └── protocol_test.rs
└── common/               # Shared test utilities
    └── mod.rs

deployments/
├── docker/
│   └── Dockerfile        # Multi-stage build for statically-linked binary
└── systemd/
    └── haproxy-grpc-agent.service  # systemd unit file

docs/
├── protocol-spec.md      # Agent Text Protocol documentation
├── config-reference.md   # Configuration options
└── deployment-guide.md   # Deployment instructions

Cargo.toml                # Rust dependencies and build config
Makefile                  # Build, test, development commands
```

**Structure Decision**: Single project Rust structure. Using standard Rust layout with `src/` for source code split into focused modules: `server` (TCP server), `protocol` (Agent Text Protocol parsing), `checker` (gRPC health checking), `metrics`, `logger`, and `config`. Tests organized integration-first per constitution. No protobuf generation needed (agent is gRPC client, not server).

## Complexity Tracking

> **Fill ONLY if Constitution Check has violations that must be justified**

No violations. All design decisions align with constitution principles.

---

## Post-Design Constitution Check

*Re-evaluation after Phase 1 design complete*

### Overall Assessment: ✅ ALL GATES PASS

After completing research, data model, and contracts, the design continues to fully comply with all constitution principles:

**I. Agent Pattern** ✅
- Single binary: Confirmed via Docker multi-stage build (musl static linking)
- Dependencies: tokio, tonic, prometheus, serde, clap, config - all justified and minimal
- Configuration: Implemented via clap (CLI), config crate (file), env vars
- Single endpoint: TCP server on port 5555 for Agent Text Protocol
- No orchestration dependency: Runs as standalone binary, Docker, or systemd

**II. Integration-Heavy Testing** ✅
- docker-compose test environment: HAProxy 3.1+ + agent + mock gRPC backend
- Integration tests cover: healthy backend → up, unhealthy → down, TLS handling, concurrent checks
- Contract tests: Agent Text Protocol parser, gRPC Health Check Protocol usage
- Local runnable: `make test-integration` confirmed in quickstart.md

**III. Observability** ✅
- Structured logging: tracing crate with JSON formatter, all fields defined in data-model.md
- Metrics: Prometheus endpoint on :9090 with all required metrics (requests, duration, errors, gauges)
- Actionable errors: Error handling strategy documents clear error messages
- HAProxy communication logging: Every request/response logged with trace_id
- Configuration audit: Startup logging confirmed in research.md

**IV. HAProxy Protocol Compliance** ✅
- Protocol specification: Fully documented in contracts/agent-text-protocol.md
- Request format: Parser handles `<server> <port> <ssl_flag> <host>\n`
- Response format: `up\n` or `down\n` only (strict adherence)
- Timing: <2s total (1s connect + 1.5s RPC = 2.5s, configurable to fit under 2s)
- Connection handling: Persistent connections, graceful close handling
- Multi-version testing: HAProxy 3.1, 3.2, latest stable in integration tests

**V. Simplicity & Reliability** ✅
- YAGNI: Only 4 user stories implemented (MVP scope)
- Fail-safe defaults: All error paths return `down` (documented in contracts/agent-text-protocol.md)
- Graceful degradation: Health checks continue even if metrics/logging fail
- Justified dependencies: All 6 dependencies have documented rationale in research.md
- Explicit error handling: Error handling strategy section in research.md, no silent failures

### Changes Since Initial Check

- **Clarified protocol**: Agent Text Protocol (not gRPC Agent Check Protocol)
- **Architecture**: TCP server + gRPC client (not gRPC server)
- **Testing strategy**: docker-compose integration tests finalized
- **Dependency list**: Finalized to tokio, tonic, prometheus, serde, clap, config

### Conclusion

Design is ready for implementation. No complexity violations to track. Proceed to `/speckit.tasks` command to generate tasks.md.
