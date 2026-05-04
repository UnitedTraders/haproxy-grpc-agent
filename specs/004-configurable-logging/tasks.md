# Tasks: Configurable Logging

**Input**: Design documents from `specs/004-configurable-logging/`
**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2, US3)
- Include exact file paths in descriptions

---

## Phase 1: Setup

> Add the `tracing-appender` dependency and define new config types. No behavioral changes yet.

- [x] T001 Add `tracing-appender` dependency to `Cargo.toml`
- [x] T002 [P] Define `LogDestination` enum (`Console`, `File`) with serde/clap derives in `src/config.rs`
- [x] T003 [P] Define `LoggingConfig` struct with all fields (`destination`, `level`, `format`, `file_path`, `file_max_size_mb`, `file_max_files`, `packages`) in `src/config.rs`
- [x] T004 Add `logging` field (with `#[serde(default)]`) to `AgentConfig` struct in `src/config.rs`
- [x] T005 Implement `Default` for `LoggingConfig` (destination=Console, all options None, empty packages map) in `src/config.rs`
- [x] T006 Update `AgentConfig::default()` to include `logging: LoggingConfig::default()` in `src/config.rs`
- [x] T007 Add validation for `LoggingConfig` in `AgentConfig::validate()`: if destination=File then file_path required; file_max_size_mb > 0 if set; file_max_files > 0 if set in `src/config.rs`
- [x] T008 Add unit tests for `LoggingConfig` default values and validation rules in `src/config.rs`
- [x] T009 Run `cargo test` and `cargo clippy` to verify Phase 1 compiles and existing tests pass

**Checkpoint**: `cargo test` passes. New types exist but are not wired into logger. All existing behavior unchanged.

---

## Phase 2: Foundational — Resolved Log Level and EnvFilter Composition

> Build the helper that resolves the effective log level (precedence chain) and composes an `EnvFilter` directive string from config. This is shared by all user stories.

- [x] T010 Implement `LoggingConfig::resolved_level()` method that returns the effective `LogLevel` by checking `logging.level` first, then falling back to top-level `log_level` in `src/config.rs`
- [x] T011 Implement `LoggingConfig::build_env_filter_directive()` method that composes a directive string from resolved level + per-package overrides (e.g., `"info,haproxy_grpc_agent::checker=debug"`) in `src/config.rs`
- [x] T012 Add unit tests for `resolved_level()` covering: logging.level set, logging.level unset (falls back to top-level), both set (logging.level wins) in `src/config.rs`
- [x] T013 Add unit tests for `build_env_filter_directive()` covering: no overrides, single override, multiple overrides, empty packages map in `src/config.rs`
- [x] T014 Run `cargo test` and `cargo clippy` to verify Phase 2

**Checkpoint**: `cargo test` passes. Directive builder is tested in isolation. Logger not yet modified.

---

## Phase 3: User Story 1 — Configure Log Destination (P1)

> Enable writing logs to a file instead of console, controlled by `[logging]` config section.

- [x] T015 [US1] Refactor `logger::init()` to accept `&AgentConfig` and branch on `logging.destination`: Console path preserves current `fmt().init()` behavior in `src/logger.rs`
- [x] T016 [US1] Implement File destination path in `logger::init()`: create `tracing_appender::rolling::never()` writer for non-rotating file output, wrap in `NonBlocking`, use `fmt().with_writer()` in `src/logger.rs`
- [x] T017 [US1] Wire `build_env_filter_directive()` into `logger::init()` to replace hardcoded `log_level` match — use directive string for `EnvFilter::new()`, preserving `RUST_LOG` override check in `src/logger.rs`
- [x] T018 [US1] Update startup logging in `main.rs` to log `logging.destination`, `logging.file_path`, and resolved log level in `src/main.rs`
- [x] T019 [US1] Add environment variable support for log destination: `HAPROXY_AGENT_LOG_DESTINATION` and `HAPROXY_AGENT_LOG_FILE_PATH` in `AgentConfig::load_from_env()` in `src/config.rs`
- [x] T020 [US1] Add CLI arguments `--log-destination` and `--log-file-path` to `CliArgs` and wire into `apply_cli_overrides()` in `src/config.rs`
- [x] T021 [US1] Update existing unit tests in `src/config.rs` that construct `AgentConfig` literals to include the new `logging` field
- [x] T022 [US1] Add integration test: agent starts with `destination = "file"` config, verify log file is created and contains JSON log lines in `tests/logging_test.rs`
- [x] T023 [US1] Add integration test: agent starts with `destination = "console"` config (or no `[logging]` section), verify logs appear on stderr in `tests/logging_test.rs`
- [x] T024 [US1] Add integration test: agent starts with `destination = "file"` but no `file_path`, verify agent exits with error in `tests/logging_test.rs`
- [x] T025 [US1] Run `cargo test` and `cargo clippy` to verify Phase 3

**Checkpoint**: File logging works end-to-end. Console logging unchanged. Agent validates config at startup. Can be independently tested and demonstrated.

---

## Phase 4: User Story 2 — Default Log Level via Config File (P1)

> Default log level is already wired via `resolved_level()` in Phase 2. This phase adds integration test coverage.

- [x] T026 [US2] Add integration test: agent starts with `[logging] level = "debug"`, verify debug-level messages appear in output in `tests/logging_test.rs`
- [x] T027 [US2] Add integration test: agent starts with `[logging] level = "error"`, verify only error-level messages appear in `tests/logging_test.rs`
- [x] T028 [US2] Add integration test: agent starts with no `[logging]` section and `log_level = "warn"` (backward compat), verify warn+ messages only in `tests/logging_test.rs`
- [x] T029 [US2] Add integration test: agent starts with both `log_level = "warn"` and `[logging] level = "debug"`, verify `[logging].level` wins in `tests/logging_test.rs`
- [x] T030 [US2] Run `cargo test` and `cargo clippy` to verify Phase 4

**Checkpoint**: Log level configuration via `[logging]` section works. Backward compatibility with top-level `log_level` verified. Can be independently tested.

---

## Phase 5: User Story 3 — Per-Package Log Level Overrides (P2)

> Per-package filtering is already built into the directive builder (Phase 2). This phase adds integration tests and ensures `RUST_LOG` precedence.

- [x] T031 [US3] Add integration test: default level=info with `[logging.packages] "haproxy_grpc_agent::checker" = "debug"`, verify checker debug messages appear while other debug messages are suppressed in `tests/logging_test.rs`
- [x] T032 [US3] Add integration test: verify `RUST_LOG` env var overrides per-package config when both are set in `tests/logging_test.rs`
- [x] T033 [US3] Add integration test: per-package override referencing non-existent module is silently accepted (no startup error) in `tests/logging_test.rs`
- [x] T034 [US3] Run `cargo test` and `cargo clippy` to verify Phase 5

**Checkpoint**: Per-package overrides work. RUST_LOG precedence preserved. Non-existent modules handled gracefully.

---

## Phase 6: User Story 4 — Log File Rotation (P3)

> Add size-based rotation using `tracing-appender`.

- [x] T035 [US4] Implement rotation logic in `logger::init()`: when `file_max_size_mb` is set, use `tracing_appender::rolling` with size-based rotation policy; default `file_max_files` to 5 if not set in `src/logger.rs`
- [x] T036 [US4] Add validation: if `file_max_files` set without `file_max_size_mb`, log a warning and ignore `file_max_files` in `AgentConfig::validate()` in `src/config.rs`
- [x] T037 [US4] Add environment variable support: `HAPROXY_AGENT_LOG_FILE_MAX_SIZE_MB` and `HAPROXY_AGENT_LOG_FILE_MAX_FILES` in `AgentConfig::load_from_env()` in `src/config.rs`
- [x] T038 [US4] Add CLI arguments `--log-file-max-size-mb` and `--log-file-max-files` to `CliArgs` and wire into `apply_cli_overrides()` in `src/config.rs`
- [x] T039 [US4] Add integration test: file logging with small `file_max_size_mb`, generate enough logs to trigger rotation, verify rotated files exist in `tests/logging_test.rs`
- [x] T040 [US4] Add integration test: file logging without rotation config, verify single log file grows without rotation in `tests/logging_test.rs`
- [x] T041 [US4] Run `cargo test` and `cargo clippy` to verify Phase 6

**Checkpoint**: Log rotation works. Rotation config validated. Backward compatible (no rotation by default).

---

## Phase 7: Polish & Cross-Cutting Concerns

> Update test utilities, verify backward compatibility, and finalize.

- [x] T042 Update `tests/common/mod.rs` helper functions that build `AgentConfig` to include `logging` field with defaults
- [x] T043 Add integration test: agent starts with legacy config file (no `[logging]` section, only `log_level` and `log_format`), verify identical behavior to pre-feature baseline in `tests/logging_test.rs`
- [x] T044 Add integration test: invalid log level string in config file causes clear error on startup in `tests/logging_test.rs`
- [x] T045 Run full test suite (`cargo test`), `cargo clippy`, verify zero warnings
- [x] T046 Update startup log message in `src/main.rs` to include all new logging config fields (destination, file_path, per-package override count)

---

## Dependencies

```text
Phase 1 (Setup)
  └── Phase 2 (Foundational: resolved level + directive builder)
        ├── Phase 3 (US1: Log Destination) — can start after Phase 2
        ├── Phase 4 (US2: Default Log Level) — can start after Phase 2
        ├── Phase 5 (US3: Per-Package Overrides) — can start after Phase 2
        └── Phase 6 (US4: Rotation) — can start after Phase 3 (needs file output)
              └── Phase 7 (Polish) — after all user stories
```

## Implementation Strategy

### MVP (Minimum Viable)

Complete Phases 1-4 (Setup + Foundational + US1 + US2). This delivers:
- File and console log destination switching
- Default log level via `[logging]` config section
- Full backward compatibility

### Incremental Delivery

1. **Phase 1-2**: Types + directive builder (no behavior change)
2. **Phase 3**: File logging works → demo to stakeholders
3. **Phase 4**: Log level via new config → verify backward compat
4. **Phase 5**: Per-package overrides → power users enabled
5. **Phase 6**: Rotation → operational completeness
6. **Phase 7**: Polish → production ready

### Parallel Opportunities

After Phase 2, Phases 3/4/5 can proceed in parallel (different test files, independent features). Phase 6 depends on Phase 3 (file output must exist).

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Each user story should be independently completable and testable
- Commit after each task or logical group
- Stop at any checkpoint to validate story independently
- `RUST_LOG` always takes ultimate precedence — this is existing behavior preserved throughout
