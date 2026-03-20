FROM rust:1.93-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/core crates/core
COPY crates/api crates/api
# Stub frontend crate so workspace resolves
RUN mkdir -p crates/frontend/src && echo "" > crates/frontend/src/lib.rs
COPY crates/frontend/Cargo.toml crates/frontend/Cargo.toml

RUN cargo build --release -p openwok-api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/openwok-api /usr/local/bin/openwok-api
COPY migrations/ /app/migrations/

ENV DATABASE_PATH=/data/openwok.db

EXPOSE 3000

CMD ["openwok-api"]
