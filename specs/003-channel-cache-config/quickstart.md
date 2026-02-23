# Quickstart: Configurable Channel Cache

**Feature**: 003-channel-cache-config

## What Changed

A new configuration option `grpc_channel_cache_enabled` controls whether the agent caches gRPC channels between health checks. By default, caching is enabled (existing behavior). Setting it to `false` forces the agent to create a fresh gRPC connection for every health check.

## How to Use

### Disable channel caching via environment variable

```bash
HAPROXY_AGENT_GRPC_CHANNEL_CACHE=false ./haproxy-grpc-agent
```

### Disable channel caching via TOML config file

```toml
# agent.toml
grpc_channel_cache_enabled = false
```

```bash
./haproxy-grpc-agent -c agent.toml
```

### Disable channel caching via CLI argument

```bash
./haproxy-grpc-agent --no-grpc-channel-cache
```

### Keep default behavior (caching enabled)

No change needed. The default is `true`. All existing configurations continue to work without modification.

## Files Modified

| File | Change |
|------|--------|
| `src/config.rs` | New `grpc_channel_cache_enabled` field in `AgentConfig`, env var parsing, CLI arg, default function |
| `src/checker.rs` | Conditional cache bypass in `get_or_create_channel` |
| `src/main.rs` | Log new config value at startup |
| `tests/common/mod.rs` | New `start_agent_with_config` helper |
| `tests/cache_config_test.rs` | New integration tests for cache-disabled behavior |

## Verification

Run all tests:

```bash
cargo test
```

Run only the new cache config tests:

```bash
cargo test --test cache_config_test
```

Verify with clippy:

```bash
cargo clippy
```
