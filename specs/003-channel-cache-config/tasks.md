# Tasks: Configurable Channel Cache

**Input**: Design documents from `/specs/003-channel-cache-config/`
**Prerequisites**: plan.md (required), spec.md (required for user stories), research.md, data-model.md, quickstart.md

**Tests**: Integration tests are included per constitution principle II (Integration-Heavy Testing).

**Organization**: Tasks are grouped by user story to enable independent implementation and testing of each story.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1, US2)
- Include exact file paths in descriptions

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: No new project structure needed. This feature modifies existing files only. Phase 1 covers the test infrastructure needed by both user stories.

- [x] T001 Add `start_agent_with_config` helper function to `tests/common/mod.rs` that accepts an `AgentConfig` parameter, binds to port 0, and returns `(JoinHandle, SocketAddr)` — mirrors existing `start_agent()` but with custom config

**Checkpoint**: Test infrastructure ready for user story implementation.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Add the `grpc_channel_cache_enabled` field to `AgentConfig` with serde/default support. This is required by both US1 (checker behavior) and US2 (multi-source config).

**⚠️ CRITICAL**: No user story work can begin until this phase is complete.

- [x] T002 Add `default_grpc_channel_cache_enabled` function returning `true` in `src/config.rs`
- [x] T003 Add `grpc_channel_cache_enabled: bool` field to `AgentConfig` struct with `#[serde(default = "default_grpc_channel_cache_enabled")]` in `src/config.rs`
- [x] T004 Update `AgentConfig::default()` impl to include `grpc_channel_cache_enabled: default_grpc_channel_cache_enabled()` in `src/config.rs`
- [x] T005 Add `grpc_channel_cache_enabled` field to all existing `AgentConfig` literals in unit tests in `src/config.rs` (tests: `test_config_validation_valid`, `test_config_validation_valid_custom_ports`, `test_config_validation_port_conflict`, `test_config_validation_invalid_server_port_zero`, `test_config_validation_invalid_metrics_port_zero`, `test_config_validation_invalid_connect_timeout_zero`, `test_config_validation_invalid_rpc_timeout_zero`)
- [x] T006 Log `grpc_channel_cache_enabled` config value at startup in `src/main.rs` tracing::info block

**Checkpoint**: Config field exists with default `true`, compiles, existing tests pass (`cargo test`).

---

## Phase 3: User Story 1 — Disable Channel Caching via Configuration (Priority: P1) 🎯 MVP

**Goal**: When `grpc_channel_cache_enabled` is `false`, every health check creates a fresh gRPC channel. When `true` (default), existing caching behavior is preserved unchanged.

**Independent Test**: Set `grpc_channel_cache_enabled = false` in config, run multiple health checks to same backend, verify all succeed and no channels are cached.

### Implementation for User Story 1

- [x] T007 [US1] Modify `GrpcHealthChecker::get_or_create_channel` in `src/checker.rs`: when `self.config.grpc_channel_cache_enabled` is `false`, skip cache lookup (lines 53–70) and skip cache insert (line 114), and skip metric update on insert (line 117) — go directly to channel creation and return the fresh channel
- [x] T008 [US1] Ensure `GRPC_CHANNELS_ACTIVE` metric is never incremented when caching is disabled — verify no `metrics::GRPC_CHANNELS_ACTIVE.set(...)` calls occur on the no-cache path in `src/checker.rs`

### Integration Tests for User Story 1

- [x] T009 [US1] Create `tests/cache_config_test.rs` with test `test_health_check_cache_disabled`: start mock backend (SERVING), start agent with `grpc_channel_cache_enabled: false`, send two consecutive checks to same backend, assert both return "up"
- [x] T010 [US1] Add test `test_cache_enabled_default_behavior` in `tests/cache_config_test.rs`: start mock backend (SERVING), start agent with default config (`grpc_channel_cache_enabled: true`), send two consecutive checks, assert both return "up" (regression test for existing behavior)
- [x] T011 [US1] Add test `test_cache_disabled_unreachable_backend` in `tests/cache_config_test.rs`: start agent with `grpc_channel_cache_enabled: false`, send check to nonexistent backend, assert returns "down"

**Checkpoint**: Cache toggle works. Caching disabled = fresh channel per check. Caching enabled = existing behavior. All integration tests pass.

---

## Phase 4: User Story 2 — Configure Channel Caching via All Supported Methods (Priority: P2)

**Goal**: The `grpc_channel_cache_enabled` option is configurable via environment variable (`HAPROXY_AGENT_GRPC_CHANNEL_CACHE`), TOML config file, and CLI argument (`--grpc-channel-cache` / `--no-grpc-channel-cache`), following existing precedence.

**Independent Test**: Set the option via each config method independently, verify agent respects the setting.

### Implementation for User Story 2

- [x] T012 [US2] Add `HAPROXY_AGENT_GRPC_CHANNEL_CACHE` environment variable parsing in `AgentConfig::load_from_env` in `src/config.rs` — parse "true"/"false" strings, bail on invalid values
- [x] T013 [US2] Add `--grpc-channel-cache` / `--no-grpc-channel-cache` CLI argument to `CliArgs` struct in `src/config.rs` using `#[arg(long, default_value_t = true, action = clap::ArgAction::Set)]`
- [x] T014 [US2] Add `grpc_channel_cache` override in `AgentConfig::apply_cli_overrides` in `src/config.rs` — apply `cli.grpc_channel_cache` to `config.grpc_channel_cache_enabled`

### Unit Tests for User Story 2

- [x] T015 [US2] Add unit test `test_config_default_grpc_channel_cache_enabled` in `src/config.rs` — verify `AgentConfig::default().grpc_channel_cache_enabled == true`

**Checkpoint**: Config option works via all three methods with correct precedence. Existing tests pass.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: Final validation across all stories.

- [x] T016 Run `cargo clippy` and fix any warnings introduced by this feature
- [x] T017 Run full test suite (`cargo test`) and verify all existing + new tests pass
- [x] T018 Validate quickstart.md scenarios manually: verify env var, TOML, and CLI examples from `specs/003-channel-cache-config/quickstart.md` are accurate

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — can start immediately
- **Foundational (Phase 2)**: T002-T004 are sequential (field depends on default fn). T005-T006 can start after T004.
- **User Story 1 (Phase 3)**: Depends on Phase 2 completion. T007-T008 are sequential. T009-T011 depend on T007-T008 and T001.
- **User Story 2 (Phase 4)**: Depends on Phase 2 completion. Independent of US1. T012-T014 can run in parallel (different functions).
- **Polish (Phase 5)**: Depends on Phases 3 and 4 completion.

### User Story Dependencies

- **User Story 1 (P1)**: Depends on Phase 2 only. No dependency on US2.
- **User Story 2 (P2)**: Depends on Phase 2 only. No dependency on US1.
- US1 and US2 can be implemented in parallel after Phase 2.

### Within Each User Story

- Implementation tasks before test tasks (tests need the code to test against)
- Core logic (T007) before metric handling (T008)
- Config parsing tasks (T012-T014) can run in parallel (different functions in same file)

### Parallel Opportunities

- T001 (test helper) can run in parallel with T002-T006 (config changes) — different files
- T012, T013, T014 can run in parallel — different functions in config.rs
- US1 and US2 phases can run in parallel after Phase 2

---

## Parallel Example: User Story 1

```text
# After Phase 2 complete, launch implementation:
Task T007: "Modify get_or_create_channel cache bypass in src/checker.rs"
Task T008: "Verify metric handling on no-cache path in src/checker.rs"

# After T007+T008 complete, launch tests:
Task T009: "Test cache disabled with healthy backend"
Task T010: "Test cache enabled default behavior"
Task T011: "Test cache disabled with unreachable backend"
```

## Parallel Example: User Story 2

```text
# After Phase 2 complete, launch all config source tasks:
Task T012: "Add HAPROXY_AGENT_GRPC_CHANNEL_CACHE env var parsing in src/config.rs"
Task T013: "Add --grpc-channel-cache CLI arg in src/config.rs"
Task T014: "Add CLI override in apply_cli_overrides in src/config.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup (T001)
2. Complete Phase 2: Foundational (T002-T006)
3. Complete Phase 3: User Story 1 (T007-T011)
4. **STOP and VALIDATE**: `cargo test --test cache_config_test` — all pass
5. Feature is usable with TOML config file only (serde handles it)

### Incremental Delivery

1. Setup + Foundational → Config field exists with default
2. Add User Story 1 → Cache toggle works → Test independently
3. Add User Story 2 → All config methods work → Test independently
4. Polish → Full validation

---

## Notes

- [P] tasks = different files, no dependencies
- [Story] label maps task to specific user story for traceability
- Constitution requires integration tests (Principle II) — test tasks included
- No new dependencies needed — all changes use existing crates
- Commit after each phase completion
- Stop at any checkpoint to validate independently
