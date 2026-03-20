FROM rust:1.86-slim AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/

RUN cargo build --release -p openwok-api

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/openwok-api /usr/local/bin/openwok-api

EXPOSE 3000

CMD ["openwok-api"]
