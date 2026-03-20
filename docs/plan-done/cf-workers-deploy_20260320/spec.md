# Specification: Cloudflare Workers Deployment

**Track ID:** cf-workers-deploy_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Complete the migration from local axum+tokio+rusqlite server to a single Cloudflare Worker serving both the API (`/api/*`) and the Dioxus WASM SPA (`/*`). This is the remaining work from PRD Phase 5 — 6 of 11 tasks are already done (wrangler.toml, D1 migrations, SQLite persistence, `/api/*` prefix, WASM release profile, LA seed data).

The key challenge: route handlers are tightly coupled to `rusqlite`. Since rusqlite uses C bindings and can't compile to `wasm32-unknown-unknown`, we need a Repository abstraction that allows the same API logic to run against either rusqlite (local dev/tests) or D1 (Cloudflare Workers). WebSocket stays local-only for MVP (frontend already has polling via Refresh button).

## Acceptance Criteria

- [ ] Repository trait defined in `crates/core/` — all DB operations abstracted behind async trait
- [ ] Route handlers extracted to `crates/handlers/` — generic over Repository, no rusqlite dependency
- [ ] `crates/worker/` crate exists with `worker::event!(fetch)` entry point and D1Repository
- [ ] `wrangler dev` starts locally: API on `/api/*`, SPA on `/*`
- [ ] Dioxus SPA builds to WASM (`dx build --platform web --release`), bundle works in browser
- [ ] All existing tests pass (`make check`) — no regressions from refactor
- [ ] Single `wrangler deploy` deploys API + frontend, live URL returns HTTP 200
- [ ] Order flow works end-to-end on live URL: browse restaurants → place order → track status

## Dependencies

- `worker` crate (workers-rs) 0.5+ with `axum` + `d1` features
- `dioxus-cli` (`dx`) for WASM frontend build
- `wrangler` 4.60.0 (already installed) for D1 + deployment
- Cloudflare account with Workers + D1 access

## Out of Scope

- WebSocket support in Worker (requires Durable Objects — future track)
- Auth (Phase 6)
- Payments (Phase 6)
- Custom domain setup
- CI/CD pipeline

## Technical Notes

- **Architecture:** Extract route handlers into `crates/handlers/` crate that both `crates/api/` and `crates/worker/` depend on. Handlers are generic over `Repository` trait. This prevents code duplication and keeps local dev server identical to production logic.
- **DB abstraction:** `Repository` trait in `crates/core/src/repo.rs` with async methods. `SqliteRepo` (rusqlite) in `crates/api/`. `D1Repo` (worker::d1) in `crates/worker/`.
- **Worker entry point:** `worker::event!(fetch)` macro → builds axum Router with D1Repo → returns HTTP response. Static assets served via `[assets]` binding in wrangler.toml with `not_found_handling = "single-page-application"`.
- **WASM compatibility:** `crates/handlers/` and `crates/core/` must compile to `wasm32-unknown-unknown`. Add `js` feature to uuid, `wasm-bindgen` feature to chrono where needed.
- **WebSocket:** Stays in `crates/api/` only (uses `tokio::spawn`/`tokio::select!`). Not exposed in worker. Frontend polling is sufficient for MVP.
- **Build pipeline:** `dx build --platform web --release` → copy dist to `public/` → `wrangler deploy`.
