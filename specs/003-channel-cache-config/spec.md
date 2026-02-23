# Feature Specification: Configurable Channel Cache

**Feature Branch**: `003-channel-cache-config`
**Created**: 2026-02-23
**Status**: Draft
**Input**: User description: "Current implementation of GrpcHealthChecker has internal cache of channels. Please make is configurable so the user may disable caching with config option"

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Disable Channel Caching via Configuration (Priority: P1)

As an operator deploying the HAProxy gRPC agent, I want to disable the internal gRPC channel cache so that every health check creates a fresh connection to the backend. This is useful in environments where cached connections may become stale silently (e.g., behind load balancers that silently drop idle connections) or during debugging when I want to verify that each check independently establishes connectivity.

**Why this priority**: This is the core request — the ability to toggle channel caching on or off. Without this, the feature has no value.

**Independent Test**: Can be fully tested by setting the cache-disable option, running multiple health checks against the same backend, and verifying that a new connection is established for each check (no channel reuse).

**Acceptance Scenarios**:

1. **Given** the configuration has channel caching disabled, **When** the agent performs two consecutive health checks to the same backend, **Then** each check creates a new gRPC channel (no channel is retrieved from cache).
2. **Given** the configuration has channel caching disabled, **When** the agent starts up, **Then** no channel cache data structure is populated during operation.
3. **Given** the configuration has channel caching enabled (default), **When** the agent performs two consecutive health checks to the same backend, **Then** the second check reuses the channel created by the first check (existing behavior preserved).

---

### User Story 2 - Configure Channel Caching via All Supported Methods (Priority: P2)

As an operator, I want to control channel caching through any of the existing configuration methods (environment variable, config file, or CLI argument) so that I can use whichever method fits my deployment workflow.

**Why this priority**: Consistency with existing configuration patterns ensures operators don't need to learn new mechanisms. Less critical than the core toggle itself.

**Independent Test**: Can be tested by setting the cache option via each configuration method independently and verifying the agent respects the setting in each case.

**Acceptance Scenarios**:

1. **Given** the environment variable for channel caching is set to disabled, **When** the agent starts, **Then** channel caching is disabled.
2. **Given** the TOML config file has channel caching set to disabled, **When** the agent starts, **Then** channel caching is disabled.
3. **Given** a CLI argument disables channel caching, **When** the agent starts, **Then** channel caching is disabled regardless of other configuration sources.
4. **Given** no configuration is provided for channel caching, **When** the agent starts, **Then** channel caching is enabled (backward-compatible default).

---

### Edge Cases

- What happens when channel caching is disabled and the backend is under high load? Each request creates a new connection, which may increase connection overhead. This is expected and acceptable — the operator has explicitly chosen this trade-off.
- What happens to the active channels metric when caching is disabled? The metric should consistently report 0 cached channels.
- What happens if the configuration is changed from cached to uncached (or vice versa) between restarts? The agent should respect the current configuration at startup; there is no hot-reload requirement.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST provide a configuration option to enable or disable gRPC channel caching.
- **FR-002**: Channel caching MUST be enabled by default to preserve backward compatibility with existing deployments.
- **FR-003**: The channel caching option MUST be configurable via environment variable, TOML config file, and CLI argument, following the existing configuration precedence (env < config file < CLI).
- **FR-004**: When channel caching is disabled, the system MUST create a new gRPC channel for every health check request.
- **FR-005**: When channel caching is disabled, the cached channels metric MUST report 0.
- **FR-006**: When channel caching is enabled, the system MUST behave identically to the current implementation (channels are cached and reused, with readiness validation).

### Key Entities

- **Channel Cache Setting**: A boolean configuration value indicating whether gRPC channels should be cached and reused across health checks or created fresh for each request.

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Operators can disable channel caching with a single configuration change and observe that every health check creates a fresh connection.
- **SC-002**: Enabling channel caching (default) produces identical behavior to the current system with no regression in check latency or connection reuse.
- **SC-003**: The configuration option is available through all three existing configuration methods (environment variable, config file, CLI argument).
- **SC-004**: The active channels metric accurately reflects 0 cached channels when caching is disabled.

## Assumptions

- The configuration is read at startup and does not change during runtime (no hot-reload).
- Disabling caching may increase connection overhead and latency per check; this is an accepted trade-off chosen by the operator.
- The existing configuration precedence (environment variable < config file < CLI argument) applies to this new option as well.
- No additional cache eviction strategies (e.g., TTL, max size) are in scope for this feature; this is strictly an on/off toggle.
