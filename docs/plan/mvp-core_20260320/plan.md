# Implementation Plan: OpenWok MVP Core

**Track ID:** mvp-core_20260320
**Spec:** [spec.md](./spec.md)
**Created:** 2026-03-20
**Status:** [ ] Not Started

## Overview

Build the Rust codebase from scratch: Cargo workspace with two crates (`core` for domain logic, `api` for axum server). TDD throughout — tests before implementation for pricing calculator and order state machine. In-memory storage; persistence comes in a later track.

## Phase 1: Workspace & Domain Types <!-- checkpoint:19ff514 -->
Set up the Cargo workspace and define all domain types with serialization.

### Tasks
- [x] Task 1.1: Create Cargo workspace with two crates: `crates/core` (lib) and `crates/api` (bin). Root `Cargo.toml` as workspace. Dependencies: `rust_decimal`, `serde`, `uuid`, `chrono`, `thiserror`. <!-- sha:fa4487c -->
- [x] Task 1.2: Define money type in `crates/core/src/money.rs` — newtype over `Decimal` with `Display` (formats as `$X.XX`), arithmetic ops, `From<&str>` parsing. TDD. <!-- sha:7165045 -->
- [x] Task 1.3: Define domain types in `crates/core/src/types.rs` — `RestaurantId`, `CourierId`, `OrderId`, `NodeId`, `ZoneId` as UUID newtypes. `MenuItem` (id, name, price, restaurant_id). `Restaurant` (id, name, zone_id, menu items, active). `Zone` (id, name, node_id). `Node` (id, name, local_ops_fee, zones). `CourierKind` enum (Human). `Courier` (id, name, kind, zone_id, available). <!-- sha:77375af -->
- [x] Task 1.4: Define `PricingBreakdown` in `crates/core/src/pricing.rs` — struct with 6 fields: food_total, delivery_fee, tip, federal_fee, local_ops_fee, processing_fee. Method `total()` returns sum. `Display` impl shows 6-line receipt. <!-- sha:ca2dd2b -->
- [x] Task 1.5: Define `OrderStatus` enum and `Order` struct in `crates/core/src/order.rs` — statuses: Created, Confirmed, Preparing, ReadyForPickup, InDelivery, Delivered, Cancelled. Order holds: id, items (vec of MenuItem + qty), restaurant_id, courier_id (Option), customer address (String), zone_id, status, pricing breakdown, timestamps (created_at, updated_at). <!-- sha:e80ed3b -->
- [x] Task 1.6: Wire up `crates/core/src/lib.rs` — re-export all modules. Verify `make check` passes. <!-- sha:19ff514 -->

### Verification
- [x] `cargo build --workspace` succeeds
- [x] `make check` passes (test + clippy + fmt)
- [x] All types serialize/deserialize with serde

## Phase 2: Pricing Calculator & Order State Machine <!-- checkpoint:466b151 -->
Implement the core business logic with TDD (tests first, then implementation).

### Tasks
- [x] Task 2.1: Write pricing calculator tests in `crates/core/src/pricing.rs` — test cases: basic order ($25 food, $5 delivery, $3 tip, $2.50 local ops), zero tip, zero delivery fee, large order ($200+), rounding to cents. Verify federal fee always $1.00, processing = subtotal * 2.9% + $0.30. <!-- sha:e546869 -->
- [x] Task 2.2: Implement `calculate_pricing()` function — inputs: food_total, delivery_fee, tip, local_ops_fee. Returns `PricingBreakdown`. Federal fee hardcoded. Processing = (food + delivery + tip + federal + local_ops) * 0.029 + 0.30. All math via `Decimal`. <!-- sha:e546869 -->
- [x] Task 2.3: Write order state machine tests in `crates/core/src/order.rs` — valid transitions: Created→Confirmed→Preparing→ReadyForPickup→InDelivery→Delivered. Cancel allowed from Created/Confirmed/Preparing. Invalid transitions return error. <!-- sha:466b151 -->
- [x] Task 2.4: Implement `Order::transition(&mut self, new_status)` — validates transition, updates status + updated_at. Returns `Result<(), OrderError>`. Add `OrderError` enum via thiserror. <!-- sha:466b151 -->
- [x] Task 2.5: Implement `Order::new()` constructor — takes items, restaurant_id, address, zone_id, calculates pricing via `calculate_pricing()`. Status starts as Created. <!-- sha:466b151 -->

### Verification
- [x] `make test` — all pricing tests pass (edge cases, rounding)
- [x] `make test` — all state machine tests pass (valid + invalid transitions)
- [x] `make clippy` — zero warnings

## Phase 3: REST API (axum)
HTTP server with in-memory state, exposing order flow and restaurant catalog.

### Tasks
- [ ] Task 3.1: Set up axum app skeleton in `crates/api/src/main.rs` — tokio runtime, Router, shared state (`Arc<RwLock<AppState>>`). `AppState` holds `HashMap` collections for restaurants, orders, couriers. Health endpoint: `GET /health`.
- [ ] Task 3.2: Restaurant endpoints in `crates/api/src/routes/restaurants.rs` — `GET /restaurants` (list), `GET /restaurants/:id`, `POST /restaurants` (create). Seed 3 sample LA restaurants on startup.
- [ ] Task 3.3: Order endpoints in `crates/api/src/routes/orders.rs` — `POST /orders` (create with items + address, returns order with pricing breakdown), `GET /orders/:id`, `PATCH /orders/:id/status` (transition). Validate restaurant exists, items exist.
- [ ] Task 3.4: Courier endpoints in `crates/api/src/routes/couriers.rs` — `GET /couriers` (list available), `POST /couriers` (register), `PATCH /couriers/:id/available` (toggle). `POST /orders/:id/assign` (assign courier).
- [ ] Task 3.5: Integration tests in `tests/api_tests.rs` — full flow: create restaurant, create order, see pricing breakdown in response, confirm order, assign courier, transition through states to delivered.

### Verification
- [ ] `cargo run -p openwok-api` starts server on port 3000
- [ ] `curl localhost:3000/health` returns 200
- [ ] Full order flow works via curl/httpie
- [ ] Integration tests pass

## Phase 4: Docs & Cleanup

### Tasks
- [ ] Task 4.1: Update `CLAUDE.md` with workspace structure, crate descriptions, and run commands (`cargo run -p openwok-api`)
- [ ] Task 4.2: Update `README.md` — add tech stack section, build/run instructions, API endpoint list
- [ ] Task 4.3: Remove dead code — unused imports, orphaned files, stale exports

### Verification
- [ ] CLAUDE.md reflects current project state
- [ ] Linter clean, tests pass
- [ ] `make check` green

## Final Verification

- [ ] All acceptance criteria from spec met
- [ ] `make test` — all pass
- [ ] `make clippy` — zero warnings
- [ ] `make fmt` — no formatting issues
- [ ] `cargo build --workspace` succeeds
- [ ] Server starts and serves API
- [ ] Pricing breakdown shows 6 lines in order response

---
_Generated by /plan. Tasks marked [~] in progress and [x] complete by /build._
