# Specification: Repository Abstraction

**Track ID:** repo-abstraction_20260320
**Type:** Refactor
**Created:** 2026-03-20
**Status:** Draft

## Summary

Extract duplicated database logic from `crates/api/` (rusqlite) and `crates/worker/` (D1) behind a shared `Repository` trait in `crates/core/`. Create a `crates/handlers/` crate with axum route handlers generic over Repository, so both the local axum server and the Cloudflare Worker share the same handler code.

Currently, `crates/worker/src/lib.rs` (854 lines) duplicates every route handler from `crates/api/src/routes/` with its own inline SQL, row mapping, and request/response logic. Any new endpoint (Phase 6: auth, payments) would need to be written twice. This refactor eliminates that duplication.

This is the remaining architectural work from the cf-workers-deploy track (archived at 0% — process defect per retro). Also addresses retro recommendation #6: remove orphaned Fly.io files.

## Acceptance Criteria

- [ ] `Repository` trait defined in `crates/core/src/repo.rs` with async methods for all entity operations
- [ ] `SqliteRepo` implements Repository in `crates/api/` — wraps existing rusqlite queries
- [ ] `D1Repo` implements Repository in `crates/worker/` — wraps existing D1 queries
- [ ] `crates/handlers/` crate exists with shared axum route handlers generic over `Repository`
- [ ] `crates/api/` uses handlers crate (+ WebSocket route locally)
- [ ] `crates/worker/` uses handlers crate (+ D1Repo)
- [ ] All existing tests pass (`make check`) — no regressions
- [ ] `wrangler deploy` succeeds, live URL returns HTTP 200
- [ ] Orphaned Fly.io files removed (Dockerfile, fly.toml, .dockerignore)

## Dependencies

- `async-trait` crate (for async methods in dyn-safe Repository trait)
- Existing: axum 0.8, rusqlite, worker 0.7, openwok-core

## Out of Scope

- New API endpoints (Phase 6)
- Auth/payments integration
- WebSocket in Worker (requires Durable Objects)
- CI/CD pipeline
- Frontend changes (no handler signatures change)

## Technical Notes

- **Architecture:** `core` (Repository trait) ← `handlers` (axum handlers, generic over R: Repository) ← `api` (SqliteRepo + WS) / `worker` (D1Repo)
- **Worker is standalone workspace:** Not in main workspace due to wasm32 target. Already depends on core via path — will also depend on handlers via path.
- **wasm32 compatibility:** handlers crate must compile for wasm32-unknown-unknown. Depends only on core + axum + serde (all wasm32-safe). No tokio runtime, no rusqlite.
- **State pattern:** Handlers use `State<Arc<R>>` where `R: Repository`. API wraps in AppState (adds broadcast channel for WS). Worker passes `Arc<D1Repo>` directly.
- **async-trait vs native:** Use `async-trait` crate for Repository trait to support `dyn` dispatch and wasm32 compatibility. Native async-in-trait (Rust 1.75+) doesn't support dyn dispatch without boxing.
- **Economics/metrics handlers** have complex aggregation SQL — these stay as Repository methods, not separate queries.
