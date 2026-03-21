# Multi-stage build for OpenWok fullstack app (Dioxus 0.7)
# Uses cargo-chef for dependency caching

FROM rust:1 AS chef
RUN cargo install cargo-chef
WORKDIR /app

# Stage 1: Plan dependencies
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 2: Build
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .

# Install dx CLI via cargo-binstall (faster than cargo install)
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

# WASM target for client build
RUN rustup target add wasm32-unknown-unknown

# Bundle fullstack app (server binary + WASM client)
RUN cd crates/app && dx bundle --web

# Stage 3: Minimal runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/dx/openwok-app/release/web/ /app/

ENV PORT=3000
ENV IP=0.0.0.0
ENV DATABASE_PATH=/app/data/openwok.db

EXPOSE 3000

WORKDIR /app
ENTRYPOINT ["/app/openwok-app"]
