# haproxy-grpc-agent Development Guidelines

Auto-generated from all feature plans. Last updated: 2025-10-28

## Active Technologies
- Rust 1.75+ (edition 2024) + estcontainers 0.27 (new), tokio 1.x, tonic 0.14, existing mock-grpc-backend Docker image (002-testcontainers-integration)
- Rust 1.75+ (edition 2024) + okio 1.x, tonic 0.14.2, dashmap 6.1.0, clap 4.5, serde 1.0, toml 0.9.8, prometheus 0.14 (003-channel-cache-config)
- N/A (in-memory DashMap cache, no persistence) (003-channel-cache-config)

- Rust 1.75+ + okio (async runtime), tonic (gRPC client for backend checks), prometheus (metrics), serde (config/logging) (001-core-agent)

## Project Structure

```text
src/
tests/
```

## Commands

cargo test [ONLY COMMANDS FOR ACTIVE TECHNOLOGIES][ONLY COMMANDS FOR ACTIVE TECHNOLOGIES] cargo clippy

## Code Style

Rust 1.75+: Follow standard conventions

## Recent Changes
- 003-channel-cache-config: Added Rust 1.75+ (edition 2024) + okio 1.x, tonic 0.14.2, dashmap 6.1.0, clap 4.5, serde 1.0, toml 0.9.8, prometheus 0.14
- 002-testcontainers-integration: Added Rust 1.75+ (edition 2024) + estcontainers 0.27 (new), tokio 1.x, tonic 0.14, existing mock-grpc-backend Docker image

- 001-core-agent: Added Rust 1.75+ + okio (async runtime), tonic (gRPC client for backend checks), prometheus (metrics), serde (config/logging)

<!-- MANUAL ADDITIONS START -->
<!-- MANUAL ADDITIONS END -->
