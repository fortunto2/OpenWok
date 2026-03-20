# Implementation Plan: Courier Dispatch & Dashboard

**Track ID:** courier-dispatch_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [ ] Not Started

## Overview

Wire auto-dispatch into the order flow, fix the dead WebSocket channel, add courier self-registration + dashboard frontend pages. 3 phases, 12 tasks. Backend-first (TDD), then frontend, then docs.

## Phase 1: Dispatch Service + WebSocket Events
Backend foundation: link couriers to users, auto-dispatch on ReadyForPickup, fire WebSocket events.

### Tasks
- [x] Task 1.1: Add migration <!-- sha:4bae4d1 --> `migrations/0009_courier_user.sql` — add `user_id TEXT REFERENCES users(id)` to `couriers` table, add index on `couriers(zone_id, available)`, add index on `orders(courier_id)`
- [x] Task 1.2: Update <!-- sha:7222ba3 --> `Courier` struct in `crates/core/src/types.rs` — add `user_id: Option<String>` field. Update `CreateCourierRequest` in `crates/core/src/repo.rs` to include `user_id: Option<String>`
- [x] Task 1.3: Add Repository methods <!-- sha:d672d64 --> in `crates/core/src/repo.rs` — `get_courier_by_user_id(&self, user_id: &str) -> Result<Courier>` and `list_courier_orders(&self, courier_id: CourierId) -> Result<Vec<Order>>`
- [x] Task 1.4: Implement new repo methods <!-- sha:d672d64 --> in `crates/api/src/sqlite_repo.rs` — `get_courier_by_user_id` (SELECT by user_id), `list_courier_orders` (SELECT orders WHERE courier_id = ?), update `create_courier` to persist user_id, update `assign_courier` to use indexed query
- [x] Task 1.5: Create dispatch service <!-- sha:4e5f373 --> in `crates/core/src/dispatch.rs` — `pub async fn auto_dispatch<R: Repository>(repo: &R, order_id: OrderId) -> Result<Option<AssignCourierResult>>` that calls `repo.assign_courier()` and returns None if no courier available. Write unit tests: successful dispatch, no courier available, wrong zone
- [x] Task 1.6: Wire WebSocket events <!-- sha:32d5540 --> into order handlers in `crates/handlers/src/orders.rs` — after `update_order_status` succeeds, broadcast `OrderEvent { order_id, status }`. Requires adding `broadcast::Sender<OrderEvent>` to handler state. Also broadcast on `assign_courier` in `crates/handlers/src/couriers.rs`
- [x] Task 1.7: Wire auto-dispatch <!-- sha:83b5e9b --> into order status transition — in `crates/handlers/src/orders.rs::transition`, after successful transition to `ReadyForPickup`, call `auto_dispatch()`. If courier assigned, broadcast two events (CourierAssigned + status InDelivery)

### Verification
- [ ] `cargo test` — all existing 98 tests pass + new dispatch tests
- [ ] `make clippy` — zero warnings
- [ ] Manual test: create order → transition to ReadyForPickup → courier auto-assigned → WebSocket receives events

## Phase 2: Courier API Endpoints + Frontend Pages
Courier-facing endpoints and Dioxus pages for registration + delivery dashboard.

### Tasks
- [ ] Task 2.1: Add courier API endpoints in `crates/handlers/src/couriers.rs` — `GET /couriers/me` (get courier profile by auth user_id), `GET /my/deliveries` (list orders assigned to current courier). Both require `AuthUser` extractor
- [ ] Task 2.2: Register routes in `crates/handlers/src/lib.rs` and `crates/api/src/main.rs` — add `/api/couriers/me` and `/api/my/deliveries` to router
- [ ] Task 2.3: Add courier registration page in `crates/frontend/src/main.rs` — route `/register-courier`, form with name + zone dropdown (fetch zones from existing zone data). POST to `/api/couriers` with user_id from auth. Redirect to `/my-deliveries` on success
- [ ] Task 2.4: Add courier dashboard page in `crates/frontend/src/main.rs` — route `/my-deliveries`, shows active delivery (order details, restaurant, address, status badge) + delivery history list. "Mark Delivered" button calls `PATCH /orders/{id}/status` with `Delivered`. Connect to WebSocket for real-time status updates

### Verification
- [ ] `cargo test` — all tests pass
- [ ] `dx serve` — new pages render without console errors
- [ ] Manual flow: register as courier → receive auto-dispatched order → mark delivered from dashboard

## Phase 3: Docs & Cleanup

### Tasks
- [ ] Task 3.1: Update CLAUDE.md — add new endpoints (`GET /api/couriers/me`, `GET /api/my/deliveries`), add new routes (`/register-courier`, `/my-deliveries`), update migration table with 0009
- [ ] Task 3.2: Remove dead code — check for unused imports, verify no orphaned functions after refactoring handlers

### Verification
- [ ] CLAUDE.md reflects current project state
- [ ] `make check` passes (test + clippy + fmt)

## Final Verification
- [ ] All acceptance criteria from spec met
- [ ] Tests pass (`make test`)
- [ ] Linter clean (`make clippy`)
- [ ] Format clean (`make fmt`)
- [ ] Build succeeds (`cargo build`)
- [ ] Documentation up to date

## Context Handoff
_Summary for /build to load at session start — keeps context compact._

### Session Intent
Add auto-dispatch, WebSocket event firing, and courier-facing UI (registration + delivery dashboard) for LA pilot readiness.

### Key Files
- `crates/core/src/types.rs` — add `user_id` to Courier
- `crates/core/src/repo.rs` — new repo methods (get_courier_by_user_id, list_courier_orders)
- `crates/core/src/dispatch.rs` — NEW: auto-dispatch service
- `crates/api/src/sqlite_repo.rs` — implement new repo methods
- `crates/handlers/src/orders.rs` — wire WebSocket events + auto-dispatch on ReadyForPickup
- `crates/handlers/src/couriers.rs` — new endpoints (me, my/deliveries) + WebSocket on assign
- `crates/handlers/src/lib.rs` — register new routes
- `crates/api/src/main.rs` — register new routes
- `crates/api/src/state.rs` — OrderEvent broadcast (already exists, just needs `send()` calls)
- `crates/api/src/ws.rs` — already works, just needs events to be fired
- `crates/frontend/src/main.rs` — two new pages (/register-courier, /my-deliveries)
- `migrations/0009_courier_user.sql` — NEW: user_id column + indexes

### Decisions Made
- **FIFO dispatch, no scoring:** Zone-based FIFO is sufficient for 20-40 courier pilot. Distance/fairness scoring is wave 2.
- **No accept/reject:** Couriers auto-receive assignments for MVP simplicity. Reject flow adds complexity without pilot value.
- **WebSocket for real-time:** Already built, just needs event firing — no new infrastructure.
- **SqliteRepo only:** D1Repo changes deferred to deploy track. Local-first development.
- **No frontend split:** Adding pages to main.rs following existing pattern. Retro recommends splitting but that's a separate refactor track.

### Risks
- `broadcast::Sender<OrderEvent>` is in `AppState` which is specific to `crates/api` — handlers crate needs access. Solution: pass sender as additional state via `FromRef` or add it to handler parameters.
- Auto-dispatch race condition: two orders hitting ReadyForPickup simultaneously could assign same courier. Mitigation: `assign_courier` already uses `LIMIT 1` + `SET available = 0` — SQLite serializes writes, so safe for single-node pilot.
- Frontend main.rs growing (1822 → ~2000 lines). Acceptable for this track; splitting is a separate refactor.

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
