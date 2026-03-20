# Implementation Plan: Restaurant Order Management

**Track ID:** restaurant-orders_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [ ] Not Started

## Overview

Add the missing restaurant order management flow: Repository method → API endpoint → frontend Orders tab. Follows the exact pattern of courier dispatch (`list_courier_orders` → `/my/deliveries` → MyDeliveries page). 12 tasks across 3 phases.

## Phase 1: Backend — Repository + API
Add `list_restaurant_orders` to the Repository trait and expose via `GET /api/my/orders`.

### Tasks
- [x] Task 1.1: Add `list_restaurant_orders(restaurant_id: RestaurantId) -> Result<Vec<Order>, RepoError>` to Repository trait in `crates/core/src/repo.rs`. Follow the `list_courier_orders` pattern. <!-- sha:df9dfa5 -->
- [x] Task 1.2: Implement `list_restaurant_orders` in SqliteRepo (`crates/api/src/sqlite_repo.rs`). SQL: `SELECT * FROM orders WHERE restaurant_id = ? ORDER BY created_at DESC`. Include order items join (same as `get_order` pattern). <!-- sha:df9dfa5 -->
- [x] Task 1.3: Add integration test for `list_restaurant_orders` in `crates/api/src/sqlite_repo.rs` — create 2 restaurants, create orders for each, verify filtering returns only matching restaurant's orders, verify DESC ordering. <!-- sha:dd74994 -->
- [x] Task 1.4: Add `my_orders` handler in `crates/handlers/src/restaurants.rs` — `GET /api/my/orders`. Auth required, `get_active_user` check. Fetch user's restaurants via `list_restaurants_by_owner`, then collect orders across all of them via `list_restaurant_orders`. Return `Vec<Order>`. <!-- sha:0261693 -->
- [x] Task 1.5: Register `/my/orders` route in `crates/api/src/main.rs` (axum router) and `crates/handlers/src/lib.rs` (if needed). <!-- sha:0261693 -->
- [x] Task 1.6: Implement `list_restaurant_orders` in D1Repo (`crates/worker/src/d1_repo.rs`) with equivalent D1 SQL query. <!-- sha:27d0029 -->

### Verification
- [x] `cargo test -p openwok-api` passes with new integration test
- [x] `make check` clean (all tests + clippy + fmt)

## Phase 2: Frontend — Orders Tab
Add Orders tab to RestaurantSettings page with order cards and action buttons.

### Tasks
- [ ] Task 2.1: Add `fetch_my_orders` helper in `crates/frontend/src/api.rs` — `GET /api/my/orders` with JWT auth header. Returns `Vec<serde_json::Value>` (same pattern as other API helpers).
- [ ] Task 2.2: Add Orders tab to RestaurantSettings in `crates/frontend/src/pages/owner.rs`. Tab navigation: Info | Menu | Orders. Orders tab shows list of orders filtered to current restaurant ID.
- [ ] Task 2.3: Implement order card component in the Orders tab — show order ID (truncated), status badge (color-coded), items list, total price, customer address, and created_at timestamp.
- [ ] Task 2.4: Add action buttons per order status: "Accept" (Confirmed→Preparing), "Mark Ready" (Preparing→ReadyForPickup), "Cancel" (Confirmed/Preparing→Cancelled). Each calls `PATCH /api/orders/{id}/status` with the target status. Refresh order list after action.
- [ ] Task 2.5: Add auto-refresh: `use_future` with 15-second interval to re-fetch orders, showing a subtle "New orders" indicator when the count changes.

### Verification
- [ ] `dx build --platform web` compiles successfully
- [ ] Orders tab renders with mock/real data
- [ ] Action buttons trigger correct status transitions

## Phase 3: Deploy + Docs
Deploy to CF Workers and update documentation.

### Tasks
- [ ] Task 3.1: Build frontend (`cd crates/frontend && dx build --platform web --release`), copy to worker public, deploy via `wrangler deploy`.
- [ ] Task 3.2: Update CLAUDE.md — add `GET /api/my/orders` to API endpoints table, note restaurant order management in frontend routes.
- [ ] Task 3.3: Remove dead code — unused imports, verify no orphaned files from this track.

### Verification
- [ ] `make check` passes (all tests + clippy + fmt)
- [ ] CLAUDE.md reflects current project state

## Final Verification
- [ ] All acceptance criteria from spec met
- [ ] Tests pass (target: 108+ tests)
- [ ] Linter clean
- [ ] Build succeeds
- [ ] Documentation up to date

## Context Handoff
_Summary for /build to load at session start — keeps context compact._

### Session Intent
Enable restaurant owners to see and manage incoming orders via the dashboard, completing the pilot order flow.

### Key Files
- `crates/core/src/repo.rs` — add `list_restaurant_orders` to Repository trait
- `crates/api/src/sqlite_repo.rs` — implement SqliteRepo method + integration test
- `crates/handlers/src/restaurants.rs` — add `my_orders` handler
- `crates/api/src/main.rs` — register new route
- `crates/worker/src/d1_repo.rs` — implement D1Repo method
- `crates/frontend/src/pages/owner.rs` — Orders tab in RestaurantSettings
- `crates/frontend/src/api.rs` — add `fetch_my_orders` helper

### Decisions Made
- **Polling over WebSocket:** 15s polling is simpler and sufficient for pilot (10-20 restaurants, low order volume). WebSocket already used for customer order tracking — adding restaurant-side WS is over-engineering for MVP.
- **Aggregate endpoint:** Single `GET /api/my/orders` returns orders across ALL owner's restaurants (not per-restaurant). Simpler API, frontend filters client-side by restaurant_id for the tab view.
- **Reuse existing transition endpoint:** No new accept/reject endpoints. Restaurant uses `PATCH /api/orders/{id}/status` with `{"status": "Preparing"}` to accept. Keeps API surface minimal.
- **Tab in RestaurantSettings:** Not a separate page. Adds Orders tab alongside existing Info and Menu tabs, following the natural restaurant management flow.

### Risks
- **Order volume at scale:** `list_restaurant_orders` returns all orders (no pagination). Fine for pilot (<100 orders/restaurant), but needs pagination for growth. Out of scope.
- **Race condition:** Two browser tabs could both try to accept the same order. Repository layer handles this safely (status transition validates current state).
- **D1 query compatibility:** SqliteRepo and D1Repo must produce identical results. Same SQL syntax works for both (standard SQLite).

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
