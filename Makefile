.PHONY: test clippy fmt check build-frontend build-worker deploy dev serve-desktop serve-ios setup-hooks help

DX := $(HOME)/.cargo/bin/dx
DX_OUT := $(HOME)/.cargo-target/dx/openwok-frontend/release/web/public

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all -- --check

check: test clippy fmt

tailwind:
	cd crates/frontend && npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css

tailwind-watch:
	cd crates/frontend && npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch

build-frontend: tailwind
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

serve-desktop:
	cd crates/frontend && $(DX) serve --platform desktop

serve-ios:
	cd crates/frontend && $(DX) serve --platform ios

setup-hooks:
	git config core.hooksPath .githooks
	@echo "Git hooks configured to use .githooks/"

help:
	@grep -E '^[a-z]' Makefile | sed 's/:.*//'
