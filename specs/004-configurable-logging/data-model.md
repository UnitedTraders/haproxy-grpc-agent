# Data Model: Configurable Logging

**Feature**: 004-configurable-logging
**Created**: 2026-05-04

## Entity Changes

### AgentConfig (modified)

The existing `AgentConfig` struct gains a new `logging` field that encapsulates all logging configuration. Existing `log_level` and `log_format` fields are preserved for backward compatibility.

#### New Fields

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `logging` | `LoggingConfig` | (see below) | Logging configuration section |

#### Deprecated Fields (kept for backward compat)

| Field | Type | Notes |
|-------|------|-------|
| `log_level` | `LogLevel` | Superseded by `logging.level`. If both present, `logging.level` wins. |
| `log_format` | `LogFormat` | Unchanged, still supported at top level and under `[logging]`. |

### LoggingConfig (new entity)

Represents the `[logging]` TOML table.

| Field | Type | Default | Validation | Description |
|-------|------|---------|------------|-------------|
| `destination` | `LogDestination` | `Console` | Must be `console` or `file` | Where log output is written |
| `level` | `Option<LogLevel>` | `None` | Must be valid log level | Default log level (overrides top-level `log_level` if set) |
| `format` | `Option<LogFormat>` | `None` | Must be `json` or `pretty` | Log format (overrides top-level `log_format` if set) |
| `file_path` | `Option<String>` | `None` | Required when destination=file | Absolute path to log file |
| `file_rotation` | `Option<LogRotation>` | `None` | Must be `never`, `daily`, or `hourly` | Log file rotation strategy |
| `file_max_files` | `Option<usize>` | `None` | Must be > 0 if set | Maximum number of rotated files to retain |
| `packages` | `HashMap<String, LogLevel>` | empty | Keys must be valid Rust module paths | Per-package log level overrides |

### LogDestination (new enum)

| Variant | TOML Value | Description |
|---------|------------|-------------|
| `Console` | `"console"` | Write to stderr (default, current behavior) |
| `File` | `"file"` | Write to file at `file_path` |

### LogLevel (existing, unchanged)

| Variant | TOML Value |
|---------|------------|
| `Trace` | `"trace"` |
| `Debug` | `"debug"` |
| `Info` | `"info"` |
| `Warn` | `"warn"` |
| `Error` | `"error"` |

### LogFormat (existing, unchanged)

| Variant | TOML Value |
|---------|------------|
| `Json` | `"json"` |
| `Pretty` | `"pretty"` |

## Validation Rules

1. If `destination = "file"`, then `file_path` MUST be set (non-empty string).
2. If `destination = "console"`, then `file_path`, `file_rotation`, and `file_max_files` are ignored (warning if set).
3. If `file_max_files` is set but `file_rotation` is not, a warning is logged (no rotation means no effect).
4. Per-package override keys must be non-empty strings (syntactic check only; non-existent modules are silently accepted per spec).

## Configuration Precedence

Resolved log level (highest to lowest precedence):

1. `RUST_LOG` environment variable (if set, overrides everything — existing behavior)
2. CLI `--log-level` argument
3. `HAPROXY_AGENT_LOG_LEVEL` environment variable
4. `[logging].level` in config file
5. Top-level `log_level` in config file (backward compat)
6. Default: `info`

Per-package overrides only apply when `RUST_LOG` is NOT set (since `RUST_LOG` replaces the entire filter).

## TOML Config Examples

### Minimal (backward compatible, no changes needed)

```toml
log_level = "info"
log_format = "json"
```

### Console with per-package overrides

```toml
[logging]
destination = "console"
level = "info"

[logging.packages]
"haproxy_grpc_agent::checker" = "debug"
"tonic" = "warn"
```

### File output with rotation

```toml
[logging]
destination = "file"
level = "info"
file_path = "/var/log/haproxy-grpc-agent.log"
file_rotation = "daily"
file_max_files = 5

[logging.packages]
"haproxy_grpc_agent::checker" = "debug"
```
