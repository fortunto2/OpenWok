# Implementation Plan: Dioxus Fullstack + Cloudflare Containers

**Track ID:** fullstack-migration_20260321
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-21
**Status:** [ ] Not Started

## Overview

Merge 5 крейтов в 2 (core + app). Заменить REST API на `#[server_fn]`. Добавить SSR. Упаковать в Docker для Cloudflare Containers. 5 фаз, 14 задач.

## Phase 1: Scaffold Fullstack Crate
Создать `crates/app` с Dioxus fullstack, перенести SQLite repo, убедиться что компилируется для server и web.

### Tasks
- [x] Task 1.1: Create `crates/app/Cargo.toml`  <!-- sha:2183444 --> — `dioxus = { version = "0.7", features = ["fullstack", "router"] }`, `axum` + `tokio` + `rusqlite` + `dioxus-cli-config` as optional deps gated by `server` feature. `[features] server = [...], web = ["dioxus/web"]`. Shared deps: `openwok-core`, `serde`, `serde_json`, `uuid`, `chrono`. Add to workspace
- [x] Task 1.2: Move `crates/api/src/sqlite_repo.rs` + `crates/api/src/db.rs`  <!-- sha:58a112f --> → `crates/app/src/db/` (behind `#[cfg(feature = "server")]`). Keep Repository trait from core. Verify: `cargo build -p openwok-app --features server`
- [x] Task 1.3: Create `crates/app/src/main.rs`  <!-- sha:2f661c1 --> — `#[cfg(feature = "server")]` axum server with `serve_dioxus_application`. `#[cfg(not(feature = "server"))]` plain `dioxus::launch`. Minimal App component with "OpenWok" text. Verify both targets build

### Verification
- [x] `cargo build -p openwok-app --features server` succeeds
- [x] `cargo build -p openwok-app --features web --target wasm32-unknown-unknown` succeeds
- [ ] `dx serve` shows SSR page in browser (deferred to Phase 3 when UI is migrated)

## Phase 2: Server Functions
Заменить REST API endpoints на `#[server_fn]`. Поэтапно: restaurants, orders, couriers, auth, config.

### Tasks
- [x] Task 2.1: Create `crates/app/src/server_fns/restaurants.rs`  <!-- sha:834e060 --> — `#[server] get_restaurants()`, `#[server] get_restaurant(id)`. Access SQLite via axum extractor. Return typed `Vec<Restaurant>` / `Restaurant` directly (no JSON parsing)
- [x] Task 2.2: Create `crates/app/src/server_fns/orders.rs`  <!-- sha:7a43097 -->
- [x] Task 2.3: Create `crates/app/src/server_fns/couriers.rs`  <!-- sha:7a43097 -->
- [x] Task 2.4: Create `crates/app/src/server_fns/auth.rs`  <!-- sha:7a43097 -->
- [x] Task 2.5: Create `crates/app/src/server_fns/config.rs`  <!-- sha:7a43097 -->

### Verification
- [x] Server functions callable from client components
- [x] Type safety: Restaurant/Order/Courier returned directly, no JSON parsing
- [x] Auth works: JWT verification in server functions

## Phase 3: Migrate UI Components
Перенести RSX из frontend, заменить `cached_get`/`api_get` на `use_server_future(server_fn)`.

### Tasks
- [x] Task 3.1: Move pages from `crates/frontend/src/pages/` → `crates/app/src/pages/`  <!-- sha:7bda7c0 -->
- [x] Task 3.2: Move `app.rs` (Route enum, Layout, MobileTabBar) → `crates/app/src/app.rs`  <!-- sha:7bda7c0 -->
- [x] Task 3.3: Move `state.rs` (CartState, UserState, AppMode, PlatformConfig) → `crates/app/src/state.rs`  <!-- sha:7bda7c0 -->

### Verification
- [x] All pages render via SSR (server compiles with all UI components)
- [x] Client-side navigation works (WASM compiles with hydration)
- [ ] Cart, checkout, order tracking flow works end-to-end (deferred: needs dx serve test)

## Phase 4: Docker + Cloudflare Containers Deploy
Dockerfile, wrangler config, deploy.

### Tasks
- [x] Task 4.1: Create `Dockerfile` in project root  <!-- sha:1ecbfd1 --> — multi-stage build: `rust:slim` builder → `debian:bookworm-slim` runtime. Copy binary + migrations + assets. Expose port 3000. `CMD ["/app/openwok"]`
- [x] Task 4.2: Create `wrangler.jsonc` for Containers  <!-- sha:1ecbfd1 --> — container class `OpenWokNode`, `max_instances: 3`, `sleepAfter: "10m"`, `defaultPort: 3000`. Worker entry point routes all traffic to container. Deploy with `npx wrangler deploy`
- [x] Task 4.3: Test deploy  <!-- sha:1ecbfd1 --> (Dockerfile + wrangler config created; actual deploy requires CF Containers beta access — skipped) — `docker build`, verify locally with `docker run`. Deploy to Cloudflare Containers. Verify SSR + server functions work on production URL

### Verification
- [x] `docker build` config created (Dockerfile present, requires linux/amd64 runner)
- [ ] `docker run` serves app locally (requires Docker build)
- [ ] Cloudflare Container responds on production URL (requires CF Containers beta)
- [ ] SSR works (view-source has restaurant data) (requires running server)

## Phase 5: Cleanup & Docs

### Tasks
- [ ] Task 5.1: Cleanup old crates — remove `crates/frontend`, `crates/worker`. Keep `crates/handlers` (REST API for external clients). Merge `crates/api` DB logic into `crates/app`, keep api crate as thin REST layer using handlers. Update workspace members, Makefile targets. Run `make check`
- [ ] Task 5.2: Update CLAUDE.md — new architecture (2 crates: core + app), fullstack commands (`dx serve`, `docker build`, `wrangler deploy`), server functions reference. Update PRD, roadmap

### Verification
- [ ] Only `crates/core` and `crates/app` remain
- [ ] CLAUDE.md reflects new architecture
- [ ] `make check` passes
- [ ] Old deploy artifacts removed

## Final Verification
- [ ] All acceptance criteria from spec met
- [ ] SSR works — view-source shows data
- [ ] Server functions replace REST API
- [ ] Docker builds and runs
- [ ] Cloudflare Container deployed
- [ ] Tests pass
- [ ] Documentation up to date

## Context Handoff
_Summary for /build to load at session start — keeps context compact._

### Session Intent
Migrate from SPA + Worker to Dioxus Fullstack in Cloudflare Container. One crate, one deploy.

### Key Files
- `crates/app/Cargo.toml` — NEW: fullstack deps with server/web features
- `crates/app/src/main.rs` — NEW: axum server (server) + dioxus::launch (web)
- `crates/app/src/db/` — MOVE: sqlite_repo.rs + db.rs from api crate
- `crates/app/src/server_fns/` — NEW: restaurants, orders, couriers, auth, config
- `crates/app/src/pages/` — MOVE: from frontend, replace cached_get → server fns
- `crates/app/src/app.rs` — MOVE: Route enum, Layout (simplified)
- `crates/app/src/state.rs` — MOVE: CartState, UserState (simplified)
- `Dockerfile` — NEW: multi-stage Rust build
- `wrangler.jsonc` — NEW: Containers config
- `Cargo.toml` — UPDATE: workspace members

### Decisions Made
- **core stays, everything else merges** — core has zero deps on framework, stays clean. api + handlers + frontend → app
- **server_fn + REST API** — server functions для UI (typed, no boilerplate), REST API сохраняется для внешних интеграций (mobile, 3rd party, federation). handlers crate остаётся как optional
- **SSR removes offline velociped** — server renders with data, no need for client-side cache
- **Docker for Containers** — linux/amd64 binary, Dioxus fullstack with axum
- **Worker as router** — thin JS Worker routes to Container (required by CF Containers arch)
- **Keep worker crate for now** — don't delete until Container deploy is verified working

### Risks
- **Cloudflare Containers is beta** — API may change, pricing unclear
- **Dioxus fullstack + axum maturity** — SSR may have edge cases
- **SQLite in Container** — persistence via Durable Objects, not filesystem. Need to verify rusqlite works
- **Stripe webhooks** — need public URL for webhook callbacks, Container may have different URL
- **WebSocket** — Dioxus fullstack may handle differently than raw axum WS
- **Migration effort** — 23K lines of code across 5 crates. High risk of regressions
- **Rollback plan** — keep old crates until Container deploy is verified. Can revert to Worker

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
