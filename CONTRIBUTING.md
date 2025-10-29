# Contributing to HAProxy gRPC Agent

Thank you for considering contributing to the HAProxy gRPC Agent! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.75 or later
- Docker (for integration tests)
- Git

### Clone and Build

```bash
git clone https://github.com/unitedtraders/haproxy-grpc-agent.git
cd haproxy-grpc-agent
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test suites
cargo test --lib                           # Unit tests
cargo test --test protocol_test            # Protocol tests
cargo test --test config_test              # Configuration tests
cargo test --test logging_test -- --ignored  # Logging integration tests
```

### Code Quality

Before submitting a pull request, ensure your code passes all checks:

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Run clippy
cargo clippy --all-targets --all-features -- -D warnings

# Run all checks together
cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

## Workflow

### 1. Fork and Create a Branch

```bash
# Fork the repository on GitHub, then:
git clone https://github.com/YOUR_USERNAME/haproxy-grpc-agent.git
cd haproxy-grpc-agent
git checkout -b feature/your-feature-name
```

### 2. Make Your Changes

- Follow existing code style and patterns
- Add tests for new functionality
- Update documentation as needed
- Keep commits focused and atomic

### 3. Test Your Changes

```bash
# Run all tests
cargo test

# Test the binary locally
cargo run

# Test with Docker
docker build -t haproxy-grpc-agent:test .
docker run --rm haproxy-grpc-agent:test
```

### 4. Commit Your Changes

Follow conventional commit messages:

```bash
git commit -m "feat: add new health check option"
git commit -m "fix: resolve timeout issue in gRPC client"
git commit -m "docs: update configuration examples"
git commit -m "test: add integration tests for SSL backends"
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or updating tests
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `ci`: CI/CD changes

### 5. Push and Create Pull Request

```bash
git push origin feature/your-feature-name
```

Then create a pull request on GitHub.

## Code Structure

```
haproxy-grpc-agent/
├── src/
│   ├── main.rs         # Application entry point
│   ├── config.rs       # Configuration management
│   ├── server.rs       # TCP server for Agent Protocol
│   ├── protocol.rs     # Agent Protocol parser
│   ├── checker.rs      # gRPC health check client
│   ├── metrics.rs      # Prometheus metrics
│   └── logger.rs       # Structured logging
├── tests/
│   ├── protocol_test.rs      # Protocol parsing tests
│   ├── config_test.rs        # Configuration tests
│   ├── integration_test.rs   # End-to-end tests
│   └── logging_test.rs       # Logging tests
└── docs/               # Additional documentation
```

## Coding Standards

### Rust Guidelines

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `rustfmt` for formatting (automatically enforced by CI)
- Address all `clippy` warnings
- Prefer explicit error handling over `unwrap()` or `expect()`
- Add documentation comments for public APIs

### Testing Guidelines

- Write tests for new functionality
- Maintain or improve code coverage
- Integration tests should be marked with `#[ignore]` if they require external services
- Use descriptive test names: `test_<what>_<when>_<expected>`

### Documentation

- Update README.md for user-facing changes
- Add inline documentation for complex logic
- Update CLAUDE.md if adding new technologies

## Pull Request Process

1. **Update Documentation**: Ensure README and code comments are updated
2. **Add Tests**: All new features must include tests
3. **Pass CI Checks**: GitHub Actions must pass (formatting, clippy, tests)
4. **Request Review**: Tag maintainers for review
5. **Address Feedback**: Respond to review comments promptly
6. **Squash Commits**: Keep history clean (we'll squash on merge)

## CI/CD Pipeline

The project uses GitHub Actions for CI/CD:

### On Every Commit
- Code formatting check (`cargo fmt --check`)
- Linting (`cargo clippy`)
- Unit tests (`cargo test`)
- Protocol and config tests

### On Main Branch
- All of the above, plus:
- Docker multi-arch build
- Push to `ghcr.io/unitedtraders/haproxy-grpc-agent:latest`
- Push to `ghcr.io/unitedtraders/haproxy-grpc-agent:main-<sha>`

### On Version Tags (v*)
- All of the above, plus:
- Build release binaries for Linux (amd64, arm64) and macOS
- Create GitHub release with binary artifacts
- Tag Docker images with version number

## Release Process

Maintainers can create releases:

```bash
# Update version in Cargo.toml
# Commit changes
git commit -am "chore: bump version to 1.2.3"

# Create and push tag
git tag v1.2.3
git push origin v1.2.3

# GitHub Actions will automatically:
# - Build binaries for all platforms
# - Create GitHub release
# - Publish Docker images
```

## Getting Help

- **Questions**: Open a [Discussion](https://github.com/unitedtraders/haproxy-grpc-agent/discussions)
- **Bugs**: Open an [Issue](https://github.com/unitedtraders/haproxy-grpc-agent/issues)
- **Security**: Email security@unitedtraders.com (replace with actual email)

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Follow the project's code of conduct (if available)

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
