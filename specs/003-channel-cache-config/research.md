# Research: Configurable Channel Cache

**Feature**: 003-channel-cache-config
**Date**: 2026-02-23

## R1: Boolean config field patterns in clap + serde

**Decision**: Use `#[serde(default = "default_true")]` for TOML/serde and `#[arg(long, default_value_t = true)]` with `--no-grpc-channel-cache` negation for clap CLI.

**Rationale**: clap 4.x supports boolean flags with `--flag` / `--no-flag` syntax via `#[arg(long, default_value_t = true, action = clap::ArgAction::Set)]`. For the environment variable, parse `"true"` / `"false"` strings matching the existing pattern used for other env vars in `load_from_env`.

**Alternatives considered**:
- `clap::ArgAction::SetTrue` / `SetFalse` — doesn't support default `true` cleanly
- Separate `--disable-channel-cache` flag — inconsistent with positive-bool convention

## R2: DashMap behavior when unused

**Decision**: Instantiate `DashMap::new()` even when caching is disabled; never insert into it.

**Rationale**: An empty `DashMap` allocates minimal memory (just the internal shard array headers). Using `Option<DashMap>` would add `.as_ref().unwrap()` noise throughout `get_or_create_channel` for no meaningful benefit. The empty map is effectively free.

**Alternatives considered**:
- `Option<Arc<DashMap>>` — adds unwrap complexity, `None` path, conditional metric reporting
- Trait-based strategy pattern — over-engineered for a boolean toggle

## R3: Channel lifecycle when caching is disabled

**Decision**: Create a fresh `Channel` for each `check_backend_internal` call. The channel is used for the single RPC and then dropped (Rust ownership ensures cleanup).

**Rationale**: tonic `Channel` implements `Drop` which closes the underlying HTTP/2 connection. When the channel goes out of scope after the health check completes, the connection is cleaned up. No explicit teardown needed.

**Alternatives considered**:
- Channel pool with size=0 — unnecessary abstraction
- Explicit `channel.close()` — not available in tonic's API; Drop handles it

## R4: Test approach for verifying cache bypass

**Decision**: Use two consecutive health checks to the same backend with caching disabled. Verify both succeed (functional correctness). Verify `GRPC_CHANNELS_ACTIVE` metric stays at 0 (behavioral verification that caching is bypassed).

**Rationale**: Directly inspecting the DashMap internals would require exposing implementation details. The metric-based approach tests the observable behavior without coupling tests to internal data structures.

**Alternatives considered**:
- Expose `channel_cache.len()` as public API — leaks implementation details
- Mock the DashMap — impractical with concrete type, and constitution favors integration tests
