.PHONY: test clippy fmt check dev dev-app dev-api dev-legacy docker-build docker-run deploy deploy-fly setup-hooks help

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
	cd crates/app && npm run tailwind:watch & cd crates/app && $(DX) serve

docker-build:
	docker build --platform linux/amd64 -t openwok .

docker-run:
	docker run --rm -p 3000:3000 -v openwok-data:/app/data openwok

# ── Deploy ───────────────────────────────────────────────────────

deploy:
	cd crates/app && npm run tailwind:build
	cd crates/app && $(DX) bundle --platform web --release
	CARGO_PROFILE_RELEASE_STRIP=none cargo zigbuild -p openwok-app --features server --release --target x86_64-unknown-linux-musl --bin openwok-app
	find dist/container -type f -delete 2>/dev/null || true
	find dist/container -depth -type d -empty -delete 2>/dev/null || true
	mkdir -p dist/container
	cp -R $(HOME)/.cargo-target/dx/openwok-app/release/web dist/container/web
	cp $(HOME)/.cargo-target/x86_64-unknown-linux-musl/release/openwok-app dist/container/web/openwok-app
	wrangler deploy --config wrangler.containers.jsonc

deploy-fly:
	fly deploy --remote-only

bundle:
	cd crates/app && npm install && npm run tailwind:build
	cd crates/app && $(DX) bundle --web

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
