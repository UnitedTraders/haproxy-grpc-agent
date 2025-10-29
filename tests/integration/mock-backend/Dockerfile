# T051: Dockerfile for mock gRPC backend
FROM rust:1.90 as builder

WORKDIR /app
COPY Cargo.toml .
COPY src src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/mock-grpc-backend /usr/local/bin/mock-grpc-backend

EXPOSE 50051

CMD ["mock-grpc-backend"]
