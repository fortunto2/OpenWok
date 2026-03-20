# Specification: Frontend Module Split + Pre-commit Hooks

**Track ID:** frontend-split_20260320
**Type:** Refactor
**Created:** 2026-03-20
**Status:** Draft

## Summary

`crates/frontend/src/main.rs` is 2060 lines — a single-file monolith containing all 14 page components, route definitions, auth/cart state, API helpers, analytics, and data fetchers. This violates the project's "Module > 1000 lines → split" threshold and has been flagged in **3 consecutive retros** (retro #5, #6, #7) as a recurring recommendation.

Split into a clean module structure: `state.rs`, `analytics.rs`, `api.rs`, `app.rs`, and a `pages/` directory with one file per page group. Also add pre-commit hooks (`cargo fmt --check && cargo clippy --all`) — another recurring retro recommendation (3 retros) that would have prevented the auth-payments redo cycle.

## Acceptance Criteria

- [x] `main.rs` is under 30 lines (mod declarations + `fn main()`) — 11 lines
- [x] No single file exceeds 500 lines — max 444 (owner.rs)
- [x] All 14 frontend routes render correctly (same behavior as before) — all routes wired in app.rs, compiles clean
- [x] `cargo build -p openwok-frontend` succeeds with zero warnings
- [x] `cargo clippy --all` passes
- [x] `cargo fmt --check` passes
- [x] `cargo test --workspace` passes (101 tests, 0 failures)
- [x] Pre-commit hook exists at `.githooks/pre-commit` with fmt + clippy
- [x] `core.hooksPath` set to `.githooks` (committed, portable)
- [x] `dx serve` from `crates/frontend/` starts and renders pages — build verified, visual check N/A (no browser tools in CI)

## Dependencies

- None (pure refactor, no new crates or dependencies)

## Out of Scope

- New frontend features or pages
- Component library extraction
- CSS refactoring
- Test additions for frontend (WASM testing is complex, defer)

## Technical Notes

- Dioxus `#[component]` functions are standalone — they can be moved to separate files with minimal import changes
- `Route` enum uses `#[derive(Routable)]` — must stay in scope of all component functions (use `pub` + re-export)
- `Signal<CartState>` and `Signal<UserState>` are context-provided — components access via `use_context`, no prop drilling needed
- PostHog snippet is a `const &str` — move to analytics module, reference from App
- API helpers (`api_get`, `api_post_json`, `api_patch_json`) are generic async fns — clean module boundary
- Pre-commit hooks: use `.githooks/` directory (committed to repo) instead of `.git/hooks/` (not tracked by git)
