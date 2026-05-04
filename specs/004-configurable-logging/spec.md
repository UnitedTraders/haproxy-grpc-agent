# Feature Specification: Configurable Logging

**Feature Branch**: `004-configurable-logging`
**Created**: 2026-05-04
**Status**: Draft
**Input**: User description: "Current implementation writes log to console only. Please add feature for user to choose (1) write to console (2) write into file. User should be able to set default log level and specify log levels per package. Log destination should be configurable with config file."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Configure Log Destination via Config File (Priority: P1)

An operator deploying haproxy-grpc-agent wants to redirect log output from the console to a file so that logs persist across restarts and can be collected by external log aggregation tools. The operator edits the TOML configuration file, sets the log destination to "file", specifies a file path, and restarts the agent. All log output now goes to the specified file instead of the console.

**Why this priority**: Log destination is the core ask — without it, the operator has no way to persist logs outside of console output. This is the most fundamental change.

**Independent Test**: Can be fully tested by starting the agent with a config file that sets `log_destination = "file"` and `log_file_path = "/tmp/agent.log"`, then verifying log output appears in the file and not on the console.

**Acceptance Scenarios**:

1. **Given** a config file with `log_destination = "file"` and a valid `log_file_path`, **When** the agent starts, **Then** all log output is written to the specified file and nothing is written to the console.
2. **Given** a config file with `log_destination = "console"`, **When** the agent starts, **Then** all log output is written to stderr (current behavior).
3. **Given** a config file with `log_destination = "file"` but no `log_file_path`, **When** the agent starts, **Then** the agent exits with a clear error message indicating the missing path.
4. **Given** a config file with `log_destination = "file"` and a `log_file_path` pointing to a non-writable location, **When** the agent starts, **Then** the agent exits with a clear error message indicating the path is not writable.

---

### User Story 2 - Set Default Log Level (Priority: P1)

An operator wants to control the verbosity of log output across the entire application. The operator sets a default log level (e.g., `warn`) in the configuration file. After restarting the agent, only messages at `warn` level and above appear in the output.

**Why this priority**: Log level control is equally fundamental — operators need to reduce noise in production and increase verbosity for debugging. This works together with Story 1 as the minimum viable logging configuration.

**Independent Test**: Can be fully tested by starting the agent with `log_level = "warn"` in the config file and verifying that `info` and `debug` messages are suppressed while `warn` and `error` messages appear.

**Acceptance Scenarios**:

1. **Given** a config file with `log_level = "debug"`, **When** the agent starts, **Then** messages at all levels (debug, info, warn, error) appear in the output.
2. **Given** a config file with `log_level = "error"`, **When** the agent starts, **Then** only error-level messages appear in the output.
3. **Given** no `log_level` set in the config file, **When** the agent starts, **Then** the default level is `info` (preserving current behavior).

---

### User Story 3 - Per-Package Log Level Overrides (Priority: P2)

An operator is debugging a gRPC connectivity issue and needs detailed logs from the health checker module without being overwhelmed by debug output from all other modules. The operator adds per-package log level overrides in the configuration file, setting the health checker package to `debug` while keeping everything else at `info`. After restarting, only the targeted module produces debug output.

**Why this priority**: Per-package overrides are a power-user feature that builds on the default log level. It delivers significant value for debugging but is not required for basic logging functionality.

**Independent Test**: Can be fully tested by setting `log_level = "info"` as default and adding a per-package override like `haproxy_grpc_agent::checker = "debug"`, then verifying that debug messages appear only from the checker module.

**Acceptance Scenarios**:

1. **Given** a default `log_level = "info"` and a per-package override `haproxy_grpc_agent::checker = "debug"`, **When** the agent runs and the checker module emits a debug message, **Then** that debug message appears in the output.
2. **Given** a default `log_level = "info"` and a per-package override `haproxy_grpc_agent::checker = "debug"`, **When** another module emits a debug message, **Then** that debug message is suppressed.
3. **Given** per-package overrides in the config file, **When** the `RUST_LOG` environment variable is also set, **Then** the environment variable takes precedence over config file settings (preserving existing override behavior).

---

### User Story 4 - Log File Rotation (Priority: P3)

An operator using file-based logging wants to prevent log files from consuming all available disk space. The operator configures a maximum log file size in the configuration file. When the log file reaches the configured size, it is rotated automatically.

**Why this priority**: File rotation is an important operational concern but can be deferred — operators can use external tools (logrotate) as a workaround in the interim.

**Independent Test**: Can be fully tested by setting a small `log_file_max_size` (e.g., 1 KB for testing), generating enough log output to exceed it, and verifying that the file is rotated.

**Acceptance Scenarios**:

1. **Given** file logging with `log_file_max_size = "10MB"`, **When** the log file exceeds 10 MB, **Then** the current file is rotated and a new log file is started.
2. **Given** file logging with `log_file_max_files = 5`, **When** more than 5 rotated files exist, **Then** the oldest rotated file is deleted.
3. **Given** file logging without rotation settings, **When** the agent runs, **Then** the log file grows without limit (no rotation by default).

### Edge Cases

- What happens when the log file path becomes unavailable during runtime (e.g., disk full, mount removed)?
  - The agent should log an error to stderr as a fallback and continue operating.
- What happens when an invalid log level string is provided in the config file?
  - The agent should exit with a clear error message listing valid log levels.
- What happens when a per-package override references a non-existent module?
  - The override is silently accepted (no error) since modules may be conditionally compiled or loaded.
- What happens when both the config file and CLI `--log-level` argument are specified?
  - CLI arguments override config file values (existing precedence: CLI > env > config > defaults).

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST support writing log output to the console (stderr), preserving current behavior as the default.
- **FR-002**: System MUST support writing log output to a file at a user-specified path.
- **FR-003**: System MUST allow the user to select the log destination (console or file) via the configuration file.
- **FR-004**: System MUST allow the user to set a default log level (trace, debug, info, warn, error) via the configuration file.
- **FR-005**: System MUST allow the user to specify per-package log level overrides via the configuration file.
- **FR-006**: System MUST preserve the existing configuration precedence: CLI arguments override environment variables, which override config file values, which override defaults.
- **FR-007**: System MUST validate log configuration at startup and exit with a clear error message if the configuration is invalid (e.g., file destination without a path, unrecognized log level).
- **FR-008**: System MUST support log file rotation with configurable maximum file size and maximum number of retained files.
- **FR-009**: System MUST continue operating if the log file becomes unavailable during runtime, falling back to stderr output with an error message.
- **FR-010**: System MUST accept the log destination, default level, and per-package overrides in the TOML configuration file format already used by the application.

### Key Entities

- **Log Destination**: Where log output is written — either `console` (stderr) or `file` (specified path). Only one destination is active at a time.
- **Log Level**: The minimum severity threshold for log messages — one of `trace`, `debug`, `info`, `warn`, `error`. Applied globally as a default.
- **Per-Package Override**: A mapping from a Rust module path (e.g., `haproxy_grpc_agent::checker`) to a log level, allowing targeted verbosity for specific modules.
- **Log Rotation Policy**: Configuration controlling automatic log file rotation — maximum file size and maximum number of retained rotated files.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can switch log output between console and file by changing a single configuration value and restarting the agent.
- **SC-002**: Operators can control log verbosity at both the application-wide and per-module level without modifying source code.
- **SC-003**: Log file size remains bounded when rotation is configured, with no manual intervention required.
- **SC-004**: All existing logging behavior is preserved when no new configuration options are specified (100% backward compatibility).

## Clarifications

### Session 2026-05-04

- Q: Should the system support both log destinations simultaneously (console AND file), or only one at a time? → A: Single destination only (console OR file). Operators needing both can use shell redirection as a workaround.

## Assumptions

- Operators have filesystem write access to the configured log file path.
- The TOML configuration file is the primary mechanism for log configuration; the existing CLI argument and `RUST_LOG` environment variable override behavior is preserved.
- Log rotation (P3) can be deferred to a later iteration if needed — operators can use OS-level tools (e.g., `logrotate`) as a temporary workaround.
- Only one log destination is active at a time (console OR file, not both simultaneously).
- The existing JSON log format is preserved regardless of destination.
