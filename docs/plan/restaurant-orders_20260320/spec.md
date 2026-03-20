# Specification: Restaurant Order Management

**Track ID:** restaurant-orders_20260320
**Type:** Feature
**Created:** 2026-03-20
**Status:** Draft

## Summary

Restaurants currently have zero visibility into incoming orders. After a customer pays and the order reaches `Confirmed` status, there is no UI or API for the restaurant owner to see, accept, or manage orders. This is the critical gap blocking pilot readiness — without it, the Confirmed → Preparing → ReadyForPickup flow is broken and couriers never get dispatched.

This track adds: (1) a Repository method to list orders by restaurant, (2) a `GET /api/my/orders` endpoint for restaurant owners, (3) an Orders tab in the restaurant dashboard with accept/ready/cancel actions, and (4) real-time polling for new orders. Follows the same pattern as courier dispatch (`/my/deliveries` + `list_courier_orders`).

## Acceptance Criteria

- [x] `list_restaurant_orders(restaurant_id)` method in Repository trait returns orders for a given restaurant, ordered by `created_at DESC`
- [x] SqliteRepo implements `list_restaurant_orders` with proper SQL query joining orders by `restaurant_id`
- [x] `GET /api/my/orders` endpoint returns orders across all restaurants owned by the current user (auth required, blocked-user check via `get_active_user`)
- [x] Restaurant owner can transition order: Confirmed → Preparing (accept)
- [x] Restaurant owner can transition order: Preparing → ReadyForPickup (mark ready)
- [x] Restaurant owner can cancel order from Confirmed or Preparing status
- [x] RestaurantSettings page has an "Orders" tab showing incoming/active orders with action buttons
- [x] Orders tab auto-refreshes every 15 seconds to show new incoming orders
- [x] D1Repo (worker crate) implements matching `list_restaurant_orders` query
- [x] Integration tests for `list_restaurant_orders` (returns only orders for specified restaurant, ordered correctly)
- [x] Existing 106 tests still pass, `make check` clean

## Dependencies

- auth-payments (Supabase Auth + JWT) — already implemented
- restaurant-onboarding (owner_id on restaurants) — already implemented
- courier-dispatch (auto-dispatch on ReadyForPickup) — already implemented
- Repository pattern (core trait + SqliteRepo + D1Repo) — already implemented

## Out of Scope

- Push notifications / sound alerts for new orders
- Order preparation time estimates
- Reject with reason (cancel is sufficient for MVP)
- Order history/analytics for restaurant owners (future track)
- WebSocket for restaurant real-time updates (polling is sufficient for pilot)
- Batch order operations

## Technical Notes

- Reference pattern: `list_courier_orders` in `crates/core/src/repo.rs:192` + `my_deliveries` handler in `crates/handlers/src/couriers.rs:117-133` + courier dashboard in `crates/frontend/src/pages/courier.rs`
- Order state machine already supports all needed transitions (Confirmed→Preparing, Preparing→ReadyForPickup, Confirmed/Preparing→Cancelled)
- Status transition handler `PATCH /api/orders/{id}/status` already exists and handles auto-dispatch on ReadyForPickup — restaurant just needs to call it
- Frontend uses `gloo_net` for HTTP + `use_resource` for data fetching (same as MyDeliveries pattern)
- Owner authorization: reuse `list_restaurants_by_owner` to verify the order belongs to one of the user's restaurants
