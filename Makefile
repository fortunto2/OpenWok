.PHONY: test clippy fmt check build-worker deploy dev

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all -- --check

check: test clippy fmt

build-worker:
	cd crates/worker && worker-build --release

deploy: build-worker
	wrangler deploy

dev:
	wrangler dev

help:
	@grep -E '^[a-z]' Makefile | sed 's/:.*//'
