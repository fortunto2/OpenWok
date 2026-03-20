# Workflow — OpenWok

## TDD Policy
**Strict** — Tests BEFORE code for all business logic. The pricing calculator (6-line receipt) is the core innovation and must be fully tested.

Write tests for:
- Pricing calculator (all fee components)
- Order flow state machine
- Zone validation and geo logic
- Federation protocol rules (baseline)
- API route handlers

Tests optional for:
- One-off simulation scripts (programs/)
- Early UI prototypes
- DevOps scripts

## Test Framework
**cargo test** — built-in Rust test framework
- Unit tests: `#[cfg(test)]` modules in each source file
- Integration tests: `tests/` directory at workspace root
- Run: `make test` or `cargo test --workspace`

## Linting & Formatting
- **Clippy:** `make clippy` — zero warnings policy (`-D warnings`)
- **Rustfmt:** `make fmt` — check formatting
- **Full check:** `make check` — runs test + clippy + fmt

## Commit Strategy
**Conventional Commits**
Format: `<type>(<scope>): <description>`
Types: feat, fix, refactor, test, docs, chore, perf, style
Scopes: core, api, frontend, pricing, order, delivery, node, infra

## Verification Checkpoints
**After each phase completion:**
1. `make test` — all pass
2. `make clippy` — no warnings
3. `make fmt` — no formatting issues
4. Manual smoke test (CLI or API call)

## Branch Strategy
- `main` — production-ready
- `feat/<track-id>` — feature branches
- `fix/<description>` — hotfixes

## Key Invariants
- Pricing breakdown must always show exactly 6 lines (Food / Delivery / Tip / Federal Fee / Local Ops Fee / Processing)
- Federal fee is always $1.00 — hardcoded, not configurable
- Processing fee is pass-through (Stripe 2.9% + $0.30)
- Restaurants get 100% food revenue, couriers get 100% delivery + tips
