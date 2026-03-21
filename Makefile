.PHONY: test clippy fmt check dev dev-app dev-api dev-legacy docker-build setup-hooks help

DX := $(HOME)/.cargo/bin/dx

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all -- --check

check: test clippy fmt

# ── Fullstack (new) ──────────────────────────────────────────────

dev-app:
	cd crates/app && $(DX) serve

dev:
	cd crates/app && $(DX) serve

docker-build:
	docker build --platform linux/amd64 -t openwok .

docker-run:
	docker run --rm -p 3000:3000 -v openwok-data:/app/data openwok

# ── Legacy (SPA + API, kept until Container deploy verified) ─────

dev-api:
	DATABASE_PATH=data/openwok.db cargo run -p openwok-api

dev-legacy:
	cd crates/frontend && $(DX) serve --platform web

setup-hooks:
	git config core.hooksPath .githooks
	@echo "Git hooks configured to use .githooks/"

help:
	@grep -E '^[a-z]' Makefile | sed 's/:.*//'
