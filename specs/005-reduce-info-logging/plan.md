# Implementation Plan: Reduce Info-Level Logging

## Technical Context

| Aspect | Detail |
|--------|--------|
| Language/Runtime | Rust 1.75+ (edition 2024), tokio 1.x |
| Logging Framework | tracing + tracing-subscriber (JSON format), tracing-appender (file output) |
| Key Files | src/server.rs (11 log calls), src/main.rs (11 log calls), src/metrics.rs (3 log calls), src/checker.rs (1 log call) |
| Test Files | tests/config_logging_test.rs (8 integration tests), src/config.rs (unit tests) |
| Current Log Distribution | info: 12, warn: 3, error: 8, debug: 1, trace: 0 |

## Constitution Check

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Agent Pattern | PASS | No dependency changes; no structural changes |
| II. Integration-Heavy Testing | PASS | Existing tests unaffected; log level changes don't alter test assertions |
| III. Observability | PASS | All log statements preserved; startup config remains at INFO; HAProxy communication still logged (at DEBUG). Constitution says "MUST log" — does not mandate a specific level |
| IV. HAProxy Protocol Compliance | PASS | No protocol changes |
| V. Simplicity & Reliability | PASS | Pure reclassification; minimal change surface |
| PR Requirements | PASS | Will include `cargo fmt --all`, docs update, test verification |
| Testing Gates | PASS | No integration test changes needed |

**Constitution note on Principle III**: The rule states "Agent MUST log HAProxy communication: incoming check requests, outgoing responses, connection state changes." This feature preserves all these log entries — they move from `info` to `debug`, so they remain logged and visible when operators set verbosity to `debug`. Startup configuration remains at `info` per the explicit constitution requirement.

## Structure Decision

No new files or directories needed. Changes are limited to modifying `tracing::info!` to `tracing::debug!` in existing source files:

```
src/
  server.rs   — 4 info→debug changes (connection established, connection closed, processing request, check completed)
  main.rs     — 1 info→debug change (metrics server configured)
  metrics.rs  — 1 info→debug change (metrics server listening)
```

Unchanged files: config.rs, logger.rs, protocol.rs, checker.rs, lib.rs.

## Complexity Tracking

No violations to justify. This is a minimal change with zero new dependencies or abstractions.
