# Multi-stage build for OpenWok fullstack app
# Target: linux/amd64 for Cloudflare Containers

# Stage 1: Build the server binary + WASM client
FROM rust:slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN rustup target add wasm32-unknown-unknown
RUN cargo install dioxus-cli

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY migrations/ migrations/

# Build the fullstack binary (server + embedded WASM client)
RUN cd crates/app && dx build --platform fullstack --release

# Stage 2: Minimal runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/crates/app/dist/ /app/dist/
COPY migrations/ /app/migrations/

ENV DATABASE_PATH=/app/data/openwok.db
EXPOSE 3000

CMD ["/app/dist/openwok"]
