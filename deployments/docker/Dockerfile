# T022: Multi-stage Docker build for static binary

# Build stage
FROM rust:1.75-alpine AS builder

# Install musl-dev for static linking
RUN apk add --no-cache musl-dev

WORKDIR /build

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release --target x86_64-unknown-linux-musl && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY tests ./tests

# Build the actual application
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM scratch

# Copy the statically-linked binary
COPY --from=builder /build/target/x86_64-unknown-linux-musl/release/haproxy-grpc-agent /haproxy-grpc-agent

# Expose ports
EXPOSE 5555 9090

# Run the agent
ENTRYPOINT ["/haproxy-grpc-agent"]
