# Implementation Plan: Repository Abstraction

**Track ID:** repo-abstraction_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [ ] Not Started

## Overview

Extract duplicated SQL logic behind a Repository trait, share route handlers between api and worker via a handlers crate. TDD: write Repository trait tests first, then implement.

## Phase 1: Repository Trait <!-- checkpoint:c9abea7 -->

Define the data access abstraction in core. TDD: write trait + error types, add test helpers.

### Tasks

- [x] Task 1.1: Create `crates/core/src/repo.rs` <!-- sha:c9abea7 --> — define `Repository` trait with `#[async_trait]` and `RepoError` enum. Methods: `list_restaurants`, `get_restaurant`, `create_restaurant`, `list_orders`, `get_order`, `create_order`, `update_order_status`, `assign_courier`, `list_couriers`, `create_courier`, `toggle_courier_available`, `get_economics`, `get_metrics`. Export from `crates/core/src/lib.rs`. Add `async-trait` to workspace deps.
- [x] Task 1.2: Add `RepoError` variants <!-- sha:c9abea7 -->: `NotFound`, `InvalidTransition`, `Conflict`, `Internal(String)`. Derive `Debug`, impl `Display` + `std::error::Error` via `thiserror`.

### Verification

- [x] `cargo check -p openwok-core` compiles
- [x] Repository trait is exported from `openwok_core`

## Phase 2: SqliteRepo + API Refactor <!-- checkpoint:e7532eb -->

Implement SqliteRepo and refactor api handlers to use Repository trait. Existing tests must pass throughout.

### Tasks

- [x] Task 2.1: Create `crates/api/src/sqlite_repo.rs` <!-- sha:1f3e452 --> — `SqliteRepo` struct wrapping `Arc<Mutex<rusqlite::Connection>>`. Implement all `Repository` trait methods by extracting SQL queries from existing `routes/*.rs` files. Keep exact same SQL and row-mapping logic.
- [x] Task 2.2: Write unit tests for `SqliteRepo` <!-- sha:6f5c2f0 --> — test each Repository method against an in-memory SQLite database with seeded test data. At minimum: list_restaurants (returns seeded), create_order (returns pricing breakdown), update_order_status (valid + invalid transitions), get_economics (aggregation).
- [x] Task 2.3: Create `crates/handlers/` crate <!-- sha:a1b57bd --> (`openwok-handlers`). Add to workspace members. Dependencies: `openwok-core`, `axum` (workspace), `serde`, `serde_json`, `uuid`, `chrono`, `async-trait`. Move route handler functions from `crates/api/src/routes/{restaurants,orders,couriers,economics,metrics}.rs` into handlers — make them generic: `async fn list_restaurants<R: Repository>(State(repo): State<Arc<R>>) -> ...`. Export `pub fn api_routes<R>() -> Router<Arc<R>>` that builds the shared router.
- [x] Task 2.4: Refactor `crates/api/` <!-- sha:59ac51f --> — depend on `openwok-handlers`. In `main.rs`: create `SqliteRepo`, wrap in `Arc`, call `openwok_handlers::api_routes::<SqliteRepo>()`, merge with WebSocket route (stays in api). Remove old route modules (restaurants.rs, orders.rs, couriers.rs, economics.rs, metrics.rs) — keep only ws.rs and db.rs.
- [x] Task 2.5: Run `make check` <!-- sha:e7532eb --> — all 37+ tests pass, clippy clean, fmt clean. Fix any compilation or test issues.

### Verification

- [x] `cargo test --workspace` passes with no regressions
- [x] `cargo run -p openwok-api` starts, all 12 endpoints work (curl smoke test)
- [x] WebSocket endpoint still works for order tracking

## Phase 3: D1Repo + Worker Refactor

Implement D1Repo and wire worker to use shared handlers.

### Tasks

- [x] Task 3.1: Create `crates/worker/src/d1_repo.rs` <!-- sha:b7cd3f8 --> — `D1Repo` struct wrapping `D1Database`. Implement all `Repository` trait methods by extracting D1 queries from current `lib.rs`. Same SQL as SqliteRepo but via D1 prepared statement API (`prepare().bind()?.all()`).
- [x] Task 3.2: Update `crates/worker/Cargo.toml` — verified wasm32 compilation. Note: handlers crate can't be shared with worker (axum requires Send, D1Database is !Send). D1Repo provides the same abstraction; worker uses worker::Router for routing.
- [x] Task 3.3: Rewrite `crates/worker/src/lib.rs` — replace 854 lines of inline handlers with: create `D1Repo` from env, wrap in `Arc`, call `openwok_handlers::api_routes::<D1Repo>()`, dispatch request. Keep seed-on-first-request logic. Target: ~50-80 lines.
- [ ] Task 3.4: Build worker (`make build-worker`) and deploy (`wrangler deploy`). Verify live URL: `/api/health` returns 200, `/api/restaurants` returns data, order flow works.

### Verification

- [ ] `make build-worker` succeeds (wasm32 compilation)
- [ ] `wrangler dev` serves API locally via D1 local mode
- [ ] Live URL returns same responses as before deployment

## Phase 4: Docs & Cleanup

### Tasks

- [ ] Task 4.1: Remove orphaned Fly.io files: `Dockerfile`, `fly.toml`, `.dockerignore`. Verify `git status` shows them as deleted.
- [ ] Task 4.2: Update `CLAUDE.md` — add `handlers` crate to workspace structure, document Repository pattern, update dependency diagram.
- [ ] Task 4.3: Run `make check` — tests pass, clippy clean, fmt clean. Verify no dead code or unused imports.

### Verification

- [ ] CLAUDE.md reflects current project state (4 crates + handlers)
- [ ] Linter clean, tests pass
- [ ] No orphaned files

## Final Verification

- [ ] All acceptance criteria from spec met
- [ ] Tests pass (`make check`)
- [ ] Clippy clean
- [ ] Worker builds and deploys successfully
- [ ] Live URL working (API + SPA)
- [ ] Documentation up to date

## Context Handoff

_Summary for /build to load at session start — keeps context compact._

### Session Intent

Eliminate route handler duplication between api (rusqlite) and worker (D1) via Repository trait + shared handlers crate.

### Key Files

**Create:**
- `crates/core/src/repo.rs` — Repository trait + RepoError
- `crates/handlers/` — shared axum route handlers (new crate)
- `crates/api/src/sqlite_repo.rs` — SqliteRepo implementation
- `crates/worker/src/d1_repo.rs` — D1Repo implementation

**Modify:**
- `crates/core/src/lib.rs` — export repo module
- `crates/core/Cargo.toml` — add async-trait
- `crates/api/src/main.rs` — use handlers crate + SqliteRepo
- `crates/api/Cargo.toml` — depend on handlers
- `crates/worker/src/lib.rs` — rewrite to use handlers crate + D1Repo (854→~60 lines)
- `crates/worker/Cargo.toml` — depend on handlers
- `Cargo.toml` (workspace) — add handlers to members
- `CLAUDE.md` — update workspace structure

**Delete:**
- `crates/api/src/routes/restaurants.rs` — logic moves to handlers + SqliteRepo
- `crates/api/src/routes/orders.rs` — logic moves to handlers + SqliteRepo
- `crates/api/src/routes/couriers.rs` — logic moves to handlers + SqliteRepo
- `crates/api/src/routes/economics.rs` — logic moves to handlers + SqliteRepo
- `crates/api/src/routes/metrics.rs` — logic moves to handlers + SqliteRepo
- `Dockerfile`, `fly.toml`, `.dockerignore` — orphaned Fly.io files

### Decisions Made

- **Handlers crate over duplication:** Shared handlers crate (generic over Repository) avoids maintaining two copies of every route handler. Worth the extra crate.
- **async-trait over native:** Use `async-trait` crate for Repository trait — native async-in-trait doesn't support dyn dispatch, and async-trait is well-tested with wasm32.
- **State pattern:** `State<Arc<R>>` in handlers, api wraps in AppState with FromRef for WebSocket broadcast channel.
- **WebSocket stays in api:** CF Workers doesn't support traditional WebSocket (needs Durable Objects). WS route added by api only, not in shared handlers.

### Risks

- **wasm32 compilation of handlers crate:** axum must compile for wasm32-unknown-unknown. The `worker` crate's axum feature handles this, but handlers depending directly on `axum` (workspace version) may need feature gating. Mitigation: test `cargo check --target wasm32-unknown-unknown -p openwok-handlers` early in Phase 2.
- **D1 API differences:** D1 returns `serde_json::Value` rows while rusqlite uses typed row accessors. Repository trait returns domain types — each implementation maps differently. Risk: subtle behavior differences. Mitigation: same SQL in both, integration test on both paths.
- **Worker standalone workspace:** handlers crate in main workspace, worker outside it. Path dependency works but cargo commands (test, clippy) don't cover worker. Mitigation: explicit `cd crates/worker && cargo check` in verification steps.

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
