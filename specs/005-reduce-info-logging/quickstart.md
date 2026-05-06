# Quickstart: Reduce Info-Level Logging

## What Changed

The default `info` log level now shows only important lifecycle events:
- Agent startup (configuration summary)
- Server listening
- Initialization complete
- Shutdown signals and completion

Per-connection and per-request events (connection established/closed, health check processing/completed) are now logged at `debug` level.

## Usage

**Production (default — quiet logs):**
```toml
log_level = "info"
```
Output: startup, ready, shutdown messages only.

**Troubleshooting (verbose logs):**
```toml
log_level = "debug"
```
Output: all per-connection and per-request events, same as the previous `info` behavior.

**Per-module debugging:**
```toml
log_level = "info"

[logging.packages]
haproxy_grpc_agent__server = "debug"
```
Output: info-level for everything, plus debug-level for the server module (connections and health checks).
