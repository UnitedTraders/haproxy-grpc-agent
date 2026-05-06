# Tasks: Reduce Info-Level Logging

**Input**: Design documents from `/specs/005-reduce-info-logging/`
**Prerequisites**: plan.md, spec.md, research.md

**Tests**: No new tests required. Existing tests verify the retained `info`-level startup messages. Log level changes do not affect test assertions.

**Organization**: Tasks grouped by user story. US1 is the core change; US2 and US3 are verification-only (no code changes beyond US1).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)

---

## Phase 1: Setup

- [x] T001 Verify current log level distribution by running `cargo test --lib --bins` to confirm baseline passes

## Phase 2: US1 — Reduce info-level logging to important events only

**Goal**: Reclassify per-connection and per-request log entries from `info` to `debug` so that default `info` output shows only lifecycle events.

**Independent test criteria**: At `info` level, a startup-check-shutdown cycle produces only lifecycle messages (startup, listening, init complete, shutdown). No per-connection or per-request entries.

- [x] T002 [P] [US1] Change "HAProxy connection established" from `tracing::info!` to `tracing::debug!` in src/server.rs:57
- [x] T003 [P] [US1] Change "HAProxy connection closed" from `tracing::info!` to `tracing::debug!` in src/server.rs:77
- [x] T004 [P] [US1] Change "Processing health check request" from `tracing::info!` to `tracing::debug!` in src/server.rs:123
- [x] T005 [P] [US1] Change "Health check completed" from `tracing::info!` to `tracing::debug!` in src/server.rs:139
- [x] T006 [P] [US1] Change "Metrics server configured successfully" from `tracing::info!` to `tracing::debug!` in src/main.rs:73
- [x] T007 [P] [US1] Change "Metrics server listening" from `tracing::info!` to `tracing::debug!` in src/metrics.rs:105

## Phase 3: US2 & US3 — Verification

**Goal**: Confirm debug-level visibility and warn/error correctness.

- [x] T008 Run `cargo test --lib --bins` to verify all unit tests pass after reclassification
- [x] T009 Run `cargo test --test config_logging_test -- --include-ignored --test-threads=1` to verify integration tests pass
- [x] T010 Run `cargo clippy --all-targets --all-features -- -D warnings` to verify zero warnings

## Phase 4: Polish

- [x] T011 Run `cargo fmt --all` to ensure code formatting
- [x] T012 Verify no remaining `tracing::info!` calls exist for per-request/per-connection events (grep src/ for info! and confirm only lifecycle entries remain)

---

## Dependencies

```
T001 (baseline) → T002-T007 (all parallel) → T008-T010 (verification) → T011-T012 (polish)
```

## Implementation Strategy

### MVP Scope
All user stories are delivered in a single pass — the code changes (T002-T007) are trivial and atomic. US2 and US3 are satisfied automatically by the same changes that satisfy US1.

### Execution Order
1. Verify baseline (T001)
2. Apply all 6 log level changes in parallel (T002-T007)
3. Run full test suite (T008-T010)
4. Format and verify (T011-T012)

---

## Notes

- All 6 code changes (T002-T007) touch different lines and can be applied in any order
- No new files created; no files deleted
- No dependency changes
- Existing tests remain valid without modification
