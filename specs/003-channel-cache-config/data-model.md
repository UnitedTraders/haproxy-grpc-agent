# Data Model: Configurable Channel Cache

**Feature**: 003-channel-cache-config
**Date**: 2026-02-23

## Modified Entities

### AgentConfig

Existing configuration struct extended with one new field.

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| server_port | u16 | 5555 | TCP port for Agent Text Protocol server |
| server_bind_address | String | "0.0.0.0" | Bind address for agent server |
| grpc_connect_timeout_ms | u64 | 1000 | gRPC connection timeout in milliseconds |
| grpc_rpc_timeout_ms | u64 | 1500 | gRPC RPC timeout in milliseconds |
| **grpc_channel_cache_enabled** | **bool** | **true** | **Whether to cache and reuse gRPC channels across health checks** |
| metrics_port | u16 | 9090 | HTTP port for Prometheus metrics |
| metrics_bind_address | String | "0.0.0.0" | Bind address for metrics server |
| log_level | LogLevel | Info | Logging verbosity level |
| log_format | LogFormat | Json | Logging output format |

**New field highlighted in bold.**

### Configuration Sources

| Source | Key/Flag | Example |
|--------|----------|---------|
| Environment variable | `HAPROXY_AGENT_GRPC_CHANNEL_CACHE` | `HAPROXY_AGENT_GRPC_CHANNEL_CACHE=false` |
| TOML config file | `grpc_channel_cache_enabled` | `grpc_channel_cache_enabled = false` |
| CLI argument | `--grpc-channel-cache` / `--no-grpc-channel-cache` | `--no-grpc-channel-cache` |

### Precedence

Environment variable < TOML config file < CLI argument (unchanged from existing behavior).

## Unchanged Entities

- **BackendChannelKey**: No changes (server, port, ssl_flag)
- **GrpcHealthChecker**: Internal struct unchanged; behavior modified by reading `config.grpc_channel_cache_enabled`
- **HealthCheckRequest / HealthCheckResponse**: No changes
- **Metrics**: No new metrics; `GRPC_CHANNELS_ACTIVE` reports 0 when caching disabled
