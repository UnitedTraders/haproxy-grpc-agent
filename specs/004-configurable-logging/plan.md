# Implementation Plan: Configurable Logging

**Branch**: `004-configurable-logging`
**Spec**: [spec.md](spec.md)
**Created**: 2026-05-04

## Technical Context

- **Language/Runtime**: Rust 1.75+ (edition 2024), tokio 1.x async runtime
- **Logging Framework**: tracing 0.1 + tracing-subscriber 0.3 (features: json, env-filter)
- **Config System**: TOML (toml 0.9.8), serde 1.0, clap 4.5 (CLI), environment variables
- **Key Files**: `src/logger.rs` (init), `src/config.rs` (AgentConfig, CliArgs, load/validate), `src/main.rs` (startup wiring)
- **Current Behavior**: Console-only output (stderr), single `log_level` field, `RUST_LOG` env override, JSON or pretty format
- **Test Infrastructure**: testcontainers 0.27, process-based integration tests in `tests/logging_test.rs`

### Dependencies to Add

- `tracing-appender` (from tracing ecosystem) — provides file writer with rotation support, non-blocking writer for async-safe file logging. Already maintained by the tracing team, minimal additional dependency surface.

### Dependencies Considered and Rejected

- `rolling-file` / `log4rs` — external crates with larger dependency trees, not aligned with the existing tracing ecosystem
- Custom file writing — reinventing what tracing-appender already provides, violates Simplicity principle

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Agent Pattern | PASS | Single config file, no new external runtime dependencies. tracing-appender is from the same ecosystem already in use. |
| II. Integration-Heavy Testing | PASS | Integration tests will verify file output, log level filtering, per-package overrides against running agent. |
| III. Observability | PASS | This feature directly improves observability by making logging configurable. JSON format preserved. |
| IV. HAProxy Protocol Compliance | N/A | No protocol changes. |
| V. Simplicity & Reliability | PASS | Uses existing tracing ecosystem. Graceful fallback to stderr on file failure. No new abstractions. |

### Operational Requirements Check

- **Configuration hot-reload**: Constitution mentions "Configuration hot-reload without restart (if config change does not require binary update)". Log destination changes require subscriber re-initialization which is complex in tracing. **Decision**: Log config changes require restart. This is acceptable because: (1) changing log destination is infrequent, (2) tracing's global subscriber pattern doesn't support safe hot-reload, (3) HAProxy handles agent restarts gracefully.

## Artifacts

```text
specs/004-configurable-logging/
├── spec.md              # Feature specification
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit-tasks command)
```

### Source Code (repository root)

```text
src/
├── config.rs            # AgentConfig + CliArgs (modified: new fields)
├── logger.rs            # Logging initialization (modified: file output, per-package filters)
├── main.rs              # Startup wiring (modified: log new config fields)
├── lib.rs               # Module exports (unchanged)
├── checker.rs           # Health check logic (unchanged)
├── metrics.rs           # Metrics server (unchanged)
├── protocol.rs          # Agent protocol (unchanged)
└── server.rs            # TCP server (unchanged)

tests/
├── logging_test.rs      # Existing logging tests (modified: new test cases)
├── common/mod.rs        # Test utilities (may need updates for config builder)
└── ...                  # Other test files (unchanged)
```

**Structure Decision**: Single project, same `src/` layout. Changes are isolated to config, logger, main, and tests. No new modules needed.

## Complexity Tracking

No constitution violations to justify. All changes use existing patterns and the tracing ecosystem already in the dependency tree.
