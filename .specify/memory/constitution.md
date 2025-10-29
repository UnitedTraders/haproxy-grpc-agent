<!--
SYNC IMPACT REPORT
==================
Version Change: [No previous version] → 1.0.0
Created: 2025-10-28

Principles Defined:
  1. Agent Pattern - Lightweight, minimal dependencies, clear HAProxy integration
  2. Integration-Heavy Testing - Focus on real HAProxy integration scenarios
  3. Observability - Structured logging, metrics exposure, debuggability
  4. HAProxy Protocol Compliance - Strict adherence to Agent Check Protocol
  5. Simplicity & Reliability - YAGNI, graceful degradation, fail-safe defaults

Additional Sections:
  - Operational Requirements - Production readiness, deployment, monitoring
  - Development Workflow - PR process, testing gates, review criteria

Templates Status:
  ✅ plan-template.md - Constitution Check section references this file
  ✅ spec-template.md - Aligns with user story and requirements structure
  ✅ tasks-template.md - Task categorization supports integration testing focus

Follow-up TODOs:
  - None - all sections completed
-->

# HAProxy gRPC Agent Constitution

## Core Principles

### I. Agent Pattern

The HAProxy gRPC Agent MUST be a lightweight, focused component with minimal dependencies and clear integration boundaries with HAProxy.

**Rules**:
- Agent binary MUST be a single, statically-linked executable with no external runtime dependencies
- Configuration MUST be declarable via environment variables, command-line flags, or a single config file
- Agent MUST expose a single gRPC service endpoint for HAProxy health checks
- All HAProxy communication MUST go through the defined Agent Check Protocol interface
- Agent MUST NOT require orchestration tools (Kubernetes, systemd) to function but SHOULD integrate cleanly when present

**Rationale**: HAProxy agents must be operationally simple, deployable alongside HAProxy in diverse environments (bare metal, containers, VMs), and debuggable without complex tooling. Simplicity reduces operational burden and increases reliability.

### II. Integration-Heavy Testing

Testing MUST prioritize real integration scenarios with HAProxy over isolated unit tests, ensuring the agent works correctly in production-like environments.

**Rules**:
- Every feature MUST include at least one integration test that exercises the agent with a real HAProxy instance
- Integration tests MUST verify the complete Agent Check Protocol flow (check requests, responses, status codes)
- Contract tests MUST validate gRPC service definitions against HAProxy expectations
- Integration test suites MUST be runnable locally via `make test-integration` or equivalent
- Unit tests are OPTIONAL and should be used only for complex business logic that benefits from isolation

**Rationale**: HAProxy integration is the primary value proposition. Unit tests of isolated components provide false confidence if the integration points fail. Real integration tests catch protocol mismatches, timing issues, and deployment problems.

### III. Observability

The agent MUST be transparently observable in production through structured logging, metrics exposure, and clear error reporting.

**Rules**:
- All agent operations MUST emit structured logs (JSON format) with consistent fields: timestamp, level, component, message, trace_id
- Agent MUST expose a Prometheus-compatible metrics endpoint on `/metrics` including: check_requests_total, check_duration_seconds, check_errors_total, grpc_connections_active
- Error messages MUST include actionable context (what failed, why, what to check)
- Agent MUST log HAProxy communication: incoming check requests, outgoing responses, connection state changes
- Configuration on startup MUST be logged at INFO level for audit/debugging purposes

**Rationale**: Production debugging of agent-HAProxy interactions requires visibility into request flows, timing, and failure modes. Structured logs enable automated analysis; metrics enable proactive monitoring and alerting.

### IV. HAProxy Protocol Compliance

The agent MUST strictly adhere to the HAProxy Agent Check Protocol specification, ensuring compatibility across HAProxy versions.

**Rules**:
- Agent Check Protocol responses MUST follow the documented format: `up|down|maint [weight] [reason]`
- Response timing MUST respect HAProxy timeout expectations (default 2s check timeout)
- Agent MUST handle HAProxy connection patterns: persistent connections, pipelining, abrupt disconnects
- Protocol deviations MUST be rejected with clear error messages
- Agent MUST be tested against multiple HAProxy versions (minimum: 2.4 LTS, 2.6 LTS, latest stable)

**Rationale**: HAProxy relies on precise protocol adherence for health check decisions. Protocol violations cause hard-to-debug failures in production (flapping backends, incorrect routing). Version compatibility ensures long-term operational stability.

### V. Simplicity & Reliability

The agent MUST favor simple, proven solutions over complex abstractions, prioritizing reliability and operational safety.

**Rules**:
- YAGNI: Do not build features until explicitly needed by a documented use case
- Fail-safe defaults: Agent MUST report `down` status on internal errors rather than invalid/ambiguous states
- Graceful degradation: Agent MUST continue serving health checks even if secondary features (metrics, logging) fail
- Dependencies MUST be justified: new dependencies require documented rationale in constitution complexity table
- Error handling MUST be explicit: no silent failures, all errors logged and surfaced

**Rationale**: Agent failures directly impact traffic routing and availability. Complexity increases failure modes and operational burden. Simple, reliable agents are easier to debug, deploy, and maintain across diverse production environments.

## Operational Requirements

### Production Readiness

Every feature MUST meet production-readiness criteria before merge:
- Runnable via standard deployment methods (Docker, systemd, bare binary)
- Documented configuration options with defaults and validation
- Integration tested against HAProxy in a realistic network topology
- Observability outputs (logs, metrics) verified and documented
- Failure modes identified and handled gracefully

### Deployment

Agent deployments MUST support:
- Zero-downtime updates via rolling restarts (HAProxy detects agent restarts)
- Configuration hot-reload without restart (if config change does not require binary update)
- Rollback capability (previous binary version remains deployable)

### Monitoring

Production deployments MUST monitor:
- Agent availability (process running, endpoint responding)
- Check request rate and latency
- Error rates by type (protocol errors, internal errors, timeouts)
- HAProxy connection state (active connections, reconnection rate)

## Development Workflow

### Pull Request Requirements

Every PR MUST:
1. Pass all integration tests (`make test-integration`)
2. Include new integration tests if adding/changing HAProxy interaction
3. Update documentation (README, config examples) if adding/changing behavior
4. Include observability updates (new log fields, metrics) for significant features
5. Pass code review focused on: protocol compliance, error handling, observability, simplicity

### Review Criteria

Code reviewers MUST verify:
- Constitution compliance (especially Agent Pattern, Protocol Compliance, Simplicity)
- Integration test coverage adequacy
- Error handling and logging completeness
- Performance impact (no blocking operations on check path)
- Documentation clarity

### Testing Gates

Merge MUST be blocked if:
- Integration tests fail against any supported HAProxy version
- Observability outputs are missing or unclear
- Protocol compliance is unverified
- Complexity is unjustified (see Complexity Tracking in plan.md)

## Governance

### Constitution Authority

This constitution supersedes informal practices and individual preferences. All development decisions MUST align with these principles or require documented justification via the Complexity Tracking table in `plan.md`.

### Amendment Process

Constitution amendments require:
1. Documented rationale (problem being solved, alternatives considered)
2. Team review and consensus (all active contributors)
3. Migration plan if existing code conflicts with amendment
4. Version bump following semantic versioning (see below)

### Versioning Policy

Constitution versions follow semantic versioning:
- **MAJOR**: Backward-incompatible governance changes (principle removal/redefinition)
- **MINOR**: New principles or materially expanded guidance
- **PATCH**: Clarifications, wording improvements, typo fixes

### Compliance Review

All pull requests MUST verify constitution compliance. Reviewers MUST challenge:
- Unjustified complexity (violations of Simplicity principle)
- Missing integration tests (violations of Integration-Heavy Testing principle)
- Protocol deviations (violations of HAProxy Protocol Compliance principle)
- Poor observability (violations of Observability principle)

Use `.specify/memory/constitution.md` for governance rules and the agent-specific guidance file (if present) for runtime development tips.

**Version**: 1.0.0 | **Ratified**: 2025-10-28 | **Last Amended**: 2025-10-28
