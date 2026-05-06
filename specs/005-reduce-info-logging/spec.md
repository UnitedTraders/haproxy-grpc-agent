# Feature Specification: Reduce Info-Level Logging

## Overview

The agent currently produces excessive output at the `info` log level. Every individual connection open, connection close, health check request, and health check response generates an `info`-level log entry. In high-traffic environments this creates log noise that obscures genuinely important events.

This feature reclassifies log levels across the codebase so that:
- **`info`** is reserved for important lifecycle events (startup, shutdown, configuration summary)
- **`debug`** is used for per-request/per-connection operational activity
- **`warn`** and **`error`** remain for agent-level problems and failures

## User Scenarios & Testing

### US1: Operator monitors agent under normal load (P1)

**As** a systems operator running the agent in production,
**I want** the default `info` log level to show only important events,
**So that** I can quickly spot meaningful state changes without scrolling through per-request noise.

**Acceptance criteria:**
- At `info` level, the agent logs startup configuration, server listening, and shutdown events
- At `info` level, the agent does NOT log individual connection opens, connection closes, health check requests, or health check responses
- Individual request/connection logs are still available at `debug` level

### US2: Operator troubleshoots a specific backend (P2)

**As** a systems operator investigating a failing backend,
**I want** to increase log verbosity to `debug` to see per-request details,
**So that** I can observe each health check request and its result.

**Acceptance criteria:**
- At `debug` level, all per-connection and per-request log entries are visible
- Debug output includes backend address, SSL flag, and health check result for each request
- No log entries are lost compared to the current behavior; they are only reclassified

### US3: Operator identifies agent problems via warn/error logs (P2)

**As** a systems operator,
**I want** `warn` and `error` levels to indicate agent-level problems,
**So that** I can set up alerts on these levels with confidence they represent real issues.

**Acceptance criteria:**
- `warn` is used for recoverable problems (connection handling errors, timeout warnings, non-fatal initialization failures)
- `error` is used for unrecoverable or critical failures (failed to accept connections, failed to write responses, metrics encoding errors)
- No routine operational events use `warn` or `error`

## Functional Requirements

### FR1: Reclassify per-connection logs to debug level
Connection established and connection closed events for individual HAProxy connections must be logged at `debug` instead of `info`.

### FR2: Reclassify per-request logs to debug level
Health check request processing and health check completion events must be logged at `debug` instead of `info`.

### FR3: Retain lifecycle events at info level
Server startup configuration, server listening address, initialization complete, signal receipt, and shutdown events must remain at `info` level.

### FR4: Retain metrics server lifecycle at info level
Metrics server configured and metrics server listening events must remain at `info` level.

### FR5: Preserve warn and error classifications
Existing `warn`-level entries (connection handling errors, metrics initialization failures, timeout warnings) and `error`-level entries (accept failures, write/flush failures, metrics encoding errors, health check errors) must retain their current levels.

### FR6: No log entries removed
All existing log statements must be preserved. The change is limited to adjusting the log level macro (e.g., `tracing::info!` to `tracing::debug!`).

### FR7: Backward compatibility with per-package overrides
Operators using `[logging.packages]` configuration to set per-module log levels must continue to see expected filtering behavior with the reclassified levels.

## Success Criteria

- At default `info` level, the agent produces no more than 10 log entries during a startup-check-shutdown cycle (startup + listening + shutdown messages only; no per-request entries)
- Switching to `debug` level restores full per-request visibility with no entries missing compared to the current `info`-level output
- Existing `warn` and `error` entries are unchanged in level and content
- All existing unit and integration tests pass without modification (log level changes do not affect test assertions)

## Assumptions

- The current set of `warn` and `error` log statements are correctly classified and do not need reclassification
- Operators who need per-request logging in production will use `debug` level or per-package overrides to enable it
- The `eprintln!` timeout warnings in config validation are outside the scope of this change (they are not tracing calls)
