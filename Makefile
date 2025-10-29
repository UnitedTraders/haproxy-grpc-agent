# T024: Makefile with build, test, and integration test targets

.PHONY: build test test-unit test-integration clean run docker-build

# Build the release binary
build:
	cargo build --release

# Build the debug binary
build-debug:
	cargo build

# Run all tests (unit + integration)
test: test-unit test-integration

# Run unit tests only
test-unit:
	cargo test --lib

# Run integration tests with docker-compose
test-integration:
	cd tests/integration && docker-compose up --build --abort-on-container-exit
	cd tests/integration && docker-compose down

# Clean build artifacts
clean:
	cargo clean
	rm -rf target/

# Run the agent locally with default config
run:
	cargo run

# Run the agent with custom config
run-config:
	cargo run -- --config config.toml

# Build Docker image
docker-build:
	docker build -t haproxy-grpc-agent:latest -f deployments/docker/Dockerfile .

# Run Docker container
docker-run:
	docker run --rm -p 5555:5555 -p 9090:9090 haproxy-grpc-agent:latest

# Format code
fmt:
	cargo fmt

# Run clippy linter
lint:
	cargo clippy -- -D warnings

# Check formatting
fmt-check:
	cargo fmt -- --check

# Full CI check (format, lint, test)
ci: fmt-check lint test-unit

# Watch and rebuild on file changes (requires cargo-watch)
watch:
	cargo watch -x run
