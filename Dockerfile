# Build stage
FROM rust:latest AS builder
WORKDIR /app
COPY . .
RUN cargo build --release --example chat_cli

# Runtime stage
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y curl ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/examples/chat_cli /usr/local/bin/carokia-chat
ENTRYPOINT ["carokia-chat"]
