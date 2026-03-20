# Specification: Courier Dispatch & Dashboard

**Track ID:** courier-dispatch_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

The pilot requires couriers to accept and complete deliveries — currently there's no courier-facing UI, no auto-dispatch, and the WebSocket broadcast channel never fires events. This track adds: (1) auto-dispatch that assigns an available courier when an order reaches `ReadyForPickup`, (2) a courier dashboard page to view active delivery and update status, (3) WebSocket event firing on all order status transitions, and (4) courier self-registration with zone selection.

Scoped to the minimum needed for the 60-day LA pilot (20-40 couriers, 10-20 restaurants, 6 zones). No geolocation, no multi-order batching, no push notifications — those are wave 2.

## Acceptance Criteria

- [x] When order transitions to `ReadyForPickup`, system auto-assigns an available courier in the same zone (existing FIFO logic) and transitions order to `InDelivery`
- [x] If no courier available, order stays at `ReadyForPickup` (operator assigns manually from console)
- [x] WebSocket broadcasts `OrderEvent` on every `update_order_status` and `assign_courier` call
- [x] Courier dashboard page (`/my-deliveries`) shows: current active delivery (order details, restaurant name, customer address, status) and delivery history
- [x] Courier can update order status from dashboard: `InDelivery` → `Delivered` (with confirmation)
- [x] Courier self-registration page (`/register-courier`) with name + zone selection (requires auth)
- [x] `GET /api/my/deliveries` endpoint returns orders assigned to current courier (auth)
- [x] `GET /api/couriers/me` endpoint returns current user's courier profile (auth)
- [x] DB index on `couriers(zone_id, available)` for dispatch query performance
- [x] Integration tests for dispatch service (auto-assign + no-courier-available fallback)
- [x] Existing 98 tests still pass, `make check` clean

## Dependencies

- Supabase Auth (Google OAuth) — already implemented
- WebSocket infrastructure (`crates/api/src/ws.rs`) — exists but needs event firing
- Order state machine (`crates/core/src/order.rs`) — complete, no changes needed
- Repository pattern (`crates/core/src/repo.rs`) — extend with new queries

## Out of Scope

- Geolocation / GPS tracking of couriers
- Push notifications (SMS, email, mobile push)
- Multi-order batching / route optimization
- Courier earnings calculation / payout tracking
- Accept/reject flow (courier auto-receives for MVP)
- Distance-based dispatch scoring (zone match is sufficient for pilot)
- D1Repo / Cloudflare Worker changes (local-first, deploy in next track)

## Technical Notes

- `assign_courier` in SqliteRepo (line 465-501) already does zone-based FIFO — reuse as dispatch core
- `OrderEvent` broadcast channel exists in `AppState` (state.rs:19) but `send()` is never called — wire it into `update_order_status` and `assign_courier` handlers
- Courier struct lacks `user_id` field — need migration to link courier to auth user (like `restaurants.owner_id`)
- Frontend is a single `main.rs` (1822 lines) — add new pages following existing patterns, but as separate component functions
- Retro recommends splitting main.rs into modules — out of scope for this track but noted
