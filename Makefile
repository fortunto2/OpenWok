.PHONY: test clippy fmt check build-frontend build-worker deploy dev help

DX := $(HOME)/.cargo/bin/dx
DX_OUT := $(HOME)/.cargo-target/dx/openwok-frontend/release/web/public

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all -- --check

check: test clippy fmt

build-frontend:
	cd crates/frontend && $(DX) build --platform web --release
	rm -rf public/*
	cp -r $(DX_OUT)/* public/

build-worker:
	cd crates/worker && worker-build --release

build: build-frontend build-worker

deploy: build
	wrangler deploy

dev:
	wrangler dev

dev-frontend:
	cd crates/frontend && $(DX) serve

dev-api:
	cargo run -p openwok-api

dev-full:
	@echo "Starting API (port 3000) and Frontend (port 8080)..."
	@cargo run -p openwok-api &
	@sleep 2 && cd crates/frontend && $(DX) serve
	@trap 'kill %1' EXIT

help:
	@grep -E '^[a-z]' Makefile | sed 's/:.*//'
