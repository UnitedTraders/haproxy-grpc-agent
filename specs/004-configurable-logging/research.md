# Research: Configurable Logging

**Feature**: 004-configurable-logging
**Created**: 2026-05-04

## Decision 1: File Output Mechanism

**Decision**: Use `tracing-appender` crate for file writing and rotation.

**Rationale**: `tracing-appender` is maintained by the `tracing` team (tokio-rs/tracing repository), integrates natively with `tracing-subscriber`, and provides:
- `RollingFileAppender` with size-based and time-based rotation
- `NonBlocking` writer wrapper that prevents I/O blocking on the async runtime
- Composable with existing `tracing_subscriber::fmt` layers

**Alternatives considered**:
- **Direct `std::fs::File` + `tracing_subscriber::fmt::writer::MakeWriterExt`**: Lower-level, requires manual rotation logic. Violates Simplicity principle.
- **`log4rs` crate**: Full logging framework with its own config format. Would conflict with existing tracing setup and add significant dependency surface.
- **`rolling-file` crate**: Third-party rotation-only crate. Less integration with tracing, additional dependency when tracing-appender already exists.

## Decision 2: Per-Package Log Level Implementation

**Decision**: Build a composite `EnvFilter` directive string from the config file's per-package overrides map.

**Rationale**: `tracing-subscriber`'s `EnvFilter` already supports directive syntax like `info,haproxy_grpc_agent::checker=debug`. The implementation will:
1. Start with the default log level as the base directive
2. Append per-package overrides as comma-separated directives
3. If `RUST_LOG` is set, use it instead (preserving existing precedence)

This approach requires zero new dependencies and uses the existing `EnvFilter` parsing that already handles the `RUST_LOG` format.

**Alternatives considered**:
- **Custom `Layer` with per-module filtering**: More complex, requires maintaining filter state. `EnvFilter` already solves this problem.
- **Multiple subscribers with different targets**: tracing only supports one global subscriber. Would require `tracing_subscriber::registry()` with filter layers — more complex than needed.

## Decision 3: Config File Structure for Logging

**Decision**: Add a `[logging]` TOML table to the existing config file format with flat fields for destination, level, file path, and rotation. Per-package overrides use a `[logging.packages]` sub-table.

**Rationale**: TOML tables provide clean namespacing. Example:

```toml
[logging]
destination = "file"         # "console" (default) or "file"
level = "info"               # default log level
file_path = "/var/log/haproxy-grpc-agent.log"
file_max_size_mb = 10        # rotation trigger (optional)
file_max_files = 5           # retention count (optional)

[logging.packages]
"haproxy_grpc_agent::checker" = "debug"
"tonic" = "warn"
```

**Migration**: Existing top-level `log_level` and `log_format` fields remain supported for backward compatibility. If both `log_level` and `[logging].level` are present, `[logging].level` takes precedence with a deprecation warning. Environment variables `HAPROXY_AGENT_LOG_LEVEL` and `HAPROXY_AGENT_LOG_FORMAT` continue to work.

**Alternatives considered**:
- **Separate logging config file**: Adds operational complexity (two files to manage). Violates Agent Pattern principle ("single config file").
- **Flat top-level fields only**: Would pollute the top-level namespace and make per-package overrides awkward (e.g., `log_package_checker_level`).

## Decision 4: Graceful Fallback on File Failure

**Decision**: If file writing fails during runtime (disk full, mount removed), the `NonBlocking` writer from `tracing-appender` will drop messages that can't be written. The agent logs an error to stderr at initialization if the file can't be opened, and exits. Runtime failures are handled by `NonBlocking`'s internal buffer overflow behavior (drops oldest messages).

**Rationale**: The agent's primary function is serving health checks, not logging. Per Constitution Principle V (Simplicity & Reliability), the agent must continue serving health checks even if logging fails. The `NonBlocking` writer already handles backpressure gracefully.

**Startup behavior**: If the log file path is invalid or unwritable at startup, the agent exits with a clear error (fail-fast). This is distinct from runtime failures where the file becomes unavailable after successful initialization.

**Alternatives considered**:
- **Dual-writer (file + stderr fallback)**: Adds complexity. `NonBlocking` already handles backpressure. Operators can monitor log file freshness externally.
- **Panic on write failure**: Violates graceful degradation requirement. Agent must keep serving.
