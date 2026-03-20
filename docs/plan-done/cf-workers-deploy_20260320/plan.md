# Implementation Plan: Cloudflare Workers Deployment

**Track ID:** cf-workers-deploy_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [ ] Not Started

## Overview

Extract DB-coupled handlers into a generic Repository pattern, create a Cloudflare Worker entry point with D1 backend, build Dioxus frontend to WASM, and deploy as a single Worker.

## Phase 1: Repository Abstraction

Extract database operations behind an async trait so route handlers work with both rusqlite (local) and D1 (Worker).

### Tasks

- [ ] Task 1.1: Define `Repository` trait + `RepoError` in `crates/core/src/repo.rs`. Methods: `list_restaurants`, `get_restaurant`, `create_restaurant`, `list_orders`, `get_order`, `create_order`, `update_order_status`, `assign_courier`, `list_couriers`, `create_courier`, `toggle_courier_available`. Add `async-trait` to workspace deps. Export from `crates/core/src/lib.rs`.
- [ ] Task 1.2: Create `crates/handlers/` crate (`openwok-handlers`). Add to workspace members. Dependencies: axum (workspace), openwok-core, serde, serde_json, uuid, chrono. Move route handler functions from `crates/api/src/routes/{restaurants,orders,couriers}.rs` here, replacing `State(state): State<AppState>` + raw SQL with `State(repo): State<R>` + `repo.method().await`. Keep request/response DTOs (CreateOrder, CreateRestaurant, etc.) in handlers crate. Do NOT include WebSocket handler (stays in api).
- [ ] Task 1.3: Implement `SqliteRepo` in `crates/api/src/sqlite_repo.rs`. Wraps `Arc<Mutex<Connection>>`. Implements all `Repository` trait methods using existing rusqlite queries extracted from the old route handlers.
- [ ] Task 1.4: Refactor `crates/api/src/main.rs` — use `openwok-handlers::api_routes(repo)` for shared routes, add WebSocket route locally. Update `AppState` to hold `SqliteRepo` + `broadcast::Sender`. Update `crates/api/Cargo.toml` to depend on `openwok-handlers`.
- [ ] Task 1.5: Run `make check` — all 37 tests pass, clippy clean, fmt clean. Fix any issues from refactoring.

### Verification

- [ ] `cargo test --workspace` passes with no regressions
- [ ] `cargo run -p openwok-api` starts and serves API at `localhost:3000/api`
- [ ] API endpoints return same responses as before (manual curl test)

## Phase 2: Worker Crate + D1

Create the Cloudflare Worker entry point that uses D1 for persistence.

### Tasks

- [ ] Task 2.1: Create `crates/worker/` crate (`openwok-worker`). Add to workspace members. Dependencies: `worker = { version = "0.5", features = ["axum", "d1"] }`, openwok-core, openwok-handlers, serde, serde_json, uuid (with `js` feature), chrono (with `wasm-bindgen` feature). Set `crate-type = ["cdylib"]`. Add `wasm32-unknown-unknown` target considerations.
- [ ] Task 2.2: Implement `D1Repo` in `crates/worker/src/d1_repo.rs`. Uses `worker::d1::D1Database` from Worker env. Implements all `Repository` trait methods using D1 prepared statements. Same SQL as SqliteRepo but via D1 API (`prepare().bind()?.all()`).
- [ ] Task 2.3: Create `crates/worker/src/lib.rs` with `#[event(fetch)]` entry point. Extract `D1Database` from `Env`, create `D1Repo`, build axum router via `openwok_handlers::api_routes(repo)`, dispatch request. Handle seed data: run seed on first request if restaurants table empty (same seed data as `db::seed_la_data`).
- [ ] Task 2.4: Update `wrangler.toml` — set `main = "crates/worker/build/worker/shim.mjs"` or configure `worker-build` output path. Verify `wrangler dev` starts locally with D1 local mode. Test API endpoints via curl against wrangler dev server.

### Verification

- [ ] `wrangler dev` starts without errors
- [ ] `curl localhost:8787/api/health` returns "ok"
- [ ] `curl localhost:8787/api/restaurants` returns seeded LA restaurants
- [ ] Order creation + status transitions work via curl

## Phase 3: Frontend Build Pipeline

Build Dioxus SPA to WASM and integrate with Worker static asset serving.

### Tasks

- [ ] Task 3.1: Update `crates/frontend/Dioxus.toml` — add `[web.wasm_opt]` with `level = "z"`, ensure `index_on_404 = true` is set. Verify `API_BASE = "/api"` in frontend code points to same-origin (no proxy needed in production).
- [ ] Task 3.2: Create `scripts/build.sh` — runs `dx build --platform web --release` from `crates/frontend/`, copies WASM output (HTML + JS + WASM + assets) to `public/` at repo root for wrangler `[assets]` binding. Make executable. Add `build-frontend` and `build-worker` targets to Makefile.
- [ ] Task 3.3: Run full build pipeline (`make build-frontend`) and verify: WASM bundle exists in `public/`, index.html present, `wrangler dev` serves SPA on `/*` and API on `/api/*`. Test: open browser, navigate all 6 routes, place order end-to-end.

### Verification

- [ ] `public/` contains index.html + WASM bundle after build
- [ ] SPA loads in browser via `wrangler dev`
- [ ] Client-side routing works for all 6 routes (Home, RestaurantList, RestaurantMenu, Checkout, OrderTracking, OperatorConsole)
- [ ] Order flow works: browse → add to cart → checkout → track

## Phase 4: Deploy & Verify

Deploy to Cloudflare Workers and verify live URL.

### Tasks

- [ ] Task 4.1: Create D1 databases — `wrangler d1 create openwok-db` and `wrangler d1 create openwok-db-dev`. Update `database_id` fields in `wrangler.toml` with actual IDs. Apply migrations: `wrangler d1 migrations apply openwok-db --remote`.
- [ ] Task 4.2: Deploy — `wrangler deploy`. Verify deployment URL in output. If build fails, fix wasm32 compilation issues (missing features, incompatible deps).
- [ ] Task 4.3: Smoke test live URL — `curl https://<worker-url>/api/health`, verify restaurants endpoint returns seeded data, test SPA loads in browser, run full order flow.

### Verification

- [ ] Live URL returns HTTP 200 on `/api/health`
- [ ] Restaurants and menu data loads from D1
- [ ] SPA routes work (client-side routing, 404 → index.html)
- [ ] End-to-end order flow works on live URL

## Phase 5: Docs & Cleanup

### Tasks

- [ ] Task 5.1: Update `CLAUDE.md` — add worker crate to workspace structure, add `make build-frontend` and `make build-worker` commands, update run commands with wrangler dev instructions, note that WebSocket is local-only.
- [ ] Task 5.2: Update `README.md` — add deployment section with `wrangler deploy` instructions, document the build pipeline, add live URL.
- [ ] Task 5.3: Remove dead code — delete `Dockerfile` and `fly.toml` (leftover from failed deploy attempts, not used), clean up any unused imports or stale configs.

### Verification

- [ ] CLAUDE.md reflects current project state (new crates, commands)
- [ ] `make check` passes (tests + clippy + fmt)
- [ ] No orphaned files or unused dependencies

## Final Verification

- [ ] All acceptance criteria from spec met
- [ ] Tests pass (`make check`)
- [ ] Clippy clean
- [ ] Build succeeds (both native and WASM)
- [ ] Live URL working
- [ ] Documentation up to date

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
