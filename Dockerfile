# Multi-stage build for OpenWok fullstack app (Dioxus 0.7)
# Builds server binary + WASM client in a single dx bundle step

FROM rust:1 AS builder
WORKDIR /app

# Install dx CLI via cargo-binstall (pre-compiled binary, fast)
RUN curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash
RUN cargo binstall dioxus-cli --root /.cargo -y --force
ENV PATH="/.cargo/bin:$PATH"

# WASM target for client build
RUN rustup target add wasm32-unknown-unknown

# Install Node.js for Tailwind
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - && apt-get install -y nodejs

# Copy source and build
COPY . .
RUN cd crates/app && npm install && npm run tailwind:build && dx bundle --release --web

# Minimal runtime
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/dx/openwok-app/release/web/ /app/

ENV PORT=3000
ENV IP=0.0.0.0
ENV DATABASE_PATH=/app/data/openwok.db

EXPOSE 3000

WORKDIR /app
ENTRYPOINT ["/app/openwok-app"]
