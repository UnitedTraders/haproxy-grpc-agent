# Implementation Plan: Configurable Channel Cache

**Branch**: `003-channel-cache-config` | **Date**: 2026-02-23 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/003-channel-cache-config/spec.md`

## Summary

Add a boolean configuration option `grpc_channel_cache_enabled` (default: `true`) to `AgentConfig` that controls whether `GrpcHealthChecker` caches and reuses gRPC channels across health checks. When disabled, each health check creates a fresh channel and the `GRPC_CHANNELS_ACTIVE` metric remains at 0. The option follows existing configuration patterns: environment variable, TOML file, and CLI argument with standard precedence.

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2024)
**Primary Dependencies**: tokio 1.x, tonic 0.14.2, dashmap 6.1.0, clap 4.5, serde 1.0, toml 0.9.8, prometheus 0.14
**Storage**: N/A (in-memory DashMap cache, no persistence)
**Testing**: cargo test + testcontainers 0.27 (integration tests with mock gRPC backend)
**Target Platform**: Linux server / Docker container
**Project Type**: Single Rust binary
**Performance Goals**: No degradation when caching enabled (existing behavior). Acceptable increased latency per check when caching disabled (operator's explicit choice).
**Constraints**: Total gRPC timeout < 2000ms (HAProxy default). No new dependencies required.
**Scale/Scope**: Single config field addition, 3 files modified (config.rs, checker.rs, tests/common/mod.rs), 1 new test file.

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Agent Pattern | PASS | Single config option, no new external dependencies, follows existing config patterns (env/file/CLI) |
| II. Integration-Heavy Testing | PASS | New integration test will verify cache-disabled behavior with real mock backend via testcontainers |
| III. Observability | PASS | `GRPC_CHANNELS_ACTIVE` metric correctly reports 0 when caching disabled; startup log includes new config value |
| IV. HAProxy Protocol Compliance | PASS | No protocol changes; agent still returns up/down via Agent Check Protocol |
| V. Simplicity & Reliability | PASS | Minimal change: one boolean field, conditional logic in one method. No abstractions, no new patterns. YAGNI-compliant. |

No violations. No entries needed in Complexity Tracking table.

## Project Structure

### Documentation (this feature)

```text
specs/003-channel-cache-config/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (created by /speckit.tasks)
```

### Source Code (repository root)

```text
src/
├── checker.rs           # MODIFY: conditional cache logic in get_or_create_channel
├── config.rs            # MODIFY: add grpc_channel_cache_enabled field + env/CLI/validation
├── lib.rs               # NO CHANGE
├── logger.rs            # NO CHANGE
├── main.rs              # MODIFY: log new config value at startup
├── metrics.rs           # NO CHANGE
├── protocol.rs          # NO CHANGE
└── server.rs            # NO CHANGE

tests/
├── common/mod.rs        # MODIFY: add start_agent_with_config helper accepting AgentConfig
├── integration_test.rs  # NO CHANGE
├── resilience_test.rs   # NO CHANGE
└── cache_config_test.rs # NEW: tests for cache-enabled/disabled behavior
```

**Structure Decision**: Existing single-project layout preserved. Changes touch 3 source files (config.rs, checker.rs, main.rs), 1 test utility (common/mod.rs), and add 1 test file (cache_config_test.rs).

## Complexity Tracking

No violations to justify.

## Design Decisions

### D1: Config field name

**Decision**: `grpc_channel_cache_enabled` (bool, default `true`)
**Rationale**: Follows existing naming convention (`grpc_connect_timeout_ms`, `grpc_rpc_timeout_ms`). Positive boolean (`enabled`) avoids double-negative confusion (`no_cache = false`).
**Env var**: `HAPROXY_AGENT_GRPC_CHANNEL_CACHE` (values: `true`/`false`)
**CLI flag**: `--grpc-channel-cache` / `--no-grpc-channel-cache` (clap boolean flag)
**TOML key**: `grpc_channel_cache_enabled = true`

### D2: Implementation approach in checker.rs

**Decision**: Conditional bypass in `get_or_create_channel` — when caching is disabled, skip cache lookup/insert and always create a fresh channel.
**Rationale**: Minimal code change. The channel creation logic is identical regardless of caching. Only the DashMap interactions are skipped.
**Alternative rejected**: Separate `CachedChecker` / `UncachedChecker` types — over-engineered for a boolean toggle.

### D3: DashMap instantiation when caching disabled

**Decision**: Still create the DashMap but never use it. The `Arc<DashMap>` is a zero-cost allocation when empty.
**Rationale**: Avoids `Option<Arc<DashMap>>` complexity. An empty DashMap has negligible memory footprint.

### D4: Test strategy

**Decision**: New `tests/cache_config_test.rs` with integration tests using testcontainers. Add `start_agent_with_config` to `tests/common/mod.rs` to allow passing custom `AgentConfig`.
**Rationale**: Constitution requires integration tests. Existing tests use default config and must remain unchanged. New helper enables config customization without breaking existing test patterns.
