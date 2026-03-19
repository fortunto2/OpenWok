.PHONY: test clippy fmt check

test:
	cargo test --workspace

clippy:
	cargo clippy --workspace -- -D warnings

fmt:
	cargo fmt --all -- --check

check: test clippy fmt

help:
	@grep -E '^[a-z]' Makefile | sed 's/:.*//'
