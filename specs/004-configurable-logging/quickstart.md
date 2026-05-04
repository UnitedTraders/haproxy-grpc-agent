# Quickstart: Configurable Logging

**Feature**: 004-configurable-logging

## Usage

### 1. Console Logging (default, no config changes needed)

```bash
# Existing behavior — logs to stderr in JSON format at INFO level
./haproxy-grpc-agent --config config.toml
```

### 2. File Logging

Add a `[logging]` section to your TOML config file:

```toml
# config.toml
server_port = 5555

[logging]
destination = "file"
level = "info"
file_path = "/var/log/haproxy-grpc-agent.log"
```

```bash
./haproxy-grpc-agent --config config.toml
# Logs now write to /var/log/haproxy-grpc-agent.log
```

### 3. Per-Package Log Levels

Debug a specific module while keeping other output quiet:

```toml
[logging]
destination = "console"
level = "warn"

[logging.packages]
"haproxy_grpc_agent::checker" = "debug"
```

### 4. File Logging with Rotation

Prevent log files from growing without bound:

```toml
[logging]
destination = "file"
level = "info"
file_path = "/var/log/haproxy-grpc-agent.log"
file_rotation = "daily"
file_max_files = 5
```

### 5. Override via Environment Variable

The `RUST_LOG` environment variable still takes precedence over all config:

```bash
RUST_LOG=debug ./haproxy-grpc-agent --config config.toml
```

### 6. Override via CLI

```bash
./haproxy-grpc-agent --config config.toml --log-level debug
```

## Verification

### Verify file logging works

```bash
# Start agent with file logging config
./haproxy-grpc-agent --config config.toml

# In another terminal, check log file exists and has content
tail -f /var/log/haproxy-grpc-agent.log
```

### Verify per-package overrides

```bash
# With config setting checker to debug and default to warn:
# Only checker module debug messages should appear
./haproxy-grpc-agent --config config.toml 2>&1 | grep -c '"level":"DEBUG"'
```

### Verify backward compatibility

```bash
# Old-style config with no [logging] section still works
echo 'log_level = "debug"' > old-config.toml
echo 'server_port = 5555' >> old-config.toml
./haproxy-grpc-agent --config old-config.toml
# Should log to console at debug level (same as before)
```
