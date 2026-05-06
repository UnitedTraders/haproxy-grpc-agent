# Research: Reduce Info-Level Logging

## Decision 1: Which info-level log entries to reclassify

**Decision**: Reclassify 6 per-connection/per-request `info` entries to `debug`. Keep 6 lifecycle `info` entries unchanged.

**Entries moving to `debug`**:

| File | Line | Current Message | Rationale for debug |
|------|------|-----------------|---------------------|
| src/server.rs | 57 | "HAProxy connection established" | Per-connection; noisy under load |
| src/server.rs | 77 | "HAProxy connection closed" | Per-connection; noisy under load |
| src/server.rs | 123 | "Processing health check request" | Per-request; noisy under load |
| src/server.rs | 139 | "Health check completed" | Per-request; noisy under load |
| src/main.rs | 73 | "Metrics server configured successfully" | One-time startup detail; not a critical lifecycle event |
| src/metrics.rs | 105 | "Metrics server listening" | One-time startup detail; address already logged in main startup block |

**Entries remaining at `info`**:

| File | Line | Current Message | Rationale for keeping info |
|------|------|-----------------|---------------------------|
| src/server.rs | 42 | "Agent Text Protocol server listening" | Critical lifecycle: server is ready |
| src/main.rs | 35 | "Received Ctrl+C signal" | Critical lifecycle: shutdown trigger |
| src/main.rs | 38 | "Received SIGTERM signal" | Critical lifecycle: shutdown trigger |
| src/main.rs | 42 | "Starting graceful shutdown..." | Critical lifecycle: shutdown in progress |
| src/main.rs | 54 | "HAProxy gRPC Agent starting" | Critical lifecycle: startup config audit |
| src/main.rs | 93 | "Initialization complete, starting server" | Critical lifecycle: ready state |
| src/main.rs | 106 | "Shutdown signal received, stopping server..." | Critical lifecycle: shutdown |
| src/main.rs | 115 | "Shutdown complete" | Critical lifecycle: clean exit |

**Alternatives considered**:
- Move ALL non-startup entries to debug → rejected; "Initialization complete" is a useful operational signal
- Add a new `trace` level for request details, keep connection events at `debug` → rejected; overengineered for current needs
- Keep metrics server events at info → rejected; the main startup log already includes `metrics_port`, making a separate "metrics listening" entry redundant at info level

## Decision 2: Impact on existing warn/error entries

**Decision**: No changes to existing `warn` or `error` entries.

**Rationale**: The current classification is correct:
- `warn`: connection handling errors (recoverable), metrics init failure (graceful degradation), timeout warnings
- `error`: accept failures, write/flush failures, metrics encoding, health check errors

These all represent genuine agent-level problems. No reclassification needed.

## Decision 3: Test impact

**Decision**: No test changes required.

**Rationale**: None of the existing tests assert on specific log output at the `info` level. The `config_logging_test.rs` integration tests that check stderr for JSON logs use the startup message "HAProxy gRPC Agent starting" which remains at `info`. The `test_backward_compat_top_level_log_level` test checks that INFO messages are suppressed at warn level — this test will still pass since the startup log is at info.
