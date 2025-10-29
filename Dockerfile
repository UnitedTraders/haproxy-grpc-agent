# Multi-stage build for haproxy-grpc-agent
FROM rust:1.90 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src src

# Build with release optimizations
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/haproxy-grpc-agent /usr/local/bin/haproxy-grpc-agent

# Default configuration
ENV AGENT_SERVER_BIND_ADDRESS=0.0.0.0
ENV AGENT_SERVER_PORT=5555
ENV AGENT_METRICS_PORT=9090
ENV AGENT_LOG_LEVEL=info
ENV AGENT_LOG_FORMAT=json

EXPOSE 5555 9090

CMD ["haproxy-grpc-agent"]
